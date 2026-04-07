use std::path::Path;

use anyhow::Result;
use chrono::Utc;
use tracing;

use berg10_storage_catalog::{Berg10Catalog, FileRecord, CheckoutInfo, CheckoutReceipt};
use berg10_storage_vfs::StorageBackend;

use crate::config::WarmCacheConfig;

/// Manages local filesystem view materialization as symlink trees.
pub struct WarmCache {
    config: WarmCacheConfig,
    catalog: Berg10Catalog,
    vfs: StorageBackend,
}

impl WarmCache {
    pub fn new(config: WarmCacheConfig, catalog: Berg10Catalog, vfs: StorageBackend) -> Self {
        Self { config, catalog, vfs }
    }

    /// Materialize a view as a symlink tree under base_dir/checkouts/<view_name>/.
    pub async fn checkout(&self, view_name: &str) -> Result<CheckoutReceipt> {
        let files = self.catalog.resolve_view(view_name).await?;
        let views = self.catalog.list_views(Some("active")).await?;
        let view = views.iter().find(|v| v.view_name == view_name);

        let hierarchy_order = match view {
            Some(v) => v.hierarchy_order.clone(),
            None => vec!["hierarchy".to_string(), "level".to_string(), "plane".to_string()],
        };

        let checkout_path = self.config.checkout_path(view_name);
        let checkout_dir = Path::new(&checkout_path);

        // Clean existing checkout
        if checkout_dir.exists() {
            std::fs::remove_dir_all(checkout_dir)?;
        }
        std::fs::create_dir_all(checkout_dir)?;

        // Ensure content dir exists
        std::fs::create_dir_all(&self.config.content_dir)?;

        let mut file_count = 0;

        for file in &files {
            // Ensure content blob is cached locally
            self.ensure_content_cached(file).await?;

            // Build symlink path from hierarchy order
            let symlink_path = Self::build_view_path(file, &hierarchy_order);
            let full_symlink_path = checkout_dir.join(&symlink_path);

            // Create parent directories
            if let Some(parent) = full_symlink_path.parent() {
                std::fs::create_dir_all(parent)?;
            }

            // Create symlink to content blob
            let content_path = self.config.content_path(&file.content_hash);
            let content_path = Path::new(&content_path);

            // Compute relative path from symlink location to content
            let symlink_parent = match full_symlink_path.parent() {
                Some(p) => p,
                None => {
                    tracing::error!(view_name = %view_name, "Symlink path has no parent directory");
                    continue;
                }
            };
            let relative_target = pathdiff::diff_paths(content_path, symlink_parent)
                .unwrap_or_else(|| content_path.to_path_buf());

            // Remove existing symlink if present
            if full_symlink_path.exists() || full_symlink_path.is_symlink() {
                std::fs::remove_file(&full_symlink_path).ok();
            }

            #[cfg(unix)]
            std::os::unix::fs::symlink(&relative_target, &full_symlink_path)?;

            #[cfg(windows)]
            std::os::windows::fs::symlink_file(&relative_target, &full_symlink_path)?;

            file_count += 1;
        }

        tracing::info!(
            view_name = %view_name,
            file_count = file_count,
            checkout_path = %checkout_path,
            "View checkout complete"
        );

        Ok(CheckoutReceipt {
            view_name: view_name.to_string(),
            checkout_path,
            file_count,
            created_at: Utc::now(),
        })
    }

    /// Ensure content blob exists in local cache, fetching from cold storage if needed.
    async fn ensure_content_cached(&self, file: &FileRecord) -> Result<()> {
        let content_path = self.config.content_path(&file.content_hash);
        let path = Path::new(&content_path);

        if path.exists() {
            return Ok(());
        }

        let key = Self::extract_key_from_location(&file.physical_location);

        let data = self.vfs.read(&key).await?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, data)?;
        tracing::debug!(content_hash = %file.content_hash, "Fetched content from cold storage");

        Ok(())
    }

    /// Extract storage key from physical location URI.
    fn extract_key_from_location(location: &str) -> String {
        match location.split_once("://") {
            Some(("s3" | "s3a" | "s3n", rest)) => {
                let parts: Vec<&str> = rest.splitn(2, '/').collect();
                if parts.len() == 2 {
                    parts[1].to_string()
                } else {
                    rest.to_string()
                }
            }
            Some(("file", rest)) => rest.to_string(),
            Some(("lakefs", rest)) => {
                let parts: Vec<&str> = rest.splitn(3, '/').collect();
                if parts.len() >= 3 {
                    format!("{}/{}", parts[1], parts[2])
                } else {
                    rest.to_string()
                }
            }
            Some((_scheme, rest)) => rest.to_string(),
            None => location.to_string(),
        }
    }

    /// Build the symlink path within the checkout directory based on hierarchy order.
    pub fn build_view_path(file: &FileRecord, hierarchy_order: &[String]) -> String {
        let mut parts = Vec::new();

        for attr in hierarchy_order {
            let value = match attr.as_str() {
                "hierarchy" => file.hierarchy.clone(),
                "level" => file.level.clone(),
                "plane" => file.plane.clone(),
                "payload_kind" => file.payload_kind.clone(),
                "payload_format" => file.payload_format.clone(),
                "writer_identity" => file.writer_identity.clone().unwrap_or_default(),
                _ => {
                    // Look up in routing_tags
                    file.routing_tags.iter()
                        .find(|(k, _)| k == attr)
                        .map(|(_, v)| v.clone())
                        .unwrap_or_else(|| "unknown".to_string())
                }
            };

            if !value.is_empty() {
                parts.push(sanitize_segment(&value));
            }
        }

        // Add filename
        parts.push(format!("{}.{}", file.content_hash, file.payload_format));

        parts.join("/")
    }

    /// List all active checkouts.
    pub async fn list_checkouts(&self) -> Result<Vec<CheckoutInfo>> {
        let views = self.catalog.list_views(Some("active")).await?;
        let mut infos = Vec::new();

        for view in &views {
            let checkout_path = self.config.checkout_path(&view.view_name);
            let file_count = count_files_in_dir(&checkout_path).unwrap_or(0);

            infos.push(CheckoutInfo {
                view_name: view.view_name.clone(),
                checkout_path,
                file_count,
                status: view.status.clone(),
            });
        }

        Ok(infos)
    }

    /// Deactivate a checkout: remove symlink tree and update view status.
    pub async fn deactivate_checkout(&self, view_name: &str) -> Result<()> {
        let checkout_path = self.config.checkout_path(view_name);
        if Path::new(&checkout_path).exists() {
            std::fs::remove_dir_all(&checkout_path)?;
        }

        self.catalog.update_view_status(view_name, "inactive").await?;
        tracing::info!(view_name = %view_name, "Checkout deactivated");
        Ok(())
    }

    /// Remove a checkout: delete symlink tree and remove view definition.
    pub async fn remove_checkout(&self, view_name: &str) -> Result<()> {
        self.deactivate_checkout(view_name).await?;
        self.catalog.delete_view(view_name).await?;
        tracing::info!(view_name = %view_name, "Checkout removed");
        Ok(())
    }
}

fn sanitize_segment(value: &str) -> String {
    let mut sanitized = String::new();
    let mut last_was_dash = false;

    for ch in value.chars() {
        if ch.is_ascii_alphanumeric() || ch == '_' {
            sanitized.push(ch.to_ascii_lowercase());
            last_was_dash = false;
        } else if !last_was_dash {
            sanitized.push('-');
            last_was_dash = true;
        }
    }

    sanitized.trim_matches('-').to_owned()
}

fn count_files_in_dir(dir: &str) -> Result<usize> {
    let path = Path::new(dir);
    if !path.exists() {
        return Ok(0);
    }

    let mut count = 0;
    for entry in std::fs::read_dir(path)? {
        let entry = entry?;
        if entry.file_type()?.is_symlink() || entry.file_type()?.is_file() {
            count += 1;
        } else if entry.file_type()?.is_dir() {
            count += count_files_in_dir(&entry.path().to_string_lossy())?;
        }
    }

    Ok(count)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitize_segment_cleans_input() {
        assert_eq!(sanitize_segment("Hello World!"), "hello-world");
        assert_eq!(sanitize_segment("path/with/slashes"), "path-with-slashes");
        assert_eq!(sanitize_segment("UPPER_case"), "upper_case");
    }

    #[test]
    fn extract_key_from_location_handles_schemes() {
        assert_eq!(WarmCache::extract_key_from_location("s3://berg10-storage/abc123"), "abc123");
        assert_eq!(WarmCache::extract_key_from_location("file:///data/abc123"), "/data/abc123");
        assert_eq!(WarmCache::extract_key_from_location("lakefs://repo/main/abc123"), "main/abc123");
        assert_eq!(WarmCache::extract_key_from_location("s3://bucket/nested/path/file.dat"), "nested/path/file.dat");
    }

    #[test]
    fn config_paths_are_correct() {
        let config = WarmCacheConfig::with_base_dir("/base");
        assert_eq!(config.checkout_path("my-view"), "/base/checkouts/my-view");
        assert_eq!(config.content_path("abc123"), "/base/content/abc123");
    }

    #[test]
    fn build_view_path_with_hierarchy() {
        let file = FileRecord {
            content_hash: "abc123".to_string(),
            hierarchy: "A".to_string(),
            level: "L0".to_string(),
            plane: "as-is".to_string(),
            payload_kind: "mp3".to_string(),
            payload_format: "mp3".to_string(),
            payload_size_bytes: 1000,
            physical_location: "s3://test/abc123".to_string(),
            correlation_ids: vec![],
            lineage_refs: vec![],
            routing_tags: vec![("year".to_string(), "2024".to_string()), ("singer".to_string(), "Adele".to_string())],
            written_at: Utc::now(),
            writer_identity: Some("test".to_string()),
        };

        // Test year-first hierarchy
        let path = WarmCache::build_view_path(&file, &["year".to_string(), "singer".to_string()]);
        assert_eq!(path, "2024/adele/abc123.mp3");

        // Test singer-first hierarchy
        let path = WarmCache::build_view_path(&file, &["singer".to_string(), "year".to_string()]);
        assert_eq!(path, "adele/2024/abc123.mp3");

        // Test with standard hierarchy
        let path = WarmCache::build_view_path(&file, &["hierarchy".to_string(), "level".to_string(), "plane".to_string()]);
        assert_eq!(path, "a/l0/as-is/abc123.mp3");
    }
}
