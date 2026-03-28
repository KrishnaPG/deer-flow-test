//! World module — ECS components, state, and systems for the world simulation.
//!
//! Submodules:
//! - [`state`] — `WorldState` / `SystemHealth` resources
//! - [`components`] — ECS components and marker types
//! - [`spatial`] — Grid-based spatial index
//! - [`systems`] — frame-update systems
//! - [`plugin`] — Bevy plugin registration

pub mod components;
mod plugin;
pub mod spatial;
pub mod state;
pub mod systems;

pub use plugin::WorldPlugin;
pub use state::WorldState;
