//! Scene configuration trait.
//!
//! [`SceneConfig`] defines an extensible interface for scene-specific
//! environment setup. Implement this trait for each scene variant
//! (TET, Precursors, Descent, etc.).

use bevy::asset::Assets;
use bevy::ecs::system::Commands;
use bevy::log::trace;
use bevy::pbr::StandardMaterial;
use bevy::prelude::{Entity, Mesh};

use crate::theme::ThemeManager;

// ---------------------------------------------------------------------------
// SceneConfig trait
// ---------------------------------------------------------------------------

/// Trait for defining scene-specific configuration.
///
/// Each scene variant implements this to describe its environment,
/// lighting, and ambient audio. Designed for extensibility — new
/// scene types (Precursors, Descent) can be added without modifying
/// existing code.
pub trait SceneConfig: Send + Sync + 'static {
    /// Human-readable name of this scene.
    fn name(&self) -> &str;

    /// Spawn the environment entities (terrain, structures, starfield, etc.).
    ///
    /// Returns the root [`Entity`] that parents all scene entities.
    /// The caller can use this to despawn the entire scene hierarchy.
    ///
    /// When `theme` is `Some`, material colours are read from the active
    /// theme; otherwise the implementation should use sensible defaults.
    fn spawn_environment(
        &self,
        commands: &mut Commands,
        meshes: &mut Assets<Mesh>,
        materials: &mut Assets<StandardMaterial>,
        theme: Option<&ThemeManager>,
    ) -> Entity;

    /// Returns the asset path of the ambient audio track for this scene.
    fn ambient_audio_track(&self) -> &'static str;

    /// Called after scene entities are spawned. Default no-op.
    fn on_activate(&self) {
        trace!("SceneConfig::on_activate — scene='{}'", self.name());
    }

    /// Called before scene entities are torn down. Default no-op.
    fn on_deactivate(&self) {
        trace!("SceneConfig::on_deactivate — scene='{}'", self.name());
    }

    /// Log scene information for debugging.
    fn log_info(&self) {
        trace!(
            "SceneConfig::log_info — scene='{}' audio='{}'",
            self.name(),
            self.ambient_audio_track()
        );
    }
}
