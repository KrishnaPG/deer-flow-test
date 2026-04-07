use std::path::Path;

use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteRow};
use sqlx::{ConnectOptions, Pool, Row, Sqlite};
use tracing::{self, log::LevelFilter};

use crate::config::{CatalogBackendConfig, CatalogConfig};
use crate::types::FileRecord;
use crate::types::VirtualFolderHierarchy;

pub struct Berg10Catalog {
    pool: Pool<Sqlite>,
}

impl Berg10Catalog {
    pub async fn new(config: &CatalogConfig) -> Result<Self> {
        let pool = Self::connect(config).await?;
        let this = Self { pool };
        this.ensure_schema().await?;
        Ok(this)
    }

    async fn connect(config: &CatalogConfig) -> Result<Pool<Sqlite>> {
        let path = match &config.backend {
            CatalogBackendConfig::Sql(sql) => sql.path.clone(),
            CatalogBackendConfig::Rest(rest) => {
                return Err(anyhow!(
                    "REST catalog backend is not implemented yet: {}",
                    rest.uri
                ));
            }
        };

        if path != ":memory:" {
            if let Some(parent) = Path::new(&path).parent() {
                if !parent.as_os_str().is_empty() {
                    std::fs::create_dir_all(parent)?;
                }
            }
        }

        let mut options = if path == ":memory:" {
            SqliteConnectOptions::new().filename(":memory:")
        } else {
            SqliteConnectOptions::new().filename(&path).create_if_missing(true)
        };

        options = options
            .journal_mode(SqliteJournalMode::Wal)
            .foreign_keys(true)
            .log_statements(LevelFilter::Trace);

        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(options)
            .await?;

        Ok(pool)
    }

    async fn ensure_schema(&self) -> Result<()> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS blobs (
                content_hash TEXT PRIMARY KEY,
                hierarchy TEXT NOT NULL,
                level TEXT NOT NULL,
                plane TEXT NOT NULL,
                payload_kind TEXT NOT NULL,
                payload_format TEXT NOT NULL,
                payload_size_bytes INTEGER NOT NULL,
                written_at TEXT NOT NULL,
                writer_identity TEXT,
                logical_filename TEXT
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS blob_correlation_ids (
                content_hash TEXT NOT NULL,
                key TEXT NOT NULL,
                value TEXT NOT NULL,
                PRIMARY KEY (content_hash, key, value),
                FOREIGN KEY (content_hash) REFERENCES blobs(content_hash) ON DELETE CASCADE
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS blob_lineage_refs (
                content_hash TEXT NOT NULL,
                ref TEXT NOT NULL,
                PRIMARY KEY (content_hash, ref),
                FOREIGN KEY (content_hash) REFERENCES blobs(content_hash) ON DELETE CASCADE
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS blob_tags (
                content_hash TEXT NOT NULL,
                tag_key TEXT NOT NULL,
                tag_value TEXT NOT NULL,
                PRIMARY KEY (content_hash, tag_key, tag_value),
                FOREIGN KEY (content_hash) REFERENCES blobs(content_hash) ON DELETE CASCADE
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS virtual_folder_hierarchies (
                hierarchy_name TEXT PRIMARY KEY,
                hierarchy_order_json TEXT NOT NULL,
                filter_expr TEXT,
                status TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_blobs_payload_kind ON blobs(payload_kind)")
            .execute(&self.pool)
            .await?;
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_blob_tags_key_value ON blob_tags(tag_key, tag_value)")
            .execute(&self.pool)
            .await?;
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_hierarchies_status ON virtual_folder_hierarchies(status)")
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn register_file(&self, record: &FileRecord) -> Result<()> {
        let mut tx = self.pool.begin().await?;

        sqlx::query(
            r#"
            INSERT INTO blobs (
                content_hash, hierarchy, level, plane, payload_kind, payload_format,
                payload_size_bytes, written_at, writer_identity, logical_filename
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(content_hash) DO NOTHING
            "#,
        )
        .bind(&record.content_hash)
        .bind(&record.hierarchy)
        .bind(&record.level)
        .bind(&record.plane)
        .bind(&record.payload_kind)
        .bind(&record.payload_format)
        .bind(record.payload_size_bytes as i64)
        .bind(record.written_at.to_rfc3339())
        .bind(&record.writer_identity)
        .bind(&record.logical_filename)
        .execute(&mut *tx)
        .await?;

        sqlx::query("DELETE FROM blob_correlation_ids WHERE content_hash = ?")
            .bind(&record.content_hash)
            .execute(&mut *tx)
            .await?;
        for (key, value) in &record.correlation_ids {
            sqlx::query(
                "INSERT INTO blob_correlation_ids (content_hash, key, value) VALUES (?, ?, ?)",
            )
            .bind(&record.content_hash)
            .bind(key)
            .bind(value)
            .execute(&mut *tx)
            .await?;
        }

        sqlx::query("DELETE FROM blob_lineage_refs WHERE content_hash = ?")
            .bind(&record.content_hash)
            .execute(&mut *tx)
            .await?;
        for reference in &record.lineage_refs {
            sqlx::query("INSERT INTO blob_lineage_refs (content_hash, ref) VALUES (?, ?)")
                .bind(&record.content_hash)
                .bind(reference)
                .execute(&mut *tx)
                .await?;
        }

        sqlx::query("DELETE FROM blob_tags WHERE content_hash = ?")
            .bind(&record.content_hash)
            .execute(&mut *tx)
            .await?;
        for (key, value) in &record.routing_tags {
            sqlx::query("INSERT INTO blob_tags (content_hash, tag_key, tag_value) VALUES (?, ?, ?)")
                .bind(&record.content_hash)
                .bind(key)
                .bind(value)
                .execute(&mut *tx)
                .await?;
        }

        tx.commit().await?;
        tracing::info!(content_hash = %record.content_hash, "Registered blob metadata");
        Ok(())
    }

    pub async fn get_file(&self, content_hash: &str) -> Result<Option<FileRecord>> {
        let row = sqlx::query(
            r#"
            SELECT content_hash, hierarchy, level, plane, payload_kind, payload_format,
                   payload_size_bytes, written_at, writer_identity, logical_filename
            FROM blobs
            WHERE content_hash = ?
            "#,
        )
        .bind(content_hash)
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some(row) => self.file_record_from_row(row).await.map(Some),
            None => Ok(None),
        }
    }

    pub async fn query_files(&self, filter: &str) -> Result<Vec<FileRecord>> {
        if filter.trim().is_empty() || filter.trim() == "*" {
            return self.load_all_files().await;
        }

        let normalized = Self::normalize_filter(filter)?;
        let sql = format!(
            r#"
            SELECT DISTINCT b.content_hash, b.hierarchy, b.level, b.plane, b.payload_kind, b.payload_format,
                            b.payload_size_bytes, b.written_at, b.writer_identity, b.logical_filename
            FROM blobs b
            LEFT JOIN blob_tags t ON t.content_hash = b.content_hash
            WHERE {}
            ORDER BY b.written_at DESC, b.content_hash ASC
            "#,
            normalized
        );

        let rows = sqlx::query(&sql).fetch_all(&self.pool).await?;
        let mut records = Vec::with_capacity(rows.len());
        for row in rows {
            records.push(self.file_record_from_row(row).await?);
        }
        Ok(records)
    }

    pub async fn create_virtual_folder_hierarchy(&self, hierarchy: &VirtualFolderHierarchy) -> Result<()> {
        let order = serde_json::to_string(&hierarchy.hierarchy_order)?;
        sqlx::query(
            r#"
            INSERT INTO virtual_folder_hierarchies (
                hierarchy_name, hierarchy_order_json, filter_expr, status, created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?)
            ON CONFLICT(hierarchy_name) DO UPDATE SET
                hierarchy_order_json = excluded.hierarchy_order_json,
                filter_expr = excluded.filter_expr,
                status = excluded.status,
                updated_at = excluded.updated_at
            "#,
        )
        .bind(&hierarchy.hierarchy_name)
        .bind(order)
        .bind(&hierarchy.filter_expr)
        .bind(&hierarchy.status)
        .bind(hierarchy.created_at.to_rfc3339())
        .bind(hierarchy.updated_at.to_rfc3339())
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn list_virtual_folder_hierarchies(&self, status: Option<&str>) -> Result<Vec<VirtualFolderHierarchy>> {
        let rows = match status {
            Some(status) => {
                sqlx::query(
                    r#"
                    SELECT hierarchy_name, hierarchy_order_json, filter_expr, status, created_at, updated_at
                    FROM virtual_folder_hierarchies
                    WHERE status = ?
                    ORDER BY hierarchy_name ASC
                    "#,
                )
                .bind(status)
                .fetch_all(&self.pool)
                .await?
            }
            None => {
                sqlx::query(
                    r#"
                    SELECT hierarchy_name, hierarchy_order_json, filter_expr, status, created_at, updated_at
                    FROM virtual_folder_hierarchies
                    ORDER BY hierarchy_name ASC
                    "#,
                )
                .fetch_all(&self.pool)
                .await?
            }
        };

        rows.into_iter().map(Self::hierarchy_from_row).collect()
    }

    pub async fn update_virtual_folder_hierarchy_status(&self, hierarchy_name: &str, status: &str) -> Result<()> {
        let updated_at = Utc::now().to_rfc3339();
        let result = sqlx::query(
            "UPDATE virtual_folder_hierarchies SET status = ?, updated_at = ? WHERE hierarchy_name = ?",
        )
        .bind(status)
        .bind(updated_at)
        .bind(hierarchy_name)
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(anyhow!("Virtual folder hierarchy not found: {}", hierarchy_name));
        }

        Ok(())
    }

    pub async fn delete_virtual_folder_hierarchy(&self, hierarchy_name: &str) -> Result<()> {
        let result = sqlx::query("DELETE FROM virtual_folder_hierarchies WHERE hierarchy_name = ?")
            .bind(hierarchy_name)
            .execute(&self.pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(anyhow!("Virtual folder hierarchy not found: {}", hierarchy_name));
        }

        Ok(())
    }

    pub async fn resolve_virtual_folder_hierarchy(&self, hierarchy_name: &str) -> Result<Vec<FileRecord>> {
        let hierarchy = sqlx::query(
            r#"
            SELECT hierarchy_name, hierarchy_order_json, filter_expr, status, created_at, updated_at
            FROM virtual_folder_hierarchies
            WHERE hierarchy_name = ? AND status = 'active'
            "#,
        )
        .bind(hierarchy_name)
        .fetch_optional(&self.pool)
        .await?;

        let Some(row) = hierarchy else {
            return Ok(Vec::new());
        };

        let hierarchy = Self::hierarchy_from_row(row)?;
        self.query_files(hierarchy.filter_expr.as_deref().unwrap_or("*")).await
    }

    async fn load_all_files(&self) -> Result<Vec<FileRecord>> {
        let rows = sqlx::query(
            r#"
            SELECT content_hash, hierarchy, level, plane, payload_kind, payload_format,
                   payload_size_bytes, written_at, writer_identity, logical_filename
            FROM blobs
            ORDER BY written_at DESC, content_hash ASC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        let mut records = Vec::with_capacity(rows.len());
        for row in rows {
            records.push(self.file_record_from_row(row).await?);
        }
        Ok(records)
    }

    async fn file_record_from_row(&self, row: SqliteRow) -> Result<FileRecord> {
        let content_hash: String = row.try_get("content_hash")?;

        let correlation_ids = sqlx::query(
            "SELECT key, value FROM blob_correlation_ids WHERE content_hash = ? ORDER BY key, value",
        )
        .bind(&content_hash)
        .fetch_all(&self.pool)
        .await?
        .into_iter()
        .map(|row| Ok((row.try_get("key")?, row.try_get("value")?)))
        .collect::<Result<Vec<(String, String)>>>()?;

        let lineage_refs = sqlx::query(
            "SELECT ref FROM blob_lineage_refs WHERE content_hash = ? ORDER BY ref",
        )
        .bind(&content_hash)
        .fetch_all(&self.pool)
        .await?
        .into_iter()
        .map(|row| row.try_get("ref"))
        .collect::<std::result::Result<Vec<String>, _>>()?;

        let routing_tags = sqlx::query(
            "SELECT tag_key, tag_value FROM blob_tags WHERE content_hash = ? ORDER BY tag_key, tag_value",
        )
        .bind(&content_hash)
        .fetch_all(&self.pool)
        .await?
        .into_iter()
        .map(|row| Ok((row.try_get("tag_key")?, row.try_get("tag_value")?)))
        .collect::<Result<Vec<(String, String)>>>()?;

        Ok(FileRecord {
            content_hash,
            hierarchy: row.try_get("hierarchy")?,
            level: row.try_get("level")?,
            plane: row.try_get("plane")?,
            payload_kind: row.try_get("payload_kind")?,
            payload_format: row.try_get("payload_format")?,
            payload_size_bytes: row.try_get::<i64, _>("payload_size_bytes")? as u64,
            correlation_ids,
            lineage_refs,
            routing_tags,
            written_at: parse_rfc3339(row.try_get::<String, _>("written_at")?)?,
            writer_identity: row.try_get("writer_identity")?,
            logical_filename: row.try_get("logical_filename")?,
        })
    }

    fn hierarchy_from_row(row: SqliteRow) -> Result<VirtualFolderHierarchy> {
        Ok(VirtualFolderHierarchy {
            hierarchy_name: row.try_get("hierarchy_name")?,
            hierarchy_order: serde_json::from_str(&row.try_get::<String, _>("hierarchy_order_json")?)?,
            filter_expr: row.try_get("filter_expr")?,
            status: row.try_get("status")?,
            created_at: parse_rfc3339(row.try_get::<String, _>("created_at")?)?,
            updated_at: parse_rfc3339(row.try_get::<String, _>("updated_at")?)?,
        })
    }

    fn normalize_filter(filter: &str) -> Result<String> {
        let normalized = filter
            .replace("routing_tags[\"", "tag:")
            .replace("routing_tags['", "tag:")
            .replace("\"]", "")
            .replace("']", "");

        if let Some((left, right)) = normalized.split_once('=') {
            let left = left.trim();
            let right = right.trim();
            if let Some(tag_key) = left.strip_prefix("tag:") {
                return Ok(format!("t.tag_key = '{}' AND t.tag_value = {}", escape_sql_string(tag_key), right));
            }
        }

        if normalized.contains(';') {
            return Err(anyhow!("Unsupported filter expression"));
        }

        Ok(normalized)
    }

    pub fn pool(&self) -> &Pool<Sqlite> {
        &self.pool
    }
}

fn parse_rfc3339(value: String) -> Result<DateTime<Utc>> {
    Ok(DateTime::parse_from_rfc3339(&value)?.with_timezone(&Utc))
}

fn escape_sql_string(value: &str) -> String {
    value.replace('\'', "''")
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn catalog_initialization_creates_sqlite_file() {
        let tmp = TempDir::new().unwrap();
        let config = CatalogConfig::with_base_dir(tmp.path().to_str().unwrap());

        let catalog = Berg10Catalog::new(&config).await.unwrap();
        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='blobs'",
        )
        .fetch_one(catalog.pool())
        .await
        .unwrap();
        assert_eq!(count, 1);
    }

    #[tokio::test]
    async fn file_registration_and_retrieval() {
        let catalog = Berg10Catalog::new(&CatalogConfig::memory()).await.unwrap();

        let record = FileRecord {
            content_hash: "test_hash_123".to_string(),
            hierarchy: "A".to_string(),
            level: "L0".to_string(),
            plane: "as-is".to_string(),
            payload_kind: "chat-note".to_string(),
            payload_format: "jsonl".to_string(),
            payload_size_bytes: 100,
            correlation_ids: vec![("mission_id".to_string(), "m1".to_string())],
            lineage_refs: vec![],
            routing_tags: vec![("env".to_string(), "test".to_string())],
            written_at: Utc::now(),
            writer_identity: Some("test".to_string()),
            logical_filename: Some("note.jsonl".to_string()),
        };

        catalog.register_file(&record).await.unwrap();
        let retrieved = catalog.get_file("test_hash_123").await.unwrap().unwrap();
        assert_eq!(retrieved.content_hash, "test_hash_123");
        assert_eq!(retrieved.routing_tags, vec![("env".to_string(), "test".to_string())]);
    }

    #[tokio::test]
    async fn virtual_folder_hierarchy_crud_operations() {
        let catalog = Berg10Catalog::new(&CatalogConfig::memory()).await.unwrap();

        let hierarchy = VirtualFolderHierarchy {
            hierarchy_name: "test-hierarchy".to_string(),
            hierarchy_order: vec!["year".to_string(), "singer".to_string()],
            filter_expr: Some("payload_kind = 'mp3'".to_string()),
            status: "active".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        catalog.create_virtual_folder_hierarchy(&hierarchy).await.unwrap();
        let hierarchies = catalog.list_virtual_folder_hierarchies(None).await.unwrap();
        assert_eq!(hierarchies.len(), 1);

        catalog.update_virtual_folder_hierarchy_status("test-hierarchy", "inactive").await.unwrap();
        let active = catalog.list_virtual_folder_hierarchies(Some("active")).await.unwrap();
        assert!(active.is_empty());

        catalog.delete_virtual_folder_hierarchy("test-hierarchy").await.unwrap();
        let all = catalog.list_virtual_folder_hierarchies(None).await.unwrap();
        assert!(all.is_empty());
    }

    #[tokio::test]
    async fn query_files_supports_routing_tags() {
        let catalog = Berg10Catalog::new(&CatalogConfig::memory()).await.unwrap();

        let record = FileRecord {
            content_hash: "hash-adele".to_string(),
            hierarchy: "A".to_string(),
            level: "L0".to_string(),
            plane: "as-is".to_string(),
            payload_kind: "mp3".to_string(),
            payload_format: "mp3".to_string(),
            payload_size_bytes: 123,
            correlation_ids: vec![],
            lineage_refs: vec![],
            routing_tags: vec![("singer".to_string(), "Adele".to_string())],
            written_at: Utc::now(),
            writer_identity: None,
            logical_filename: Some("song1.mp3".to_string()),
        };
        catalog.register_file(&record).await.unwrap();

        let rows = catalog.query_files("routing_tags['singer'] = 'Adele'").await.unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].content_hash, "hash-adele");
    }
}
