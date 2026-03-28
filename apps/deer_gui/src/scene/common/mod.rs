//! Shared scene systems — parallax scrolling and atmosphere transitions.

pub mod atmosphere;
pub mod parallax;

pub use atmosphere::{atmosphere_transition_system, AtmosphereConfig, AtmosphereState};
pub use parallax::{parallax_scroll_system, ParallaxLayer};
