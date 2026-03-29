//! CSS font resolution pipeline for Google Fonts.
//!
//! This module provides functionality to fetch and parse Google Fonts CSS,
//! download font files, and build egui::FontDefinitions for custom typography.
//! Supports IBM Plex Sans (body) and JetBrains Mono (monospace).
//!
//! # Module Structure
//!
//! - `types` — Error types, FontStyle, FontFaceRule, ResolvedFont
//! - `parse` — CSS parsing functions
//! - `fetch` — HTTP fetching functions  
//! - `build` — Building egui FontDefinitions

mod build;
mod fetch;
mod parse;
mod types;

// Re-export all public items
pub use build::build_font_definitions;
pub use fetch::{fetch_font_bytes, fetch_font_css, is_url_reachable};
pub use parse::parse_font_css;
pub use types::{FontFaceRule, FontLoadError, FontStyle, ResolvedFont};

use crate::constants::fonts::{DEFAULT_BODY_FONT_CSS, DEFAULT_MONO_FONT_CSS};
use bevy::log::{info, warn};

/// Load fonts from a Google Fonts CSS URL.
///
/// This is the main entry point for loading custom fonts. It fetches the CSS,
/// parses the @font-face rules, downloads each font file, and returns the
/// resolved fonts ready for building `FontDefinitions`.
///
/// # Arguments
///
/// * `url` - A Google Fonts CSS URL.
///
/// # Returns
///
/// A `Result` containing a vector of `ResolvedFont` or a `FontLoadError`.
///
/// # Errors
///
/// - `FontLoadError::Http` if fetching CSS or font files fails.
/// - `FontLoadError::NoCssRules` if no valid @font-face rules are found.
///
/// # Example
///
/// ```no_run
/// use deer_gui::ui::fonts::{load_fonts_from_css_url, build_font_definitions};
/// use deer_gui::constants::fonts::DEFAULT_BODY_FONT_CSS;
///
/// // Load fonts from Google Fonts
/// let fonts = load_fonts_from_css_url(DEFAULT_BODY_FONT_CSS).expect("Failed to load fonts");
///
/// // Build egui font definitions
/// let font_defs = build_font_definitions(&fonts);
/// ```
pub fn load_fonts_from_css_url(url: &str) -> Result<Vec<ResolvedFont>, FontLoadError> {
    info!("Loading fonts from CSS URL: {}", url);

    // Fetch and parse CSS
    let css = fetch_font_css(url)?;
    let rules = parse_font_css(&css);

    if rules.is_empty() {
        warn!("No @font-face rules found in CSS from: {}", url);
        return Err(FontLoadError::NoCssRules);
    }

    info!(
        "Found {} font rules, downloading font files...",
        rules.len()
    );

    // Download each font file
    let mut resolved_fonts = Vec::with_capacity(rules.len());
    let mut failed_downloads = 0usize;

    for rule in rules {
        match fetch_font_bytes(&rule.url) {
            Ok(bytes) => {
                info!(
                    "Downloaded {} bytes for {} (weight: {})",
                    bytes.len(),
                    rule.family,
                    rule.weight
                );
                resolved_fonts.push(ResolvedFont::new(
                    rule.family,
                    rule.weight,
                    rule.style,
                    bytes,
                ));
            }
            Err(e) => {
                warn!(
                    "Failed to download font {} (weight: {}) from {}: {}",
                    rule.family, rule.weight, rule.url, e
                );
                failed_downloads += 1;
            }
        }
    }

    if resolved_fonts.is_empty() {
        return Err(FontLoadError::NoCssRules);
    }

    if failed_downloads > 0 {
        warn!(
            "Downloaded {}/{} fonts ({} failed)",
            resolved_fonts.len(),
            resolved_fonts.len() + failed_downloads,
            failed_downloads
        );
    } else {
        info!("Successfully downloaded all {} fonts", resolved_fonts.len());
    }

    Ok(resolved_fonts)
}

/// Load multiple font families from their respective CSS URLs.
///
/// # Arguments
///
/// * `urls` - A slice of Google Fonts CSS URLs.
///
/// # Returns
///
/// A `Result` containing a combined vector of all `ResolvedFont` or a `FontLoadError`.
///
/// # Errors
///
/// Returns an error if any URL fails to load, unless at least one font is successfully loaded.
pub fn load_font_families(urls: &[&str]) -> Result<Vec<ResolvedFont>, FontLoadError> {
    let mut all_fonts = Vec::new();
    let mut any_success = false;

    for url in urls {
        match load_fonts_from_css_url(url) {
            Ok(fonts) => {
                all_fonts.extend(fonts);
                any_success = true;
            }
            Err(e) => {
                warn!("Failed to load fonts from {}: {}", url, e);
            }
        }
    }

    if !any_success {
        return Err(FontLoadError::NoCssRules);
    }

    info!(
        "Loaded {} total fonts from {} families",
        all_fonts.len(),
        urls.len()
    );
    Ok(all_fonts)
}

/// Synchronous font loader for initialization contexts.
///
/// This function is designed to be called during app initialization
/// to load fonts synchronously before the Bevy app starts.
///
/// # Arguments
///
/// * `body_font_url` - Optional URL for body font (defaults to IBM Plex Sans).
/// * `mono_font_url` - Optional URL for monospace font (defaults to JetBrains Mono).
///
/// # Returns
///
/// A `FontDefinitions` struct ready to be used with egui.
pub fn load_default_fonts(
    body_font_url: Option<&str>,
    mono_font_url: Option<&str>,
) -> bevy_egui::egui::FontDefinitions {
    let body_url = body_font_url.unwrap_or(DEFAULT_BODY_FONT_CSS);
    let mono_url = mono_font_url.unwrap_or(DEFAULT_MONO_FONT_CSS);

    info!(
        "Loading default fonts: body={}, mono={}",
        body_url, mono_url
    );

    match load_font_families(&[body_url, mono_url]) {
        Ok(fonts) => build_font_definitions(&fonts),
        Err(e) => {
            warn!("Failed to load custom fonts, using defaults: {}", e);
            bevy_egui::egui::FontDefinitions::default()
        }
    }
}
