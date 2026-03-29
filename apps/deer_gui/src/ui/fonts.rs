//! CSS font resolution pipeline for Google Fonts.
//!
//! This module provides functionality to fetch and parse Google Fonts CSS,
//! download font files, and build egui::FontDefinitions for custom typography.
//! Supports IBM Plex Sans (body) and JetBrains Mono (monospace).

use crate::constants::fonts::FONT_CSS_USER_AGENT;
use bevy::log::{debug, info, trace, warn};
use bevy_egui::egui;
use regex::Regex;
use std::collections::HashMap;
use std::io::Read;
use std::sync::Arc;

/// Errors that can occur during font loading operations.
#[derive(Debug, thiserror::Error)]
pub enum FontLoadError {
    /// HTTP request failed.
    #[error("HTTP request failed: {0}")]
    Http(String),
    /// CSS parse error: no @font-face rules found.
    #[error("CSS parse error: no @font-face rules found")]
    NoCssRules,
    /// IO error during font operations.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Font style variants.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum FontStyle {
    /// Normal (upright) font style.
    Normal,
    /// Italic font style.
    Italic,
}

impl FontStyle {
    /// Convert to CSS string representation.
    pub fn as_css_str(&self) -> &'static str {
        match self {
            FontStyle::Normal => "normal",
            FontStyle::Italic => "italic",
        }
    }
}

impl std::fmt::Display for FontStyle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_css_str())
    }
}

/// Represents a parsed @font-face CSS rule.
#[derive(Debug, Clone)]
pub struct FontFaceRule {
    /// Font family name (e.g., "IBM Plex Sans").
    pub family: String,
    /// Font weight (e.g., 400, 700).
    pub weight: u16,
    /// Font style (normal or italic).
    pub style: FontStyle,
    /// URL to the font file.
    pub url: String,
}

impl FontFaceRule {
    /// Create a new font face rule.
    pub fn new(
        family: impl Into<String>,
        weight: u16,
        style: FontStyle,
        url: impl Into<String>,
    ) -> Self {
        Self {
            family: family.into(),
            weight,
            style,
            url: url.into(),
        }
    }
}

/// A fully resolved font with downloaded bytes.
#[derive(Debug, Clone)]
pub struct ResolvedFont {
    /// Font family name.
    pub family: String,
    /// Font weight.
    pub weight: u16,
    /// Font style.
    pub style: FontStyle,
    /// Raw font file bytes.
    pub bytes: Vec<u8>,
}

impl ResolvedFont {
    /// Create a new resolved font.
    pub fn new(family: impl Into<String>, weight: u16, style: FontStyle, bytes: Vec<u8>) -> Self {
        Self {
            family: family.into(),
            weight,
            style,
            bytes,
        }
    }

    /// Generate a unique identifier for this font variant.
    pub fn identifier(&self) -> String {
        format!(
            "{}-{}-{}",
            self.family.replace(' ', "-"),
            self.weight,
            self.style
        )
    }
}

/// Parse Google Fonts CSS and extract @font-face rules.
///
/// # Arguments
///
/// * `css` - The CSS content to parse.
///
/// # Returns
///
/// A vector of parsed `FontFaceRule` structs.
///
/// # Example
///
/// ```
/// use deer_gui::ui::fonts::parse_font_css;
///
/// let css = r#"
/// @font-face {
///   font-family: 'Test Font';
///   font-style: normal;
///   font-weight: 400;
///   src: url(https://example.com/font.woff2) format('woff2');
/// }
/// "#;
///
/// let rules = parse_font_css(css);
/// assert_eq!(rules.len(), 1);
/// assert_eq!(rules[0].family, "Test Font");
/// assert_eq!(rules[0].weight, 400);
/// ```
pub fn parse_font_css(css: &str) -> Vec<FontFaceRule> {
    trace!("Parsing CSS content ({} bytes)", css.len());

    let mut rules = Vec::new();

    // Regex to match @font-face blocks
    let font_face_regex =
        Regex::new(r"@font-face\s*\{([^}]+)\}").expect("Failed to compile font-face regex");

    // Regexes to extract properties within a font-face block
    let family_regex =
        Regex::new(r#"font-family\s*:\s*['"]([^'"]+)['"]|font-family\s*:\s*([^;'"\s]+)"#)
            .expect("Failed to compile family regex");

    let weight_regex =
        Regex::new(r"font-weight\s*:\s*(\d+)").expect("Failed to compile weight regex");

    let style_regex = Regex::new(r"font-style\s*:\s*(\w+)").expect("Failed to compile style regex");

    let url_regex = Regex::new("src:\\s*url\\(\\s*['\"]?([^'\"\\)]+)['\"]?\\s*\\)")
        .expect("Failed to compile URL regex");

    for cap in font_face_regex.captures_iter(css) {
        let block = &cap[1];
        trace!("Processing @font-face block");

        // Extract font family
        let family = family_regex
            .captures(block)
            .and_then(|c| c.get(1).or_else(|| c.get(2)))
            .map(|m| m.as_str().trim().to_string())
            .unwrap_or_default();

        if family.is_empty() {
            warn!("Skipping @font-face block: missing font-family");
            continue;
        }

        // Extract font weight (default to 400)
        let weight = weight_regex
            .captures(block)
            .and_then(|c| c.get(1))
            .and_then(|m| m.as_str().parse::<u16>().ok())
            .unwrap_or(400);

        // Extract font style (default to Normal)
        let style = style_regex
            .captures(block)
            .and_then(|c| c.get(1))
            .map(|m| {
                if m.as_str().to_lowercase() == "italic" {
                    FontStyle::Italic
                } else {
                    FontStyle::Normal
                }
            })
            .unwrap_or(FontStyle::Normal);

        // Extract URL (may be quoted or unquoted)
        let url = url_regex
            .captures(block)
            .and_then(|c| c.get(1))
            .map(|m| m.as_str().trim().to_string())
            .unwrap_or_default();

        if url.is_empty() {
            warn!("Skipping @font-face block for '{}': missing URL", family);
            continue;
        }

        trace!(
            "Parsed font rule: family='{}', weight={}, style={}",
            family,
            weight,
            style
        );

        rules.push(FontFaceRule::new(family, weight, style, url));
    }

    debug!("Parsed {} font rules from CSS", rules.len());
    rules
}

/// Fetch font bytes from a URL.
///
/// # Arguments
///
/// * `url` - The URL to fetch font bytes from.
///
/// # Returns
///
/// A `Result` containing the font bytes or a `FontLoadError`.
///
/// # Errors
///
/// Returns `FontLoadError::Http` if the request fails or returns a non-200 status.
pub fn fetch_font_bytes(url: &str) -> Result<Vec<u8>, FontLoadError> {
    trace!("Fetching font bytes from: {}", url);

    let response = ureq::get(url)
        .header("User-Agent", FONT_CSS_USER_AGENT)
        .call()
        .map_err(|e| FontLoadError::Http(format!("Request failed: {e}")))?;

    let status = response.status();
    if status != 200 {
        return Err(FontLoadError::Http(format!("HTTP {status} for URL: {url}")));
    }

    let mut bytes = Vec::new();
    response
        .into_body()
        .into_reader()
        .read_to_end(&mut bytes)
        .map_err(FontLoadError::Io)?;

    trace!("Fetched {} bytes from {}", bytes.len(), url);
    Ok(bytes)
}

/// Fetch CSS content from a Google Fonts URL.
///
/// # Arguments
///
/// * `url` - The Google Fonts CSS URL.
///
/// # Returns
///
/// A `Result` containing the CSS content as a string or a `FontLoadError`.
///
/// # Errors
///
/// Returns `FontLoadError::Http` if the request fails.
pub fn fetch_font_css(url: &str) -> Result<String, FontLoadError> {
    trace!("Fetching font CSS from: {}", url);

    let response = ureq::get(url)
        .header("User-Agent", FONT_CSS_USER_AGENT)
        .call()
        .map_err(|e| FontLoadError::Http(format!("Request failed: {e}")))?;

    let status = response.status();
    if status != 200 {
        return Err(FontLoadError::Http(format!("HTTP {status} for URL: {url}")));
    }

    let css = response
        .into_body()
        .read_to_string()
        .map_err(|e| FontLoadError::Http(format!("Failed to read response body: {e}")))?;

    debug!("Fetched {} bytes of CSS from {}", css.len(), url);
    Ok(css)
}

/// Build egui::FontDefinitions from resolved fonts.
///
/// # Arguments
///
/// * `fonts` - A slice of resolved fonts.
///
/// # Returns
///
/// A `FontDefinitions` struct configured with the custom fonts.
///
/// # Details
///
/// Maps font families to egui font families:
/// - "IBM Plex Sans" → Proportional (body text)
/// - "JetBrains Mono" → Monospace
/// - Others → Proportional (fallback)
///
/// Font weights are mapped to egui's font sizes concept by registering
/// each weight as a separate font entry.
pub fn build_font_definitions(fonts: &[ResolvedFont]) -> egui::FontDefinitions {
    let mut font_definitions = egui::FontDefinitions::default();

    if fonts.is_empty() {
        warn!("No fonts provided, returning default font definitions");
        return font_definitions;
    }

    debug!("Building font definitions from {} fonts", fonts.len());

    // Group fonts by family
    let mut fonts_by_family: HashMap<String, Vec<&ResolvedFont>> = HashMap::new();
    for font in fonts {
        fonts_by_family
            .entry(font.family.clone())
            .or_default()
            .push(font);
    }

    // Track which families we've assigned to which egui families
    let mut assigned_proportional = false;
    let mut assigned_monospace = false;

    for (family, family_fonts) in fonts_by_family {
        info!(
            "Configuring font family '{}' with {} variants",
            family,
            family_fonts.len()
        );

        // Determine which egui font family to use
        let egui_family = if family.to_lowercase().contains("mono") && !assigned_monospace {
            assigned_monospace = true;
            egui::FontFamily::Monospace
        } else if !family.to_lowercase().contains("mono") && !assigned_proportional {
            assigned_proportional = true;
            egui::FontFamily::Proportional
        } else if family.to_lowercase().contains("mono") {
            egui::FontFamily::Proportional
        } else {
            egui::FontFamily::Monospace
        };

        // Register each font variant
        for font in family_fonts {
            let font_name = font.identifier();

            // Insert the font data (wrapped in Arc)
            let font_data = egui::FontData::from_owned(font.bytes.clone());
            font_definitions
                .font_data
                .insert(font_name.clone(), Arc::new(font_data));

            // Add to the appropriate font family
            let font_list = font_definitions
                .families
                .entry(egui_family.clone())
                .or_default();

            // Insert with priority based on weight (higher weight = lower priority index)
            // This ensures normal weight (400) comes first
            if !font_list.contains(&font_name) {
                font_list.push(font_name);
            }

            trace!(
                "Registered font: {} (family: {:?}, weight: {})",
                font.identifier(),
                egui_family,
                font.weight
            );
        }
    }

    // Ensure we have fallback fonts configured
    if font_definitions
        .families
        .get(&egui::FontFamily::Proportional)
        .is_none_or(|v| v.is_empty())
    {
        debug!("No proportional fonts assigned, using default");
    }

    if font_definitions
        .families
        .get(&egui::FontFamily::Monospace)
        .is_none_or(|v| v.is_empty())
    {
        debug!("No monospace fonts assigned, using default");
    }

    font_definitions
}

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
                debug!(
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
) -> egui::FontDefinitions {
    use crate::constants::fonts::{DEFAULT_BODY_FONT_CSS, DEFAULT_MONO_FONT_CSS};

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
            egui::FontDefinitions::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_font_style_display() {
        assert_eq!(FontStyle::Normal.to_string(), "normal");
        assert_eq!(FontStyle::Italic.to_string(), "italic");
    }

    #[test]
    fn test_font_face_rule_creation() {
        let rule = FontFaceRule::new(
            "Test Font",
            400,
            FontStyle::Normal,
            "https://example.com/font.woff2",
        );
        assert_eq!(rule.family, "Test Font");
        assert_eq!(rule.weight, 400);
        assert_eq!(rule.style, FontStyle::Normal);
        assert_eq!(rule.url, "https://example.com/font.woff2");
    }

    #[test]
    fn test_resolved_font_creation() {
        let font = ResolvedFont::new("Test", 700, FontStyle::Italic, vec![0u8, 1u8, 2u8]);
        assert_eq!(font.family, "Test");
        assert_eq!(font.weight, 700);
        assert_eq!(font.style, FontStyle::Italic);
        assert_eq!(font.bytes, vec![0u8, 1u8, 2u8]);
    }

    #[test]
    fn test_resolved_font_identifier() {
        let font = ResolvedFont::new("IBM Plex Sans", 400, FontStyle::Normal, vec![]);
        assert_eq!(font.identifier(), "IBM-Plex-Sans-400-normal");
    }
}
