use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SqlCatalogConfig {
    pub path: String,
}

impl Default for SqlCatalogConfig {
    fn default() -> Self {
        Self {
            path: "catalog/berg10.sqlite".to_string(),
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
}

impl Default for CatalogConfig {
    fn default() -> Self {
        Self {
            backend: CatalogBackendConfig::default(),
        }
    }
}

impl CatalogConfig {
    pub fn with_base_dir(base_dir: &str) -> Self {
        Self {
            backend: CatalogBackendConfig::Sql(SqlCatalogConfig {
                path: format!("{}/catalog/berg10.sqlite", base_dir),
                ..Default::default()
            }),
        }
    }

    pub fn memory() -> Self {
        Self {
            backend: CatalogBackendConfig::Sql(SqlCatalogConfig {
                path: ":memory:".to_string(),
            }),
        }
    }
}
