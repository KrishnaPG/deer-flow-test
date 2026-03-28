//! Theme engine — visual palette management for the Deer GUI.
//!
//! Re-exports the public API so that downstream modules can write
//! `use crate::theme::{ThemePlugin, ThemeManager, Theme};`.

mod plugin;
pub mod tet_theme;
mod theme;

pub use plugin::ThemePlugin;
pub use tet_theme::tet_theme;
pub use theme::{Theme, ThemeManager};
