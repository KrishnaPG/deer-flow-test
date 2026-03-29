//! Scene loader — scans a directory for scene descriptor files and caches them.
//!
//! Supports `.scene.ron` and `.scene.json` files.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use bevy::log::{debug, info, warn};
use bevy::prelude::Resource;

use super::descriptor::SceneDescriptor;

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

/// Errors that can occur during scene loading.
#[derive(Debug, thiserror::Error)]
pub enum LoaderError {
    #[error("IO error reading {path}: {source}")]
    Io {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("Parse error in {path}: {message}")]
    Parse { path: PathBuf, message: String },
}

// ---------------------------------------------------------------------------
// SceneLoader
// ---------------------------------------------------------------------------

/// Loads scene descriptors from disk and caches parsed results.
#[derive(Resource)]
pub struct SceneLoader {
    scenes_dir: PathBuf,
    cache: HashMap<String, SceneDescriptor>,
}

impl SceneLoader {
    /// Create a new loader targeting the given directory.
    pub fn new(scenes_dir: PathBuf) -> Self {
        info!("SceneLoader::new — dir={}", scenes_dir.display());
        Self {
            scenes_dir,
            cache: HashMap::new(),
        }
    }

    /// Scan the scenes directory and parse all descriptor files.
    pub fn scan(&mut self) -> Result<usize, LoaderError> {
        debug!("SceneLoader::scan — scanning {}", self.scenes_dir.display());
        self.cache.clear();

        let entries = std::fs::read_dir(&self.scenes_dir).map_err(|e| LoaderError::Io {
            path: self.scenes_dir.clone(),
            source: e,
        })?;

        for entry in entries.flatten() {
            let path = entry.path();
            if Self::is_scene_file(&path) {
                match self.parse_file(&path) {
                    Ok(desc) => {
                        info!("SceneLoader::scan — loaded '{}'", desc.name);
                        self.cache.insert(desc.name.clone(), desc);
                    }
                    Err(e) => {
                        warn!("SceneLoader::scan — skipping {}: {e}", path.display());
                    }
                }
            }
        }

        let count = self.cache.len();
        info!("SceneLoader::scan — loaded {count} scene(s)");
        Ok(count)
    }

    /// Get a cached descriptor by scene name.
    pub fn get(&self, name: &str) -> Option<&SceneDescriptor> {
        self.cache.get(name)
    }

    /// List all available scene names.
    pub fn available(&self) -> Vec<&str> {
        self.cache.keys().map(|s| s.as_str()).collect()
    }

    // -----------------------------------------------------------------------
    // Private helpers
    // -----------------------------------------------------------------------

    fn is_scene_file(path: &Path) -> bool {
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        name.ends_with(".scene.ron") || name.ends_with(".scene.json")
    }

    fn parse_file(&self, path: &Path) -> Result<SceneDescriptor, LoaderError> {
        let content = std::fs::read_to_string(path).map_err(|e| LoaderError::Io {
            path: path.to_path_buf(),
            source: e,
        })?;

        if path.to_str().unwrap_or("").ends_with(".ron") {
            ron::from_str(&content).map_err(|e| LoaderError::Parse {
                path: path.to_path_buf(),
                message: e.to_string(),
            })
        } else {
            serde_json::from_str(&content).map_err(|e| LoaderError::Parse {
                path: path.to_path_buf(),
                message: e.to_string(),
            })
        }
    }
}
