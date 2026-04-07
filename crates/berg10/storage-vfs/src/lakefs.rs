use reqwest::Client;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum LakeFsError {
    #[error("lakefs request failed: {0}")]
    Request(#[from] reqwest::Error),
    #[error("lakefs API error: {status} {message}")]
    Api { status: u16, message: String },
    #[error("not committed: {0}")]
    NotCommitted(String),
}

#[derive(Clone)]
pub struct LakeFsClient {
    client: Client,
    host: String,
    repo: String,
    branch: String,
    access_key: String,
    secret_key: String,
}

impl LakeFsClient {
    pub fn new(
        host: &str,
        repo: &str,
        branch: &str,
        access_key: &str,
        _secret_key: &str,
    ) -> Self {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .expect("LakeFsClient: reqwest Client must build");
        Self {
            client,
            host: host.to_string(),
            repo: repo.to_string(),
            branch: branch.to_string(),
            access_key: access_key.to_string(),
            secret_key: String::new(),
        }
    }

    fn base_url(&self) -> String {
        format!("{}/api/v1", self.host.trim_end_matches('/'))
    }

    fn auth_header(&self) -> String {
        format!(
            "Basic {}",
            base64::Engine::encode(
                &base64::engine::general_purpose::STANDARD,
                format!("{}:{}", self.access_key, self.secret_key)
            )
        )
    }

    pub async fn upload_object(
        &self,
        path: &str,
        data: Vec<u8>,
    ) -> Result<ObjectStats, LakeFsError> {
        let url = format!(
            "{}/repositories/{}/branches/{}/objects",
            self.base_url(),
            self.repo,
            self.branch
        );

        let digest = sha256_hash(&data);
        let payload = UploadRequest {
            path: path.to_string(),
            physical_address: format!("lakefs://{}/{}/{}", self.repo, self.branch, path),
            checksum: digest.clone(),
            size_bytes: data.len() as i64,
            content_type: "application/octet-stream".to_string(),
        };

        let response = self
            .client
            .post(&url)
            .header("Authorization", self.auth_header())
            .json(&payload)
            .send()
            .await?;

        if response.status().is_success() {
            let stats: ObjectStats = response.json().await.map_err(|e| {
                LakeFsError::Request(e)
            })?;
            Ok(stats)
        } else {
            let status = response.status().as_u16();
            let message = response.text().await.unwrap_or_default();
            Err(LakeFsError::Api { status, message })
        }
    }

    pub async fn commit(
        &self,
        message: &str,
        expected_hash: Option<&str>,
    ) -> Result<CommitResult, LakeFsError> {
        let url = format!(
            "{}/repositories/{}/branches/{}/commits",
            self.base_url(),
            self.repo,
            self.branch
        );

        let payload = CommitRequest {
            message: message.to_string(),
            expected_base_sha: expected_hash.map(String::from),
        };

        let response = self
            .client
            .post(&url)
            .header("Authorization", self.auth_header())
            .json(&payload)
            .send()
            .await?;

        if response.status().is_success() {
            let commit: CommitResult = response.json().await.map_err(LakeFsError::Request)?;
            Ok(commit)
        } else {
            let status = response.status().as_u16();
            let message = response.text().await.unwrap_or_default();
            Err(LakeFsError::Api { status, message })
        }
    }

    pub async fn get_commit_id(&self) -> Result<String, LakeFsError> {
        let url = format!(
            "{}/repositories/{}/branches/{}",
            self.base_url(),
            self.repo,
            self.branch
        );

        let response = self
            .client
            .get(&url)
            .header("Authorization", self.auth_header())
            .send()
            .await?;

        if response.status().is_success() {
            let branch: Branch = response.json().await.map_err(LakeFsError::Request)?;
            Ok(branch.head_commit_id)
        } else {
            let status = response.status().as_u16();
            let message = response.text().await.unwrap_or_default();
            Err(LakeFsError::Api { status, message })
        }
    }

    pub fn repo(&self) -> &str {
        &self.repo
    }

    pub fn branch(&self) -> &str {
        &self.branch
    }

    pub fn physical_location(&self, key: &str, commit_id: &str) -> String {
        format!("lakefs://{}/{}/{}/{}", self.repo, self.branch, commit_id, key)
    }
}

fn sha256_hash(data: &[u8]) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    data.hash(&mut hasher);
    format!("{:x}", hasher.finish())
}

#[derive(Debug, Serialize)]
struct UploadRequest {
    path: String,
    physical_address: String,
    checksum: String,
    size_bytes: i64,
    content_type: String,
}

#[derive(Debug, Deserialize)]
pub struct ObjectStats {
    pub path: String,
    pub checksum: String,
    pub physical_address: String,
    pub size_bytes: i64,
}

#[derive(Debug, Serialize)]
struct CommitRequest {
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    expected_base_sha: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CommitResult {
    pub id: String,
    pub message: String,
    pub commit_id: String,
}

#[derive(Debug, Deserialize)]
struct Branch {
    #[allow(dead_code)]
    name: String,
    head_commit_id: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn physical_location_format() {
        let client = LakeFsClient::new(
            "http://localhost:8000",
            "my-repo",
            "main",
            "access",
            "secret",
        );
        let loc = client.physical_location("abc123", "commit-456");
        assert_eq!(loc, "lakefs://my-repo/main/commit-456/abc123");
    }
}
