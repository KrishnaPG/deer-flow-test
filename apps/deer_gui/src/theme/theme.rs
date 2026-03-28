//! Core theme types and the [`ThemeManager`] resource.
//!
//! A [`Theme`] describes a complete visual palette. The [`ThemeManager`]
//! stores all registered themes, tracks the active one, and exposes a
//! generation counter so downstream systems can react to switches cheaply.

use bevy::log::{debug, info, trace};
use bevy::prelude::*;

// ---------------------------------------------------------------------------
// Theme definition
// ---------------------------------------------------------------------------

/// A complete visual theme definition.
///
/// Each field maps to a semantic role so that UI code never hard-codes
/// colours — it always resolves them through the current theme.
#[derive(Clone, Debug)]
pub struct Theme {
    /// Human-readable theme name (used as lookup key).
    pub name: String,

    /// Primary background colour (panels, frames).
    pub background: Color,
    /// Secondary background (cards, insets).
    pub surface: Color,

    /// Primary accent colour (active elements, highlights).
    pub accent: Color,
    /// Secondary accent.
    pub accent_secondary: Color,

    /// Primary text colour.
    pub text_primary: Color,
    /// Secondary / dimmed text colour.
    pub text_secondary: Color,

    /// Success indicator colour.
    pub success: Color,
    /// Warning indicator colour.
    pub warning: Color,
    /// Error / danger indicator colour.
    pub error: Color,

    /// Panel background alpha (0.0–1.0).
    pub panel_alpha: f32,
    /// Panel corner rounding (pixels).
    pub panel_rounding: f32,
}

// ---------------------------------------------------------------------------
// ThemeManager resource
// ---------------------------------------------------------------------------

/// Resource managing registered themes and the currently active one.
///
/// The `generation` counter is bumped on every theme switch so that
/// downstream systems can detect changes with a simple integer compare
/// (no events, no change-detection boilerplate).
#[derive(Resource)]
pub struct ThemeManager {
    themes: Vec<Theme>,
    active_index: usize,
    /// Incremented on every [`Self::switch`] call that actually changes the theme.
    pub generation: u64,
}

impl ThemeManager {
    /// Creates a new manager seeded with the given themes.
    ///
    /// # Panics
    /// Panics if `themes` is empty — at least one theme is required.
    pub fn new(themes: Vec<Theme>) -> Self {
        assert!(
            !themes.is_empty(),
            "ThemeManager requires at least one theme"
        );
        info!(
            "ThemeManager::new — registered {} theme(s), active='{}'",
            themes.len(),
            themes[0].name,
        );
        Self {
            themes,
            active_index: 0,
            generation: 1, // start at 1 so first apply always triggers
        }
    }

    /// Returns a reference to the currently active theme.
    #[inline]
    pub fn current(&self) -> &Theme {
        trace!(
            "ThemeManager::current — '{}'",
            self.themes[self.active_index].name
        );
        &self.themes[self.active_index]
    }

    /// Switches to the theme identified by `name`.
    ///
    /// Returns `true` if a matching theme was found and activated,
    /// `false` otherwise (active theme remains unchanged).
    pub fn switch(&mut self, name: &str) -> bool {
        debug!("ThemeManager::switch — request for '{name}'");
        if let Some(idx) = self.themes.iter().position(|t| t.name == name) {
            if idx != self.active_index {
                self.active_index = idx;
                self.generation = self.generation.wrapping_add(1);
                info!(
                    "ThemeManager::switch — activated '{}' (gen={})",
                    name, self.generation,
                );
            } else {
                debug!("ThemeManager::switch — '{name}' already active, no-op");
            }
            true
        } else {
            debug!("ThemeManager::switch — '{name}' not found");
            false
        }
    }

    /// Returns the name of the currently active theme.
    #[inline]
    pub fn active_name(&self) -> &str {
        &self.themes[self.active_index].name
    }

    /// Returns the names of all registered themes.
    pub fn available_themes(&self) -> Vec<&str> {
        self.themes.iter().map(|t| t.name.as_str()).collect()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn dummy_theme(name: &str) -> Theme {
        Theme {
            name: name.to_string(),
            background: Color::BLACK,
            surface: Color::BLACK,
            accent: Color::WHITE,
            accent_secondary: Color::WHITE,
            text_primary: Color::WHITE,
            text_secondary: Color::WHITE,
            success: Color::WHITE,
            warning: Color::WHITE,
            error: Color::WHITE,
            panel_alpha: 0.75,
            panel_rounding: 8.0,
        }
    }

    #[test]
    fn new_sets_first_theme_active() {
        let mgr = ThemeManager::new(vec![dummy_theme("A"), dummy_theme("B")]);
        assert_eq!(mgr.active_name(), "A");
    }

    #[test]
    fn switch_changes_generation() {
        let mut mgr = ThemeManager::new(vec![dummy_theme("A"), dummy_theme("B")]);
        let gen_before = mgr.generation;
        assert!(mgr.switch("B"));
        assert_eq!(mgr.active_name(), "B");
        assert!(mgr.generation > gen_before);
    }

    #[test]
    fn switch_same_is_noop() {
        let mut mgr = ThemeManager::new(vec![dummy_theme("A")]);
        let gen_before = mgr.generation;
        assert!(mgr.switch("A"));
        assert_eq!(mgr.generation, gen_before);
    }

    #[test]
    fn switch_unknown_returns_false() {
        let mut mgr = ThemeManager::new(vec![dummy_theme("A")]);
        assert!(!mgr.switch("nope"));
    }

    #[test]
    fn available_themes_returns_all() {
        let mgr = ThemeManager::new(vec![dummy_theme("X"), dummy_theme("Y"), dummy_theme("Z")]);
        assert_eq!(mgr.available_themes(), vec!["X", "Y", "Z"]);
    }
}
