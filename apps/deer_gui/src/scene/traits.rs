//! Scene configuration trait.
//!
//! [`SceneConfig`] defines an extensible interface for scene-specific
//! environment setup. Implement this trait for each scene variant
//! (TET, Precursors, Descent, etc.).

use bevy::asset::Assets;
use bevy::ecs::system::{Commands, ResMut};
use bevy::log::trace;
use bevy::pbr::StandardMaterial;
use bevy::prelude::Mesh;

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
    fn spawn_environment(
        &self,
        commands: &mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
    );

    /// Returns the asset path of the ambient audio track for this scene.
    fn ambient_audio_track(&self) -> &str;

    /// Log scene information for debugging.
    fn log_info(&self) {
        trace!(
            "SceneConfig::log_info — scene='{}' audio='{}'",
            self.name(),
            self.ambient_audio_track()
        );
    }
}
