use opendal::{Operator, services, layers};
use thiserror::Error;
use tracing::instrument;

use crate::lakefs::LakeFsClient;

#[derive(Error, Debug)]
pub enum StorageError {
    #[error("storage operation failed: {0}")]
    OperationFailed(#[from] opendal::Error),
    #[error("unsupported backend: {0}")]
    UnsupportedBackend(String),
    #[error("missing required config: {0}")]
    MissingConfig(&'static str),
    #[error("lakefs api error: {0}")]
    LakeFsApi(String),
    #[error("content not found: {0}")]
    NotFound(String),
}

#[derive(Debug, Clone)]
pub enum BackendType {
    Fs,
    S3,
    Gcs,
    Azure,
    LakeFs,
    Memory,
}

impl std::str::FromStr for BackendType {
    type Err = StorageError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "fs" | "filesystem" => Ok(BackendType::Fs),
            "s3" => Ok(BackendType::S3),
            "gcs" | "gs" => Ok(BackendType::Gcs),
            "azure" | "azblob" => Ok(BackendType::Azure),
            "lakefs" | "lake_fs" => Ok(BackendType::LakeFs),
            "memory" => Ok(BackendType::Memory),
            other => Err(StorageError::UnsupportedBackend(other.to_string())),
        }
    }
}

#[derive(Debug, Clone)]
pub struct StorageConfig {
    pub backend: BackendType,
    pub root: String,
    pub bucket: Option<String>,
    pub region: Option<String>,
    pub endpoint: Option<String>,
    pub access_key_id: Option<String>,
    pub secret_access_key: Option<String>,
    pub lakefs_host: Option<String>,
    pub lakefs_access_key: Option<String>,
    pub lakefs_secret_key: Option<String>,
    pub lakefs_branch: Option<String>,
    pub lakefs_repo: Option<String>,
}

impl StorageConfig {
    pub fn memory() -> Self {
        Self {
            backend: BackendType::Memory,
            root: "/".to_string(),
            bucket: None,
            region: None,
            endpoint: None,
            access_key_id: None,
            secret_access_key: None,
            lakefs_host: None,
            lakefs_access_key: None,
            lakefs_secret_key: None,
            lakefs_branch: None,
            lakefs_repo: None,
        }
    }

    pub fn local_fs(root: &str) -> Self {
        Self {
            backend: BackendType::Fs,
            root: root.to_string(),
            bucket: None,
            region: None,
            endpoint: None,
            access_key_id: None,
            secret_access_key: None,
            lakefs_host: None,
            lakefs_access_key: None,
            lakefs_secret_key: None,
            lakefs_branch: None,
            lakefs_repo: None,
        }
    }
}

/// Pluggable storage backend wrapping OpenDAL Operator.
pub struct StorageBackend {
    operator: Operator,
    backend_type: BackendType,
    lakefs_client: Option<LakeFsClient>,
}

impl StorageBackend {
    pub fn new(config: &StorageConfig) -> Result<Self, StorageError> {
        let operator = Self::build_operator(config)?;
        let lakefs_client = match &config.backend {
            BackendType::LakeFs => {
                let host = config.lakefs_host.as_deref()
                    .unwrap_or("http://localhost:8000");
                let repo = config.lakefs_repo.as_deref()
                    .ok_or(StorageError::MissingConfig("lakefs_repo"))?;
                let branch = config.lakefs_branch.as_deref()
                    .unwrap_or("main");
                let access_key = config.lakefs_access_key.as_deref()
                    .ok_or(StorageError::MissingConfig("lakefs_access_key"))?;
                let secret_key = config.lakefs_secret_key.as_deref()
                    .ok_or(StorageError::MissingConfig("lakefs_secret_key"))?;
                Some(LakeFsClient::new(host, repo, branch, access_key, secret_key))
            }
            _ => None,
        };

        Ok(Self {
            operator,
            backend_type: config.backend.clone(),
            lakefs_client,
        })
    }

    fn build_operator(config: &StorageConfig) -> Result<Operator, StorageError> {
        let op = match &config.backend {
            BackendType::Fs => {
                let builder = services::Fs::default().root(&config.root);
                Operator::new(builder)?.finish()
            }
            BackendType::S3 => {
                let mut builder = services::S3::default()
                    .bucket(config.bucket.as_deref().unwrap_or("default"))
                    .root(&config.root);
                if let Some(region) = &config.region {
                    builder = builder.region(region);
                }
                if let Some(endpoint) = &config.endpoint {
                    builder = builder.endpoint(endpoint);
                }
                if let Some(ak) = &config.access_key_id {
                    builder = builder.access_key_id(ak);
                }
                if let Some(sk) = &config.secret_access_key {
                    builder = builder.secret_access_key(sk);
                }
                Operator::new(builder)?.finish()
            }
            BackendType::Gcs => {
                let mut builder = services::Gcs::default()
                    .bucket(config.bucket.as_deref().unwrap_or("default"))
                    .root(&config.root);
                if let Some(cred) = &config.access_key_id {
                    builder = builder.credential(cred);
                }
                Operator::new(builder)?.finish()
            }
            BackendType::Azure => {
                let mut builder = services::Azblob::default()
                    .container(config.bucket.as_deref().unwrap_or("default"))
                    .root(&config.root);
                if let Some(account) = &config.access_key_id {
                    builder = builder.account_name(account);
                }
                if let Some(key) = &config.secret_access_key {
                    builder = builder.account_key(key);
                }
                Operator::new(builder)?.finish()
            }
            BackendType::LakeFs => {
                let mut builder = services::S3::default()
                    .bucket(config.lakefs_repo.as_deref().unwrap_or("default"))
                    .root(&config.root)
                    .endpoint(
                        config.lakefs_host
                            .as_deref()
                            .unwrap_or("http://localhost:8000")
                    );
                if let Some(ak) = &config.lakefs_access_key {
                    builder = builder.access_key_id(ak);
                }
                if let Some(sk) = &config.lakefs_secret_key {
                    builder = builder.secret_access_key(sk);
                }
                Operator::new(builder)?.finish()
            }
            BackendType::Memory => {
                let builder = services::Memory::default();
                Operator::new(builder)?.finish()
            }
        };

        let op = op.layer(layers::RetryLayer::new());

        Ok(op)
    }

    /// Write content to storage with the given key.
    #[instrument(skip(self, data), fields(key = key), ret)]
    pub async fn write(&self, key: &str, data: impl Into<Vec<u8>>) -> Result<(), StorageError> {
        match &self.backend_type {
            BackendType::LakeFs => {
                let client = self.lakefs_client.as_ref()
                    .ok_or(StorageError::LakeFsApi("LakeFS client not initialized".into()))?;
                let data = data.into();
                client.upload_object(key, data).await
                    .map_err(|e| StorageError::LakeFsApi(e.to_string()))?;
                Ok(())
            }
            _ => {
                self.operator.write(key, data.into()).await?;
                Ok(())
            }
        }
    }

    /// Read content from storage by key.
    #[instrument(skip(self), fields(key = key))]
    pub async fn read(&self, key: &str) -> Result<Vec<u8>, StorageError> {
        match self.operator.read(key).await {
            Ok(content) => Ok(content.to_vec()),
            Err(e) if e.kind() == opendal::ErrorKind::NotFound => {
                Err(StorageError::NotFound(key.to_string()))
            }
            Err(e) => Err(StorageError::OperationFailed(e)),
        }
    }

    /// Check if content exists in storage.
    pub async fn exists(&self, key: &str) -> Result<bool, StorageError> {
        match self.operator.stat(key).await {
            Ok(_) => Ok(true),
            Err(e) if e.kind() == opendal::ErrorKind::NotFound => Ok(false),
            Err(e) => Err(StorageError::OperationFailed(e)),
        }
    }

    /// Get the backend type.
    pub fn backend_type(&self) -> &BackendType {
        &self.backend_type
    }

    /// Get the raw operator for advanced operations.
    pub fn operator(&self) -> &Operator {
        &self.operator
    }

    /// Commit all staged LakeFS objects and return the commit ID.
    /// No-op for non-LakeFS backends.
    pub async fn commit(&self, message: &str) -> Result<Option<String>, StorageError> {
        match &self.backend_type {
            BackendType::LakeFs => {
                let client = self.lakefs_client.as_ref()
                    .ok_or(StorageError::LakeFsApi("LakeFS client not initialized".into()))?;
                let commit = client.commit(message, None).await
                    .map_err(|e| StorageError::LakeFsApi(e.to_string()))?;
                Ok(Some(commit.commit_id))
            }
            _ => Ok(None),
        }
    }

    /// Build a physical location string for a key using a known commit ID.
    /// Returns None for non-LakeFS backends.
    pub fn physical_location(&self, key: &str, commit_id: &str) -> Option<String> {
        match &self.backend_type {
            BackendType::LakeFs => {
                let client = self.lakefs_client.as_ref()?;
                Some(client.physical_location(key, commit_id))
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn memory_backend_write_and_read() {
        let config = StorageConfig::memory();
        let backend = StorageBackend::new(&config).unwrap();

        backend.write("test-key", b"hello world").await.unwrap();
        let data = backend.read("test-key").await.unwrap();
        assert_eq!(data, b"hello world");
    }

    #[tokio::test]
    async fn memory_backend_exists() {
        let config = StorageConfig::memory();
        let backend = StorageBackend::new(&config).unwrap();

        backend.write("exists-key", b"data").await.unwrap();
        assert!(backend.exists("exists-key").await.unwrap());
        assert!(!backend.exists("missing-key").await.unwrap());
    }

    #[tokio::test]
    async fn memory_backend_read_not_found() {
        let config = StorageConfig::memory();
        let backend = StorageBackend::new(&config).unwrap();

        let result = backend.read("nonexistent").await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), StorageError::NotFound(_)));
    }

    #[test]
    fn backend_type_from_str() {
        assert!(matches!("fs".parse::<BackendType>(), Ok(BackendType::Fs)));
        assert!(matches!("s3".parse::<BackendType>(), Ok(BackendType::S3)));
        assert!(matches!("gcs".parse::<BackendType>(), Ok(BackendType::Gcs)));
        assert!(matches!("azure".parse::<BackendType>(), Ok(BackendType::Azure)));
        assert!(matches!("lakefs".parse::<BackendType>(), Ok(BackendType::LakeFs)));
        assert!(matches!("memory".parse::<BackendType>(), Ok(BackendType::Memory)));
        assert!("unknown".parse::<BackendType>().is_err());
    }
}
