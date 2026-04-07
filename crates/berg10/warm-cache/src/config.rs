use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WarmCacheConfig {
    pub base_dir: String,
    pub checkouts_dir: String,
    pub content_dir: String,
}

impl Default for WarmCacheConfig {
    fn default() -> Self {
        Self {
            base_dir: ".berg10".to_string(),
            checkouts_dir: "checkouts".to_string(),
            content_dir: "content".to_string(),
        }
    }
}

impl WarmCacheConfig {
    pub fn with_base_dir(base_dir: &str) -> Self {
        Self {
            base_dir: base_dir.to_string(),
            checkouts_dir: format!("{}/checkouts", base_dir),
            content_dir: format!("{}/content", base_dir),
        }
    }

    pub fn checkout_path(&self, hierarchy_name: &str) -> String {
        format!("{}/{}", self.checkouts_dir, hierarchy_name)
    }

    pub fn content_path(&self, content_hash: &str) -> String {
        let first = &content_hash[0..content_hash.len().min(2)];
        let second = if content_hash.len() > 2 {
            &content_hash[2..content_hash.len().min(4)]
        } else {
            ""
        };
        let rest = if content_hash.len() > 4 {
            &content_hash[4..]
        } else {
            ""
        };

        if second.is_empty() {
            format!("{}/{}.blob", self.content_dir, first)
        } else if rest.is_empty() {
            format!("{}/{}/{}.blob", self.content_dir, first, second)
        } else {
            format!("{}/{}/{}/{}.blob", self.content_dir, first, second, rest)
        }
    }
}
