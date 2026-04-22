//! Faction theme system for dynamic UI styling.
//!
//! Provides a reusable [`FactionThemePlugin`] that manages faction-based
//! theming with smooth transitions between factions.
//!
//! ## Features
//!
//! - Multiple faction themes (English, French, Byzantine, Mongol)
//! - Smooth color transitions with purple midpoint
//! - Border style morphing
//! - Heraldry crossfade effects
//! - Faction selector UI
//!
//! ## Reusability
//!
//! This plugin can be extracted to a standalone crate for use in any
//! Bevy project needing faction-based theming. Configuration is done
//! via Bevy resources (data-driven, no hardcoded paths).

use bevy::ecs::system::{Res, ResMut};
use bevy::log::{info, trace, warn};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Faction Identifiers
// ---------------------------------------------------------------------------

/// Unique faction identifiers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FactionId {
    /// English faction (red/white rose motif).
    English,
    /// French faction (fleur-de-lis, blue/gold).
    French,
    /// Byzantine faction (double-headed eagle, purple/gold).
    Byzantine,
    /// Mongol faction (wolf/sun motif, red/blue).
    Mongol,
    /// Neutral/default faction.
    Neutral,
}

impl Default for FactionId {
    fn default() -> Self {
        Self::Neutral
    }
}

impl FactionId {
    /// Get all playable factions.
    pub fn playable() -> Vec<FactionId> {
        vec![Self::English, Self::French, Self::Byzantine, Self::Mongol]
    }
}

// ---------------------------------------------------------------------------
// Border Styles
// ---------------------------------------------------------------------------

/// Border decoration styles for UI elements.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BorderStyle {
    /// Simple solid border.
    Solid,
    /// Ornate medieval border.
    Ornate,
    /// Double-line border.
    Double,
    /// Dotted/dashed border.
    Dashed,
    /// No border.
    None,
}

impl Default for BorderStyle {
    fn default() -> Self {
        Self::Solid
    }
}

// ---------------------------------------------------------------------------
// Faction Theme Definition
// ---------------------------------------------------------------------------

/// Theme configuration for a specific faction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactionTheme {
    /// Faction identifier.
    pub id: FactionId,
    /// Primary color (main UI elements).
    pub primary: (f32, f32, f32),
    /// Secondary color (accents, buttons).
    pub secondary: (f32, f32, f32),
    /// Heraldic color (banners, shields).
    pub heraldic: (f32, f32, f32),
    /// Background color.
    pub background: (f32, f32, f32),
    /// Text color.
    pub text: (f32, f32, f32),
    /// Border decoration style.
    pub border_style: BorderStyle,
    /// Symbol/icon identifier.
    pub symbol: String,
    /// Description for UI display.
    pub description: String,
}

impl Default for FactionTheme {
    fn default() -> Self {
        Self::neutral()
    }
}

impl FactionTheme {
    /// Create English faction theme.
    pub fn english() -> Self {
        Self {
            id: FactionId::English,
            primary: (0.7, 0.1, 0.1),      // Deep red
            secondary: (0.9, 0.9, 0.95),   // White/off-white
            heraldic: (0.8, 0.6, 0.2),     // Gold
            background: (0.15, 0.12, 0.1), // Dark brown
            text: (0.95, 0.95, 0.9),       // Warm white
            border_style: BorderStyle::Ornate,
            symbol: "symbols/english_rose.svg".to_string(),
            description: "Kingdom of England - Red Rose".to_string(),
        }
    }

    /// Create French faction theme.
    pub fn french() -> Self {
        Self {
            id: FactionId::French,
            primary: (0.1, 0.2, 0.6),     // Royal blue
            secondary: (0.9, 0.85, 0.5),  // Gold
            heraldic: (0.95, 0.9, 0.8),   // Cream
            background: (0.1, 0.12, 0.2), // Dark blue
            text: (0.95, 0.95, 0.95),     // White
            border_style: BorderStyle::Double,
            symbol: "symbols/french_fleur.svg".to_string(),
            description: "Kingdom of France - Fleur-de-lis".to_string(),
        }
    }

    /// Create Byzantine faction theme.
    pub fn byzantine() -> Self {
        Self {
            id: FactionId::Byzantine,
            primary: (0.4, 0.15, 0.5),      // Purple
            secondary: (0.85, 0.75, 0.2),   // Gold
            heraldic: (0.6, 0.2, 0.3),      // Deep red
            background: (0.12, 0.08, 0.15), // Dark purple
            text: (0.95, 0.9, 0.85),        // Warm white
            border_style: BorderStyle::Ornate,
            symbol: "symbols/byzantine_eagle.svg".to_string(),
            description: "Byzantine Empire - Double Eagle".to_string(),
        }
    }

    /// Create Mongol faction theme.
    pub fn mongol() -> Self {
        Self {
            id: FactionId::Mongol,
            primary: (0.6, 0.2, 0.15),     // Red-brown
            secondary: (0.2, 0.4, 0.6),    // Sky blue
            heraldic: (0.85, 0.75, 0.4),   // Gold
            background: (0.15, 0.12, 0.1), // Earth brown
            text: (0.95, 0.9, 0.8),        // Warm white
            border_style: BorderStyle::Dashed,
            symbol: "symbols/mongol_sky.svg".to_string(),
            description: "Mongol Empire - Sky Wolf".to_string(),
        }
    }

    /// Create neutral/default theme.
    pub fn neutral() -> Self {
        Self {
            id: FactionId::Neutral,
            primary: (0.4, 0.4, 0.45),    // Grey
            secondary: (0.6, 0.6, 0.65),  // Light grey
            heraldic: (0.5, 0.5, 0.55),   // Mid grey
            background: (0.2, 0.2, 0.22), // Dark grey
            text: (0.9, 0.9, 0.9),        // Off-white
            border_style: BorderStyle::Solid,
            symbol: "symbols/neutral.svg".to_string(),
            description: "Neutral Faction".to_string(),
        }
    }

    /// Get a Bevy Color from the RGB tuple.
    pub fn primary_color(&self) -> Color {
        Color::srgb(self.primary.0, self.primary.1, self.primary.2)
    }

    /// Get a Bevy Color from the RGB tuple.
    pub fn secondary_color(&self) -> Color {
        Color::srgb(self.secondary.0, self.secondary.1, self.secondary.2)
    }
}

// ---------------------------------------------------------------------------
// Transition System
// ---------------------------------------------------------------------------

/// Configuration for faction transition animations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransitionConfig {
    /// Duration of transition in seconds.
    #[serde(default = "default_transition_duration")]
    pub duration: f32,
    /// Use purple midpoint in color interpolation.
    #[serde(default = "default_use_purple")]
    pub use_purple_midpoint: bool,
    /// Easing function type.
    #[serde(default)]
    pub easing: EasingType,
}

fn default_transition_duration() -> f32 {
    3.0
}

fn default_use_purple() -> bool {
    true
}

impl Default for TransitionConfig {
    fn default() -> Self {
        Self {
            duration: default_transition_duration(),
            use_purple_midpoint: default_use_purple(),
            easing: EasingType::default(),
        }
    }
}

/// Easing function types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EasingType {
    /// Linear interpolation.
    Linear,
    /// Ease in-out cubic.
    EaseInOutCubic,
    /// Ease out elastic.
    EaseOutElastic,
}

impl Default for EasingType {
    fn default() -> Self {
        Self::EaseInOutCubic
    }
}

// ---------------------------------------------------------------------------
// Faction Theme Manager
// ---------------------------------------------------------------------------

/// Resource managing current faction theme and transitions.
#[derive(Resource, Debug)]
pub struct FactionThemeManager {
    /// Current faction theme.
    pub current: FactionTheme,
    /// Target faction theme (when transitioning).
    pub target: Option<FactionTheme>,
    /// Transition progress (0.0 to 1.0).
    pub transition_progress: f32,
    /// Transition configuration.
    pub transition_config: TransitionConfig,
    /// All registered faction themes.
    pub themes: std::collections::HashMap<FactionId, FactionTheme>,
    /// Current interpolated border style (morphed during transition).
    pub current_border_style: BorderStyle,
    /// Current heraldry opacity (0.0 = old faction, 1.0 = new faction).
    pub heraldry_opacity: f32,
}

impl Default for FactionThemeManager {
    fn default() -> Self {
        let mut themes = std::collections::HashMap::new();
        themes.insert(FactionId::Neutral, FactionTheme::neutral());
        themes.insert(FactionId::English, FactionTheme::english());
        themes.insert(FactionId::French, FactionTheme::french());
        themes.insert(FactionId::Byzantine, FactionTheme::byzantine());
        themes.insert(FactionId::Mongol, FactionTheme::mongol());

        Self {
            current: FactionTheme::neutral(),
            target: None,
            transition_progress: 1.0,
            transition_config: TransitionConfig::default(),
            themes,
            current_border_style: BorderStyle::Solid,
            heraldry_opacity: 1.0,
        }
    }
}

impl FactionThemeManager {
    /// Start a transition to a new faction theme.
    pub fn transition_to(&mut self, faction_id: FactionId) {
        if let Some(theme) = self.themes.get(&faction_id) {
            if self.current.id != faction_id {
                info!("FactionThemeManager: transitioning to {:?}", faction_id);
                self.target = Some(theme.clone());
                self.transition_progress = 0.0;
            }
        } else {
            warn!("FactionThemeManager: faction {:?} not found", faction_id);
        }
    }

    /// Update transition progress (called by system).
    pub fn update_transition(&mut self, delta: f32) {
        if self.target.is_some() && self.transition_progress < 1.0 {
            self.transition_progress += delta / self.transition_config.duration;
            self.transition_progress = self.transition_progress.min(1.0);

            // Update border style morphing (switch at midpoint)
            if let Some(target) = &self.target {
                if self.transition_progress >= 0.5 {
                    self.current_border_style = target.border_style;
                } else {
                    self.current_border_style = self.current.border_style;
                }
            }

            // Update heraldry opacity (crossfade)
            self.heraldry_opacity = self.transition_progress;

            if self.transition_progress >= 1.0 {
                if let Some(target) = self.target.take() {
                    self.current = target;
                    self.current_border_style = self.current.border_style;
                    self.heraldry_opacity = 1.0;
                }
            }
        }
    }

    /// Get the interpolated color during transition.
    pub fn interpolated_primary(&self) -> Color {
        if let Some(target) = &self.target {
            let t = ease(self.transition_progress, self.transition_config.easing);
            lerp_color(
                &self.current.primary_color(),
                &target.primary_color(),
                t,
                self.transition_config.use_purple_midpoint,
            )
        } else {
            self.current.primary_color()
        }
    }

    /// Get the current border style (interpolated during transition).
    pub fn current_border_style(&self) -> BorderStyle {
        self.current_border_style
    }

    /// Get the heraldry crossfade opacity (0.0 = old, 1.0 = new).
    pub fn heraldry_opacity(&self) -> f32 {
        self.heraldry_opacity
    }

    /// Get a faction theme by ID.
    pub fn get_theme(&self, faction_id: FactionId) -> Option<&FactionTheme> {
        self.themes.get(&faction_id)
    }
}

// ---------------------------------------------------------------------------
// Helper Functions
// ---------------------------------------------------------------------------

/// Apply easing function to a value.
fn ease(t: f32, easing: EasingType) -> f32 {
    match easing {
        EasingType::Linear => t,
        EasingType::EaseInOutCubic => {
            if t < 0.5 {
                4.0 * t * t * t
            } else {
                1.0 - (-2.0 * t + 2.0).powi(3) / 2.0
            }
        }
        EasingType::EaseOutElastic => {
            if t == 0.0 || t == 1.0 {
                t
            } else {
                let c4 = (2.0 * std::f32::consts::PI) / 3.0;
                2.0_f32.powf(-10.0 * t) * ((t * 10.0 - 0.75) * c4).sin() + 1.0
            }
        }
    }
}

/// Interpolate between two colors with optional purple midpoint.
fn lerp_color(a: &Color, b: &Color, t: f32, use_purple: bool) -> Color {
    let a_rgba = a.to_srgba();
    let b_rgba = b.to_srgba();

    let (r, g, bl) = if use_purple && t > 0.3 && t < 0.7 {
        // Add purple tint during mid-transition
        let purple_t = ((t - 0.3) / 0.4 - 0.5).abs() * 2.0; // 1.0 at midpoint, 0.0 at edges
        let purple_strength = 0.3 * (1.0 - purple_t);
        (
            a_rgba.red * (1.0 - t) + b_rgba.red * t + purple_strength * 0.5,
            a_rgba.green * (1.0 - t) + b_rgba.green * t - purple_strength * 0.2,
            a_rgba.blue * (1.0 - t) + b_rgba.blue * t + purple_strength * 0.3,
        )
    } else {
        (
            a_rgba.red * (1.0 - t) + b_rgba.red * t,
            a_rgba.green * (1.0 - t) + b_rgba.green * t,
            a_rgba.blue * (1.0 - t) + b_rgba.blue * t,
        )
    };

    Color::srgba(r, g, bl, 1.0)
}

// ---------------------------------------------------------------------------
// Faction Theme Plugin
// ---------------------------------------------------------------------------

/// Modular faction theme plugin for dynamic UI styling.
///
/// This plugin can be added to any Bevy app for faction-based theming.
/// Configuration is done via the `FactionThemeManager` resource.
///
/// # Usage
///
/// ```rust,ignore
/// app.add_plugins(FactionThemePlugin::default());
/// ```
#[derive(Default)]
pub struct FactionThemePlugin;

impl Plugin for FactionThemePlugin {
    fn build(&self, app: &mut App) {
        info!("FactionThemePlugin: initializing faction theme system");

        // Add theme manager resource
        app.init_resource::<FactionThemeManager>();

        // Add systems
        app.add_systems(Startup, setup_faction_system);
        app.add_systems(Update, update_faction_transition_system);

        info!("FactionThemePlugin: registered systems");
    }
}

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

/// Setup the faction theme system.
fn setup_faction_system(manager: Res<FactionThemeManager>) {
    trace!("setup_faction_system: initializing");

    info!(
        "FactionThemePlugin: initialized with {} faction themes",
        manager.themes.len()
    );

    for (id, theme) in &manager.themes {
        trace!("  - Faction: {:?} ({})", id, theme.description);
    }
}

/// Update faction transition animations.
fn update_faction_transition_system(time: Res<Time>, mut manager: ResMut<FactionThemeManager>) {
    manager.update_transition(time.delta_secs());
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn faction_id_default() {
        let faction = FactionId::default();
        assert_eq!(faction, FactionId::Neutral);
    }

    #[test]
    fn faction_id_playable() {
        let playable = FactionId::playable();
        assert_eq!(playable.len(), 4);
        assert!(playable.contains(&FactionId::English));
        assert!(!playable.contains(&FactionId::Neutral));
    }

    #[test]
    fn border_style_default() {
        let style = BorderStyle::default();
        assert_eq!(style, BorderStyle::Solid);
    }

    #[test]
    fn faction_theme_default() {
        let theme = FactionTheme::default();
        assert_eq!(theme.id, FactionId::Neutral);
    }

    #[test]
    fn faction_theme_colors() {
        let english = FactionTheme::english();
        let color = english.primary_color();
        // Just verify it doesn't panic
        let _ = color;
    }

    #[test]
    fn transition_config_default() {
        let config = TransitionConfig::default();
        assert!(config.duration > 0.0);
        assert!(config.use_purple_midpoint);
    }

    #[test]
    fn faction_theme_manager_default() {
        let manager = FactionThemeManager::default();
        assert_eq!(manager.themes.len(), 5);
        assert_eq!(manager.transition_progress, 1.0);
    }

    #[test]
    fn faction_theme_serialization() {
        use serde_json;

        let theme = FactionTheme::english();
        let json = serde_json::to_string(&theme).unwrap();
        let deserialized: FactionTheme = serde_json::from_str(&json).unwrap();

        assert_eq!(theme.id, deserialized.id);
        assert_eq!(theme.primary, deserialized.primary);
        assert_eq!(theme.border_style, deserialized.border_style);
    }

    #[test]
    fn faction_theme_manager_transition() {
        let mut manager = FactionThemeManager::default();
        manager.transition_to(FactionId::English);
        assert!(manager.target.is_some());
        assert_eq!(manager.transition_progress, 0.0);
    }

    #[test]
    fn border_style_morphing() {
        let mut manager = FactionThemeManager::default();
        manager.transition_to(FactionId::French);

        // Initial state should be Neutral's border style
        assert_eq!(manager.current_border_style(), BorderStyle::Solid);

        // Simulate transition progress
        manager.update_transition(1.5); // Half of 3.0s duration

        // At 50% progress, should still be current style (neutral solid)
        // because transition hasn't passed 0.5 threshold yet
        assert_eq!(manager.transition_progress, 0.5);
        assert_eq!(manager.current_border_style(), BorderStyle::Solid);

        // Advance past midpoint
        manager.update_transition(0.1);
        assert!(manager.transition_progress > 0.5);
        assert_eq!(manager.current_border_style(), BorderStyle::Double); // French style
    }

    #[test]
    fn heraldry_crossfade() {
        let mut manager = FactionThemeManager::default();
        manager.transition_to(FactionId::English);

        // Initial heraldry opacity
        assert_eq!(manager.heraldry_opacity(), 0.0);

        // Simulate transition progress
        manager.update_transition(1.5);
        assert_eq!(manager.heraldry_opacity(), 0.5);

        // Complete transition
        manager.update_transition(3.0);
        assert_eq!(manager.heraldry_opacity(), 1.0);
    }

    #[test]
    fn ease_linear() {
        assert_eq!(ease(0.0, EasingType::Linear), 0.0);
        assert_eq!(ease(0.5, EasingType::Linear), 0.5);
        assert_eq!(ease(1.0, EasingType::Linear), 1.0);
    }
}
