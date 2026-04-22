//! Theme engine — visual palette management for the Deer GUI.
//!
//! Re-exports the public API so that downstream modules can write
//! `use crate::theme::{ThemePlugin, ThemeManager, Theme};`.
//!
//! Submodules:
//! - [`core`] — Base theme infrastructure
//! - [`faction`] — Faction-based theming with transitions
//! - [`*_theme`] — Preset themes (descent, precursors, tet)

pub mod core;
pub mod descent_theme;
pub mod descriptor;
pub mod faction;
mod plugin;
pub mod precursors_theme;
pub mod tet_theme;

pub use core::{Theme, ThemeManager};
pub use descent_theme::descent_theme;
pub use descriptor::ThemeDescriptor;
pub use faction::{FactionId, FactionTheme, FactionThemeManager, FactionThemePlugin};
pub use plugin::ThemePlugin;
pub use precursors_theme::precursors_theme;
pub use tet_theme::tet_theme;
