use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SqlCatalogConfig {
    pub database: String,
    pub path: String,
}

impl Default for SqlCatalogConfig {
    fn default() -> Self {
        Self {
            database: "sqlite".to_string(),
            path: "catalog/iceberg.sqlite".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestCatalogConfig {
    pub uri: String,
    pub token: Option<String>,
    pub warehouse: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CatalogBackendConfig {
    Sql(SqlCatalogConfig),
    Rest(RestCatalogConfig),
}

impl Default for CatalogBackendConfig {
    fn default() -> Self {
        CatalogBackendConfig::Sql(SqlCatalogConfig::default())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CatalogConfig {
    pub backend: CatalogBackendConfig,
    pub warehouse_path: String,
}

impl Default for CatalogConfig {
    fn default() -> Self {
        Self {
            backend: CatalogBackendConfig::default(),
            warehouse_path: "warehouse".to_string(),
        }
    }
}

impl CatalogConfig {
    pub fn with_base_dir(base_dir: &str) -> Self {
        Self {
            backend: CatalogBackendConfig::Sql(SqlCatalogConfig {
                path: format!("{}/catalog/iceberg.sqlite", base_dir),
                ..Default::default()
            }),
            warehouse_path: format!("{}/warehouse", base_dir),
        }
    }

    pub fn memory() -> Self {
        Self {
            backend: CatalogBackendConfig::Sql(SqlCatalogConfig::default()),
            warehouse_path: "memory://warehouse".to_string(),
        }
    }
}
