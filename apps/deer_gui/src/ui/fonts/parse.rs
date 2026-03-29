//! CSS parsing for @font-face rules.

use bevy::log::{debug, trace, warn};

use super::types::{FontFaceRule, FontRegexes, FontStyle};

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
    let regexes = FontRegexes::new();

    for cap in regexes.font_face.captures_iter(css) {
        let block = &cap[1];
        trace!("Processing @font-face block");

        // Extract font family
        let family = regexes
            .family
            .captures(block)
            .and_then(|c| c.get(1).or_else(|| c.get(2)))
            .map(|m| m.as_str().trim().to_string())
            .unwrap_or_default();

        if family.is_empty() {
            warn!("Skipping @font-face block: missing font-family");
            continue;
        }

        // Extract font weight (default to 400)
        let weight = regexes
            .weight
            .captures(block)
            .and_then(|c| c.get(1))
            .and_then(|m| m.as_str().parse::<u16>().ok())
            .unwrap_or(400);

        // Extract font style (default to Normal)
        let style = regexes
            .style
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
        let url = regexes
            .url
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_font_rule() {
        let css = r#"
            @font-face {
                font-family: 'Test Font';
                font-style: normal;
                font-weight: 400;
                src: url(https://example.com/font.woff2) format('woff2');
            }
        "#;

        let rules = parse_font_css(css);
        assert_eq!(rules.len(), 1);
        assert_eq!(rules[0].family, "Test Font");
        assert_eq!(rules[0].weight, 400);
        assert_eq!(rules[0].style, FontStyle::Normal);
        assert_eq!(rules[0].url, "https://example.com/font.woff2");
    }

    #[test]
    fn test_parse_multiple_font_rules() {
        let css = r#"
            @font-face {
                font-family: 'Font A';
                font-weight: 400;
                src: url(https://a.com/400.woff2);
            }
            @font-face {
                font-family: 'Font B';
                font-weight: 700;
                src: url(https://b.com/700.woff2);
            }
        "#;

        let rules = parse_font_css(css);
        assert_eq!(rules.len(), 2);
        assert_eq!(rules[0].family, "Font A");
        assert_eq!(rules[1].family, "Font B");
    }

    #[test]
    fn test_parse_italic_style() {
        let css = r#"
            @font-face {
                font-family: 'Italic Font';
                font-style: italic;
                font-weight: 400;
                src: url(https://example.com/italic.woff2);
            }
        "#;

        let rules = parse_font_css(css);
        assert_eq!(rules.len(), 1);
        assert_eq!(rules[0].style, FontStyle::Italic);
    }

    #[test]
    fn test_parse_missing_family_skipped() {
        let css = r#"
            @font-face {
                font-weight: 400;
                src: url(https://example.com/font.woff2);
            }
        "#;

        let rules = parse_font_css(css);
        assert!(rules.is_empty());
    }

    #[test]
    fn test_parse_missing_url_skipped() {
        let css = r#"
            @font-face {
                font-family: 'No URL';
                font-weight: 400;
            }
        "#;

        let rules = parse_font_css(css);
        assert!(rules.is_empty());
    }
}
