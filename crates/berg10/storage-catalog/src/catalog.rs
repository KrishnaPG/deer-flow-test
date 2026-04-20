use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteRow};
use sqlx::{ConnectOptions, Pool, Row, Sqlite};
use std::path::Path;
use tracing::{self, log::LevelFilter};

use crate::config::{CatalogBackendConfig, CatalogConfig};
use crate::types::{
    ContentRecord, ContentTag, HierarchyPathSegment, HierarchyStatus, LineageRef, TagKey, TagValue,
    VirtualFolderHierarchy,
};

#[derive(Clone)]
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
            SqliteConnectOptions::new()
                .filename(&path)
                .create_if_missing(true)
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
        sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_blob_tags_key_value ON blob_tags(tag_key, tag_value)",
        )
        .execute(&self.pool)
        .await?;
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_hierarchies_status ON virtual_folder_hierarchies(status)")
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn register_content(&self, record: &ContentRecord) -> Result<()> {
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
        .bind(record.content_hash.as_str())
        .bind(record.data_hierarchy.as_str())
        .bind(record.data_level.as_str())
        .bind(record.storage_plane.as_str())
        .bind(record.payload_kind.as_str())
        .bind(record.payload_format.as_str())
        .bind(record.payload_size_bytes as i64)
        .bind(record.written_at.to_rfc3339())
        .bind(record.writer_identity.as_ref().map(|value| value.as_str()))
        .bind(record.logical_filename.as_ref().map(|value| value.as_str()))
        .execute(&mut *tx)
        .await?;

        sqlx::query("DELETE FROM blob_correlation_ids WHERE content_hash = ?")
            .bind(record.content_hash.as_str())
            .execute(&mut *tx)
            .await?;
        for tag in &record.correlation_ids {
            sqlx::query(
                "INSERT INTO blob_correlation_ids (content_hash, key, value) VALUES (?, ?, ?)",
            )
            .bind(record.content_hash.as_str())
            .bind(tag.key.as_str())
            .bind(tag.value.as_str())
            .execute(&mut *tx)
            .await?;
        }

        sqlx::query("DELETE FROM blob_lineage_refs WHERE content_hash = ?")
            .bind(record.content_hash.as_str())
            .execute(&mut *tx)
            .await?;
        for reference in &record.lineage_refs {
            sqlx::query("INSERT INTO blob_lineage_refs (content_hash, ref) VALUES (?, ?)")
                .bind(record.content_hash.as_str())
                .bind(reference.as_str())
                .execute(&mut *tx)
                .await?;
        }

        sqlx::query("DELETE FROM blob_tags WHERE content_hash = ?")
            .bind(record.content_hash.as_str())
            .execute(&mut *tx)
            .await?;
        for tag in &record.routing_tags {
            sqlx::query(
                "INSERT INTO blob_tags (content_hash, tag_key, tag_value) VALUES (?, ?, ?)",
            )
            .bind(record.content_hash.as_str())
            .bind(tag.key.as_str())
            .bind(tag.value.as_str())
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
        tracing::info!(content_hash = %record.content_hash, "Registered content metadata");
        Ok(())
    }

    pub async fn get_content(&self, content_hash: &str) -> Result<Option<ContentRecord>> {
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
            Some(row) => self.content_record_from_row(row).await.map(Some),
            None => Ok(None),
        }
    }

    pub async fn query_content(&self, filter: &str) -> Result<Vec<ContentRecord>> {
        if filter.trim().is_empty() || filter.trim() == "*" {
            return self.load_all_content().await;
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
            records.push(self.content_record_from_row(row).await?);
        }
        Ok(records)
    }

    pub async fn create_virtual_folder_hierarchy(
        &self,
        hierarchy: &VirtualFolderHierarchy,
    ) -> Result<()> {
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
        .bind(hierarchy.hierarchy_name.as_str())
        .bind(order)
        .bind(&hierarchy.filter_expr)
        .bind(hierarchy.status.as_str())
        .bind(hierarchy.created_at.to_rfc3339())
        .bind(hierarchy.updated_at.to_rfc3339())
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn list_virtual_folder_hierarchies(
        &self,
        status: Option<HierarchyStatus>,
    ) -> Result<Vec<VirtualFolderHierarchy>> {
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
                .bind(status.as_str())
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

    pub async fn update_virtual_folder_hierarchy_status(
        &self,
        hierarchy_name: &str,
        status: HierarchyStatus,
    ) -> Result<()> {
        let updated_at = Utc::now().to_rfc3339();
        let result = sqlx::query(
            "UPDATE virtual_folder_hierarchies SET status = ?, updated_at = ? WHERE hierarchy_name = ?",
        )
        .bind(status.as_str())
        .bind(updated_at)
        .bind(hierarchy_name)
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(anyhow!(
                "Virtual folder hierarchy not found: {}",
                hierarchy_name
            ));
        }

        Ok(())
    }

    pub async fn delete_virtual_folder_hierarchy(&self, hierarchy_name: &str) -> Result<()> {
        let result = sqlx::query("DELETE FROM virtual_folder_hierarchies WHERE hierarchy_name = ?")
            .bind(hierarchy_name)
            .execute(&self.pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(anyhow!(
                "Virtual folder hierarchy not found: {}",
                hierarchy_name
            ));
        }

        Ok(())
    }

    pub async fn resolve_virtual_folder_hierarchy(
        &self,
        hierarchy_name: &str,
    ) -> Result<Vec<ContentRecord>> {
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
        self.query_content(hierarchy.filter_expr.as_deref().unwrap_or("*"))
            .await
    }

    async fn load_all_content(&self) -> Result<Vec<ContentRecord>> {
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
            records.push(self.content_record_from_row(row).await?);
        }
        Ok(records)
    }

    async fn content_record_from_row(&self, row: SqliteRow) -> Result<ContentRecord> {
        let content_hash: String = row.try_get("content_hash")?;

        let correlation_ids = sqlx::query(
            "SELECT key, value FROM blob_correlation_ids WHERE content_hash = ? ORDER BY key, value",
        )
        .bind(&content_hash)
        .fetch_all(&self.pool)
        .await?
        .into_iter()
        .map(|row| {
            Ok(ContentTag::new(
                TagKey::new(row.try_get::<String, _>("key")?),
                TagValue::new(row.try_get::<String, _>("value")?),
            ))
        })
        .collect::<Result<Vec<ContentTag>>>()?;

        let lineage_refs =
            sqlx::query("SELECT ref FROM blob_lineage_refs WHERE content_hash = ? ORDER BY ref")
                .bind(&content_hash)
                .fetch_all(&self.pool)
                .await?
                .into_iter()
                .map(|row| row.try_get::<String, _>("ref").map(LineageRef::new))
                .collect::<std::result::Result<Vec<LineageRef>, _>>()?;

        let routing_tags = sqlx::query(
            "SELECT tag_key, tag_value FROM blob_tags WHERE content_hash = ? ORDER BY tag_key, tag_value",
        )
        .bind(&content_hash)
        .fetch_all(&self.pool)
        .await?
        .into_iter()
        .map(|row| {
            Ok(ContentTag::new(
                TagKey::new(row.try_get::<String, _>("tag_key")?),
                TagValue::new(row.try_get::<String, _>("tag_value")?),
            ))
        })
        .collect::<Result<Vec<ContentTag>>>()?;

        Ok(ContentRecord {
            content_hash: content_hash
                .parse::<berg10_storage_vfs::ContentHash>()
                .map_err(|e| anyhow!(e))?,
            data_hierarchy: row
                .try_get::<String, _>("hierarchy")?
                .parse()
                .map_err(|e: String| anyhow!(e))?,
            data_level: row
                .try_get::<String, _>("level")?
                .parse()
                .map_err(|e: String| anyhow!(e))?,
            storage_plane: row
                .try_get::<String, _>("plane")?
                .parse()
                .map_err(|e: String| anyhow!(e))?,
            payload_kind: row.try_get::<String, _>("payload_kind")?.into(),
            payload_format: row.try_get::<String, _>("payload_format")?.into(),
            payload_size_bytes: row.try_get::<i64, _>("payload_size_bytes")? as u64,
            correlation_ids,
            lineage_refs,
            routing_tags,
            written_at: parse_rfc3339(row.try_get::<String, _>("written_at")?)?,
            writer_identity: row
                .try_get::<Option<String>, _>("writer_identity")?
                .map(Into::into),
            logical_filename: row
                .try_get::<Option<String>, _>("logical_filename")?
                .map(Into::into),
        })
    }

    fn hierarchy_from_row(row: SqliteRow) -> Result<VirtualFolderHierarchy> {
        let hierarchy_order_json: String = row.try_get("hierarchy_order_json")?;
        Ok(VirtualFolderHierarchy {
            hierarchy_name: row.try_get::<String, _>("hierarchy_name")?.into(),
            hierarchy_order: parse_hierarchy_order(&hierarchy_order_json)?,
            filter_expr: row.try_get("filter_expr")?,
            status: row
                .try_get::<String, _>("status")?
                .parse()
                .map_err(|e: String| anyhow!(e))?,
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
                return Ok(format!(
                    "t.tag_key = '{}' AND t.tag_value = {}",
                    escape_sql_string(tag_key),
                    right
                ));
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
    use crate::types::{Berg10DataHierarchy, Berg10DataLevel, Berg10StoragePlane};
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

        let record = ContentRecord {
            content_hash: berg10_storage_vfs::ContentHash::new(b"test_hash_123"),
            data_hierarchy: Berg10DataHierarchy::Orchestration,
            data_level: Berg10DataLevel::L0,
            storage_plane: Berg10StoragePlane::AS_IS,
            payload_kind: "chat-note".to_string().into(),
            payload_format: "jsonl".to_string().into(),
            payload_size_bytes: 100,
            correlation_ids: vec![ContentTag::new("mission_id", "m1")],
            lineage_refs: vec![],
            routing_tags: vec![ContentTag::new("env", "test")],
            written_at: Utc::now(),
            writer_identity: Some("test".into()),
            logical_filename: Some("note.jsonl".into()),
        };

        catalog.register_content(&record).await.unwrap();
        let retrieved = catalog
            .get_content(record.content_hash.as_str())
            .await
            .unwrap()
            .unwrap();
        assert_eq!(retrieved.content_hash, record.content_hash);
        assert_eq!(retrieved.routing_tags, vec![ContentTag::new("env", "test")]);
    }

    #[tokio::test]
    async fn virtual_folder_hierarchy_crud_operations() {
        let catalog = Berg10Catalog::new(&CatalogConfig::memory()).await.unwrap();

        let hierarchy = VirtualFolderHierarchy {
            hierarchy_name: "test-hierarchy".into(),
            hierarchy_order: vec![
                HierarchyPathSegment::Tag("year".into()),
                HierarchyPathSegment::Tag("singer".into()),
            ],
            filter_expr: Some("payload_kind = 'mp3'".to_string()),
            status: HierarchyStatus::Active,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        catalog
            .create_virtual_folder_hierarchy(&hierarchy)
            .await
            .unwrap();
        let hierarchies = catalog.list_virtual_folder_hierarchies(None).await.unwrap();
        assert_eq!(hierarchies.len(), 1);

        catalog
            .update_virtual_folder_hierarchy_status("test-hierarchy", HierarchyStatus::Inactive)
            .await
            .unwrap();
        let active = catalog
            .list_virtual_folder_hierarchies(Some(HierarchyStatus::Active))
            .await
            .unwrap();
        assert!(active.is_empty());

        catalog
            .delete_virtual_folder_hierarchy("test-hierarchy")
            .await
            .unwrap();
        let all = catalog.list_virtual_folder_hierarchies(None).await.unwrap();
        assert!(all.is_empty());
    }

    #[tokio::test]
    async fn query_files_supports_routing_tags() {
        let catalog = Berg10Catalog::new(&CatalogConfig::memory()).await.unwrap();

        let record = ContentRecord {
            content_hash: berg10_storage_vfs::ContentHash::new(b"hash-adele"),
            data_hierarchy: Berg10DataHierarchy::Orchestration,
            data_level: Berg10DataLevel::L0,
            storage_plane: Berg10StoragePlane::AS_IS,
            payload_kind: "mp3".to_string().into(),
            payload_format: "mp3".to_string().into(),
            payload_size_bytes: 123,
            correlation_ids: vec![],
            lineage_refs: vec![],
            routing_tags: vec![ContentTag::new("singer", "Adele")],
            written_at: Utc::now(),
            writer_identity: None,
            logical_filename: Some("song1.mp3".into()),
        };
        catalog.register_content(&record).await.unwrap();

        let rows = catalog
            .query_content("routing_tags['singer'] = 'Adele'")
            .await
            .unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].content_hash, record.content_hash);
    }
}

fn parse_hierarchy_order(value: &str) -> Result<Vec<HierarchyPathSegment>> {
    serde_json::from_str(value).or_else(|_| {
        let legacy: Vec<String> = serde_json::from_str(value)?;
        legacy
            .into_iter()
            .map(|segment| segment.parse().map_err(|e: String| anyhow!(e)))
            .collect()
    })
}
