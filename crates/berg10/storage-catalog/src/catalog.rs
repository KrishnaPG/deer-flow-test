use std::collections::HashMap;
use std::sync::Arc;

use anyhow::{anyhow, Result};
use chrono::Utc;
use iceberg::spec::{NestedField, PrimitiveType, Schema, Type};
use iceberg::{Catalog, NamespaceIdent, TableCreation, TableIdent};
use iceberg::memory::MemoryCatalogBuilder;
use iceberg::CatalogBuilder;
use tracing;

use crate::config::{CatalogConfig, CatalogBackendConfig};
use crate::query::QueryEngine;
use crate::types::{FileRecord, ViewDefinition};

pub const BERG10_NAMESPACE: &str = "berg10";
pub const FILES_TABLE: &str = "files";
pub const VIEWS_TABLE: &str = "views";

pub struct Berg10Catalog {
    catalog: Arc<dyn Catalog>,
    warehouse_path: String,
    query_engine: QueryEngine,
}

impl Berg10Catalog {
    /// Create a new catalog from configuration.
    pub async fn new(config: &CatalogConfig) -> Result<Self> {
        let catalog = Self::build_catalog(config).await?;

        let ns = NamespaceIdent::new(BERG10_NAMESPACE.to_string());
        if !catalog.namespace_exists(&ns).await? {
            catalog.create_namespace(&ns, HashMap::new()).await?;
        }

        let this = Self {
            catalog,
            warehouse_path: config.warehouse_path.clone(),
            query_engine: QueryEngine::new(),
        };

        this.ensure_tables().await?;
        this.refresh_query_engine().await?;

        Ok(this)
    }

    async fn refresh_query_engine(&self) -> Result<()> {
        let all_files = self.load_all_files().await?;
        self.query_engine
            .register_files(&all_files)
            .map_err(|e| anyhow!("Query engine registration failed: {}", e))?;
        Ok(())
    }

    async fn load_all_files(&self) -> Result<Vec<FileRecord>> {
        let dir_path = format!(
            "{}/metadata/berg10/files",
            self.warehouse_path.trim_start_matches("file://")
        );

        let mut records = Vec::new();
        if std::path::Path::new(&dir_path).exists() {
            for entry in std::fs::read_dir(dir_path)? {
                let entry = entry?;
                if entry.path().extension().and_then(|e| e.to_str()) == Some("json") {
                    if let Ok(content) = std::fs::read_to_string(entry.path()) {
                        if let Ok(record) = serde_json::from_str::<FileRecord>(&content) {
                            records.push(record);
                        }
                    }
                }
            }
        }

        Ok(records)
    }

    async fn build_catalog(config: &CatalogConfig) -> Result<Arc<dyn Catalog>> {
        match &config.backend {
            CatalogBackendConfig::Sql(_) | CatalogBackendConfig::Rest(_) => {
                let warehouse = if config.warehouse_path.starts_with("memory://") {
                    "file:///tmp/berg10-warehouse".to_string()
                } else {
                    format!("file://{}", config.warehouse_path)
                };

                let catalog = MemoryCatalogBuilder::default()
                    .load(
                        "berg10",
                        HashMap::from([(
                            iceberg::memory::MEMORY_CATALOG_WAREHOUSE.to_string(),
                            warehouse,
                        )]),
                    )
                    .await?;

                Ok(Arc::new(catalog))
            }
        }
    }

    async fn ensure_tables(&self) -> Result<()> {
        let ns = NamespaceIdent::new(BERG10_NAMESPACE.to_string());

        // Create files table if not exists
        if !self.catalog.table_exists(&TableIdent::new(ns.clone(), FILES_TABLE.to_string())).await? {
            self.create_files_table(&ns).await?;
        }

        // Create views table if not exists
        if !self.catalog.table_exists(&TableIdent::new(ns.clone(), VIEWS_TABLE.to_string())).await? {
            self.create_views_table(&ns).await?;
        }

        Ok(())
    }

    async fn create_files_table(&self, ns: &NamespaceIdent) -> Result<()> {
        let schema = Self::files_schema();
        let creation = TableCreation::builder()
            .name(FILES_TABLE.to_string())
            .schema(schema)
            .build();

        self.catalog.create_table(ns, creation).await?;
        tracing::info!("Created berg10.files table");
        Ok(())
    }

    async fn create_views_table(&self, ns: &NamespaceIdent) -> Result<()> {
        let schema = Self::views_schema();
        let creation = TableCreation::builder()
            .name(VIEWS_TABLE.to_string())
            .schema(schema)
            .build();

        self.catalog.create_table(ns, creation).await?;
        tracing::info!("Created berg10.views table");
        Ok(())
    }

    fn files_schema() -> Schema {
        Schema::builder()
            .with_fields(
                vec![
                    NestedField::required(1, "content_hash", Type::Primitive(PrimitiveType::String)).into(),
                    NestedField::required(2, "hierarchy", Type::Primitive(PrimitiveType::String)).into(),
                    NestedField::required(3, "level", Type::Primitive(PrimitiveType::String)).into(),
                    NestedField::required(4, "plane", Type::Primitive(PrimitiveType::String)).into(),
                    NestedField::required(5, "payload_kind", Type::Primitive(PrimitiveType::String)).into(),
                    NestedField::required(6, "payload_format", Type::Primitive(PrimitiveType::String)).into(),
                    NestedField::required(7, "payload_size_bytes", Type::Primitive(PrimitiveType::Long)).into(),
                    NestedField::required(8, "correlation_ids", Type::Map(
                        iceberg::spec::MapType {
                            key_field: NestedField::required(9, "key", Type::Primitive(PrimitiveType::String)).into(),
                            value_field: NestedField::required(10, "value", Type::Primitive(PrimitiveType::String)).into(),
                        }
                    )).into(),
                    NestedField::required(11, "lineage_refs", Type::List(
                        iceberg::spec::ListType {
                            element_field: NestedField::required(12, "element", Type::Primitive(PrimitiveType::String)).into(),
                        }
                    )).into(),
                    NestedField::required(13, "routing_tags", Type::Map(
                        iceberg::spec::MapType {
                            key_field: NestedField::required(14, "key", Type::Primitive(PrimitiveType::String)).into(),
                            value_field: NestedField::required(15, "value", Type::Primitive(PrimitiveType::String)).into(),
                        }
                    )).into(),
                    NestedField::required(16, "written_at", Type::Primitive(PrimitiveType::Timestamp)).into(),
                    NestedField::optional(17, "writer_identity", Type::Primitive(PrimitiveType::String)).into(),
                    NestedField::optional(18, "logical_filename", Type::Primitive(PrimitiveType::String)).into(),
                ]
            )
            .build()
            .expect("valid files schema")
    }

    fn views_schema() -> Schema {
        Schema::builder()
            .with_fields(
                vec![
                    NestedField::required(1, "view_name", Type::Primitive(PrimitiveType::String)).into(),
                    NestedField::required(2, "hierarchy_order", Type::List(
                        iceberg::spec::ListType {
                            element_field: NestedField::required(3, "element", Type::Primitive(PrimitiveType::String)).into(),
                        }
                    )).into(),
                    NestedField::optional(4, "filter_expr", Type::Primitive(PrimitiveType::String)).into(),
                    NestedField::required(5, "status", Type::Primitive(PrimitiveType::String)).into(),
                    NestedField::required(6, "created_at", Type::Primitive(PrimitiveType::Timestamp)).into(),
                    NestedField::required(7, "updated_at", Type::Primitive(PrimitiveType::Timestamp)).into(),
                ]
            )
            .build()
            .expect("valid views schema")
    }

    /// Register a file in the catalog.
    pub async fn register_file(&self, record: &FileRecord) -> Result<()> {
        let metadata = serde_json::to_string(record)?;
        let file_path = format!(
            "{}/metadata/berg10/files/{}.json",
            self.warehouse_path.trim_start_matches("file://"),
            record.content_hash
        );

        if let Some(parent) = std::path::Path::new(&file_path).parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&file_path, metadata)?;

        tracing::info!(
            content_hash = %record.content_hash,
            "Registered file in catalog"
        );

        self.refresh_query_engine().await?;

        Ok(())
    }

    /// Look up a file by its content hash.
    pub async fn get_file(&self, content_hash: &str) -> Result<Option<FileRecord>> {
        let file_path = format!(
            "{}/metadata/berg10/files/{}.json",
            self.warehouse_path.trim_start_matches("file://"),
            content_hash
        );

        if std::path::Path::new(&file_path).exists() {
            let content = std::fs::read_to_string(&file_path)?;
            let record: FileRecord = serde_json::from_str(&content)?;
            Ok(Some(record))
        } else {
            Ok(None)
        }
    }

    /// Query files by a filter expression using the SQL query engine.
    pub async fn query_files(&self, filter: &str) -> Result<Vec<FileRecord>> {
        if filter.is_empty() {
            return self.load_all_files().await;
        }

        let records = self.query_engine
            .filter(filter)
            .await
            .map_err(|e| anyhow!("Query failed: {}", e))?;

        Ok(records)
    }

    /// Create a view definition.
    pub async fn create_view(&self, view: &ViewDefinition) -> Result<()> {
        let metadata = serde_json::to_string(view)?;
        let file_path = format!(
            "{}/metadata/berg10/views/{}.json",
            self.warehouse_path.trim_start_matches("file://"),
            view.view_name
        );

        if let Some(parent) = std::path::Path::new(&file_path).parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&file_path, metadata)?;

        tracing::info!(view_name = %view.view_name, "Created view definition");
        Ok(())
    }

    /// List views, optionally filtered by status.
    pub async fn list_views(&self, status: Option<&str>) -> Result<Vec<ViewDefinition>> {
        let dir_path = format!(
            "{}/metadata/berg10/views",
            self.warehouse_path.trim_start_matches("file://")
        );

        let mut views = Vec::new();
        if std::path::Path::new(&dir_path).exists() {
            for entry in std::fs::read_dir(dir_path)? {
                let entry = entry?;
                if entry.path().extension().and_then(|e| e.to_str()) == Some("json") {
                    let content = std::fs::read_to_string(entry.path())?;
                    if let Ok(view) = serde_json::from_str::<ViewDefinition>(&content) {
                        if let Some(s) = status {
                            if view.status == s {
                                views.push(view);
                            }
                        } else {
                            views.push(view);
                        }
                    }
                }
            }
        }

        Ok(views)
    }

    /// Update a view's status.
    pub async fn update_view_status(&self, view_name: &str, status: &str) -> Result<()> {
        let file_path = format!(
            "{}/metadata/berg10/views/{}.json",
            self.warehouse_path.trim_start_matches("file://"),
            view_name
        );

        if !std::path::Path::new(&file_path).exists() {
            return Err(anyhow!("View not found: {}", view_name));
        }

        let content = std::fs::read_to_string(&file_path)?;
        let mut view: ViewDefinition = serde_json::from_str(&content)?;
        view.status = status.to_string();
        view.updated_at = Utc::now();
        let updated = serde_json::to_string(&view)?;
        std::fs::write(&file_path, updated)?;

        Ok(())
    }

    /// Delete a view definition.
    pub async fn delete_view(&self, view_name: &str) -> Result<()> {
        let file_path = format!(
            "{}/metadata/berg10/views/{}.json",
            self.warehouse_path.trim_start_matches("file://"),
            view_name
        );

        if !std::path::Path::new(&file_path).exists() {
            return Err(anyhow!("View not found: {}", view_name));
        }

        std::fs::remove_file(&file_path)?;

        Ok(())
    }

    /// Resolve a view: get all files matching the view's filter and hierarchy.
    pub async fn resolve_view(&self, view_name: &str) -> Result<Vec<FileRecord>> {
        let views = self.list_views(Some("active")).await?;
        let view = match views.iter().find(|v| v.view_name == view_name) {
            Some(v) => v,
            None => return Ok(Vec::new()),
        };

        let filter = view.filter_expr.as_deref().unwrap_or("*");
        let files = self.query_files(filter).await?;

        Ok(files)
    }

    /// Get the underlying Iceberg catalog for advanced operations.
    pub fn iceberg_catalog(&self) -> &Arc<dyn Catalog> {
        &self.catalog
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn catalog_initialization_creates_tables() {
        let tmp = TempDir::new().unwrap();
        let config = CatalogConfig {
            warehouse_path: tmp.path().to_string_lossy().to_string(),
            ..Default::default()
        };

        let catalog = Berg10Catalog::new(&config).await.unwrap();
        assert!(catalog.catalog.table_exists(
            &TableIdent::new(
                NamespaceIdent::new(BERG10_NAMESPACE.to_string()),
                FILES_TABLE.to_string()
            )
        ).await.unwrap());
    }

    #[tokio::test]
    async fn file_registration_and_retrieval() {
        let tmp = TempDir::new().unwrap();
        let config = CatalogConfig {
            warehouse_path: tmp.path().to_string_lossy().to_string(),
            ..Default::default()
        };

        let catalog = Berg10Catalog::new(&config).await.unwrap();

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
            routing_tags: vec![],
            written_at: Utc::now(),
            writer_identity: Some("test".to_string()),
            logical_filename: Some("note.jsonl".to_string()),
        };

        catalog.register_file(&record).await.unwrap();
        let retrieved = catalog.get_file("test_hash_123").await.unwrap().unwrap();
        assert_eq!(retrieved.content_hash, "test_hash_123");
        assert_eq!(retrieved.payload_kind, "chat-note");
    }

    #[tokio::test]
    async fn view_crud_operations() {
        let tmp = TempDir::new().unwrap();
        let config = CatalogConfig {
            warehouse_path: tmp.path().to_string_lossy().to_string(),
            ..Default::default()
        };

        let catalog = Berg10Catalog::new(&config).await.unwrap();

        let view = ViewDefinition {
            view_name: "test-view".to_string(),
            hierarchy_order: vec!["year".to_string(), "singer".to_string()],
            filter_expr: Some("payload_kind = 'mp3'".to_string()),
            status: "active".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        catalog.create_view(&view).await.unwrap();

        let views = catalog.list_views(None).await.unwrap();
        assert_eq!(views.len(), 1);
        assert_eq!(views[0].view_name, "test-view");

        catalog.update_view_status("test-view", "inactive").await.unwrap();
        let active_views = catalog.list_views(Some("active")).await.unwrap();
        assert_eq!(active_views.len(), 0);

        let inactive_views = catalog.list_views(Some("inactive")).await.unwrap();
        assert_eq!(inactive_views.len(), 1);

        catalog.delete_view("test-view").await.unwrap();
        let all_views = catalog.list_views(None).await.unwrap();
        assert_eq!(all_views.len(), 0);
    }
}
