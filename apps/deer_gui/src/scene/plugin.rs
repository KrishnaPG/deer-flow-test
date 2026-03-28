//! [`ScenePlugin`] — registers scene resources, startup, and per-frame systems.

use bevy::log::info;
use bevy::prelude::{App, Plugin, Startup, Update};

use super::common::atmosphere::{atmosphere_transition_system, AtmosphereConfig};
use super::common::parallax::PreviousCameraPosition;
use super::tet::setup::tet_scene_setup_system;
use super::tet::systems::{data_trail_system, tet_glow_system};

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

/// Registers all TET scene systems and resources.
///
/// **Resources:** [`AtmosphereConfig`], [`PreviousCameraPosition`]
/// **Startup:** [`tet_scene_setup_system`]
/// **Update:** glow, data trails, atmosphere transitions
pub struct ScenePlugin;

impl Plugin for ScenePlugin {
    fn build(&self, app: &mut App) {
        info!("ScenePlugin::build — registering scene systems");

        app.init_resource::<AtmosphereConfig>()
            .init_resource::<PreviousCameraPosition>()
            .add_systems(Startup, tet_scene_setup_system)
            .add_systems(
                Update,
                (
                    tet_glow_system,
                    data_trail_system,
                    atmosphere_transition_system,
                ),
            );
    }
}
