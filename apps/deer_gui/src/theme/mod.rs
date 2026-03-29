//! Theme engine — visual palette management for the Deer GUI.
//!
//! Re-exports the public API so that downstream modules can write
//! `use crate::theme::{ThemePlugin, ThemeManager, Theme};`.

pub mod descriptor;
mod plugin;
pub mod tet_theme;
mod theme;

pub use descriptor::ThemeDescriptor;
pub use plugin::ThemePlugin;
pub use tet_theme::tet_theme;
pub use theme::{Theme, ThemeManager};
