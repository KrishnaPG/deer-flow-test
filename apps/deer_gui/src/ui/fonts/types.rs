//! Font type definitions and error types.

use regex::Regex;
use std::fmt;

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

impl fmt::Display for FontStyle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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

/// Compiled regex patterns for CSS parsing.
pub struct FontRegexes {
    pub font_face: Regex,
    pub family: Regex,
    pub weight: Regex,
    pub style: Regex,
    pub url: Regex,
}

impl FontRegexes {
    /// Create and compile all regex patterns.
    pub fn new() -> Self {
        Self {
            font_face: Regex::new(r"@font-face\s*\{([^}]+)\}")
                .expect("Failed to compile font-face regex"),
            family: Regex::new(
                r#"font-family\s*:\s*['"]([^'"]+)['"]|font-family\s*:\s*([^;'"\s]+)"#,
            )
            .expect("Failed to compile family regex"),
            weight: Regex::new(r"font-weight\s*:\s*(\d+)").expect("Failed to compile weight regex"),
            style: Regex::new(r"font-style\s*:\s*(\w+)").expect("Failed to compile style regex"),
            url: Regex::new("src:\\s*url\\(\\s*['\"]?([^'\"\\)]+)['\"]?\\s*\\)")
                .expect("Failed to compile URL regex"),
        }
    }
}

impl Default for FontRegexes {
    fn default() -> Self {
        Self::new()
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
