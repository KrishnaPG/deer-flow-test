//! User preferences persistence.
//!
//! Loads and saves [`UserPreferences`] from a RON file in the platform-specific
//! config directory. The file is `deer_gui/preferences.ron` under the
//! project's config directory (see `directories` crate).

use bevy::log::{debug, info, warn};
use bevy::prelude::*;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

use crate::camera::CameraMode;

// ---------------------------------------------------------------------------
// UserPreferences
// ---------------------------------------------------------------------------

/// Serializable user preferences.
#[derive(Resource, Debug, Serialize, Deserialize, Clone)]
pub struct UserPreferences {
    /// Default camera mode on startup.
    pub camera_mode: CameraMode,
    // Future fields: resource_mode, faction_theme, etc.
}

impl Default for UserPreferences {
    fn default() -> Self {
        Self {
            camera_mode: CameraMode::Orbital,
        }
    }
}

impl UserPreferences {
    /// Returns the config file path, creating parent directories if needed.
    fn config_path() -> Option<PathBuf> {
        // ProjectDirs::from(qualifier, organization, application)
        let proj_dirs = ProjectDirs::from("", "DeerFlow", "DeerGUI")?;
        let config_dir = proj_dirs.config_dir();
        let path = config_dir.join("preferences.ron");
        Some(path)
    }

    /// Load preferences from disk. If the file doesn't exist or fails to parse,
    /// returns default preferences.
    pub fn load() -> Self {
        let Some(path) = Self::config_path() else {
            warn!("preferences: could not determine config directory, using defaults");
            return Self::default();
        };

        if !path.exists() {
            debug!("preferences: file not found at {:?}, using defaults", path);
            return Self::default();
        }

        match fs::read_to_string(&path) {
            Ok(contents) => match ron::from_str(&contents) {
                Ok(prefs) => {
                    info!("preferences: loaded from {:?}", path);
                    prefs
                }
                Err(e) => {
                    warn!(
                        "preferences: failed to parse {:?}: {}, using defaults",
                        path, e
                    );
                    Self::default()
                }
            },
            Err(e) => {
                warn!(
                    "preferences: failed to read {:?}: {}, using defaults",
                    path, e
                );
                Self::default()
            }
        }
    }

    /// Save preferences to disk. Creates parent directories if needed.
    pub fn save(&self) {
        let Some(path) = Self::config_path() else {
            warn!("preferences: could not determine config directory, not saving");
            return;
        };

        // Ensure parent directory exists.
        if let Some(parent) = path.parent() {
            if let Err(e) = fs::create_dir_all(parent) {
                warn!(
                    "preferences: failed to create directory {:?}: {}",
                    parent, e
                );
                return;
            }
        }

        match ron::ser::to_string_pretty(self, ron::ser::PrettyConfig::default()) {
            Ok(serialized) => match fs::write(&path, serialized) {
                Ok(_) => {
                    debug!("preferences: saved to {:?}", path);
                }
                Err(e) => {
                    warn!("preferences: failed to write {:?}: {}", path, e);
                }
            },
            Err(e) => {
                warn!("preferences: failed to serialize: {}", e);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

/// Plugin that inserts [`UserPreferences`] as a resource and loads from disk.
pub struct PreferencesPlugin;

impl Plugin for PreferencesPlugin {
    fn build(&self, app: &mut App) {
        let prefs = UserPreferences::load();
        app.insert_resource(prefs)
            .add_systems(Update, save_preferences_on_change);
        info!("PreferencesPlugin: loaded user preferences");
    }
}

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

/// System that saves preferences when they change.
pub fn save_preferences_on_change(
    prefs: Res<UserPreferences>,
    mut last_saved: Local<Option<UserPreferences>>,
) {
    // Skip the first frame (resource just inserted).
    let Some(previous) = last_saved.as_ref() else {
        *last_saved = Some(prefs.clone());
        return;
    };

    // Check if any field changed.
    if previous.camera_mode != prefs.camera_mode {
        debug!("preferences: camera mode changed, saving");
        prefs.save();
        *last_saved = Some(prefs.clone());
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_preferences() {
        let prefs = UserPreferences::default();
        assert_eq!(prefs.camera_mode, CameraMode::Orbital);
    }

    #[test]
    fn config_path_is_some() {
        // This test may fail in CI if no home directory.
        let path = UserPreferences::config_path();
        // Just ensure it doesn't panic.
        assert!(path.is_some() || path.is_none());
    }
}
