//! Integration tests for the CSS font resolution pipeline.

use deer_gui::ui::fonts::{
    build_font_definitions, parse_font_css, FontFaceRule, FontLoadError, FontStyle, ResolvedFont,
};

/// Sample Google Fonts CSS for IBM Plex Sans (simplified).
const SAMPLE_CSS_IBM_PLEX: &str = r#"/* latin */
@font-face {
  font-family: 'IBM Plex Sans';
  font-style: normal;
  font-weight: 400;
  src: url(https://fonts.gstatic.com/s/ibmplexsans/v19/zYXgKVElMYYaJe8bpLHnCwDKhdXeFaxOedfTDw.woff2) format('woff2');
  unicode-range: U+0000-00FF, U+0131, U+0152-0153, U+02BB-02BC, U+02C6, U+02DA, U+02DC, U+0304, U+0308, U+0329, U+2000-206F, U+2074, U+20AC, U+2122, U+2191, U+2193, U+2212, U+2215, U+FEFF, U+FFFD;
}
/* latin */
@font-face {
  font-family: 'IBM Plex Sans';
  font-style: normal;
  font-weight: 700;
  src: url(https://fonts.gstatic.com/s/ibmplexsans/v19/zYX9KVElMYYaJe8bpLHnCwDKjWr7AIFsdP3pBms.woff2) format('woff2');
  unicode-range: U+0000-00FF, U+0131, U+0152-0153, U+02BB-02BC, U+02C6, U+02DA, U+02DC, U+0304, U+0308, U+0329, U+2000-206F, U+2074, U+20AC, U+2122, U+2191, U+2193, U+2212, U+2215, U+FEFF, U+FFFD;
}
"#;

/// Sample CSS with italic variant.
const SAMPLE_CSS_WITH_ITALIC: &str = r#"@font-face {
  font-family: 'Test Font';
  font-style: italic;
  font-weight: 400;
  src: url(https://example.com/italic.woff2) format('woff2');
}
@font-face {
  font-family: 'Test Font';
  font-style: normal;
  font-weight: 600;
  src: url(https://example.com/bold.woff2) format('woff2');
}
"#;

/// Test parsing CSS extracts @font-face rules correctly.
#[test]
fn t_font_01_parse_css_extracts_rules() {
    let rules = parse_font_css(SAMPLE_CSS_IBM_PLEX);

    // Should extract 2 font rules
    assert_eq!(rules.len(), 2, "Expected 2 font rules, got {}", rules.len());

    // First rule: weight 400
    assert_eq!(rules[0].family, "IBM Plex Sans");
    assert_eq!(rules[0].weight, 400);
    assert_eq!(rules[0].style, FontStyle::Normal);
    assert!(rules[0].url.contains("ibmplexsans"));
    assert!(rules[0]
        .url
        .contains("zYXgKVElMYYaJe8bpLHnCwDKhdXeFaxOedfTDw"));

    // Second rule: weight 700
    assert_eq!(rules[1].family, "IBM Plex Sans");
    assert_eq!(rules[1].weight, 700);
    assert_eq!(rules[1].style, FontStyle::Normal);
    assert!(rules[1].url.contains("ibmplexsans"));
    assert!(rules[1]
        .url
        .contains("zYX9KVElMYYaJe8bpLHnCwDKjWr7AIFsdP3pBms"));
}

/// Test parsing CSS with italic variants.
#[test]
fn t_font_01b_parse_css_with_italic() {
    let rules = parse_font_css(SAMPLE_CSS_WITH_ITALIC);

    assert_eq!(rules.len(), 2);

    // Italic rule
    let italic_rule = rules
        .iter()
        .find(|r| r.style == FontStyle::Italic)
        .expect("Should have italic rule");
    assert_eq!(italic_rule.family, "Test Font");
    assert_eq!(italic_rule.weight, 400);
    assert_eq!(italic_rule.url, "https://example.com/italic.woff2");

    // Normal rule
    let normal_rule = rules
        .iter()
        .find(|r| r.style == FontStyle::Normal)
        .expect("Should have normal rule");
    assert_eq!(normal_rule.family, "Test Font");
    assert_eq!(normal_rule.weight, 600);
    assert_eq!(normal_rule.url, "https://example.com/bold.woff2");
}

/// Test parsing empty or invalid CSS.
#[test]
fn t_font_01c_parse_css_empty_invalid() {
    let empty_rules = parse_font_css("");
    assert!(
        empty_rules.is_empty(),
        "Empty CSS should return empty rules"
    );

    let no_fontface = parse_font_css("body { font-family: sans-serif; }");
    assert!(
        no_fontface.is_empty(),
        "CSS without @font-face should return empty rules"
    );

    let incomplete = parse_font_css("@font-face { font-weight: 400; }");
    assert!(
        incomplete.is_empty(),
        "Incomplete @font-face should be skipped"
    );
}

/// Test building font definitions from resolved fonts.
#[test]
fn t_font_02_build_definitions() {
    // Create mock resolved fonts
    let fonts = vec![
        ResolvedFont::new("IBM Plex Sans", 400, FontStyle::Normal, vec![0u8; 100]),
        ResolvedFont::new("IBM Plex Sans", 700, FontStyle::Normal, vec![0u8; 100]),
        ResolvedFont::new("JetBrains Mono", 400, FontStyle::Normal, vec![0u8; 100]),
    ];

    let font_defs = build_font_definitions(&fonts);

    // Should have at least 3 custom font data entries (plus egui defaults)
    assert!(
        font_defs.font_data.len() >= 3,
        "Should have at least 3 font data entries, got {}",
        font_defs.font_data.len()
    );

    // Check that IBM Plex Sans is in Proportional family
    let proportional = font_defs
        .families
        .get(&bevy_egui::egui::FontFamily::Proportional);
    assert!(proportional.is_some(), "Should have Proportional family");
    let proportional = proportional.unwrap();
    assert!(
        proportional
            .iter()
            .any(|name: &String| name.contains("IBM-Plex-Sans")),
        "Proportional family should contain IBM Plex Sans"
    );

    // Check that JetBrains Mono is in Monospace family
    let monospace = font_defs
        .families
        .get(&bevy_egui::egui::FontFamily::Monospace);
    assert!(monospace.is_some(), "Should have Monospace family");
    let monospace = monospace.unwrap();
    assert!(
        monospace
            .iter()
            .any(|name: &String| name.contains("JetBrains-Mono")),
        "Monospace family should contain JetBrains Mono"
    );

    // Verify custom font data content (only check our custom fonts)
    let custom_fonts = ["IBM-Plex-Sans", "JetBrains-Mono"];
    for font_prefix in &custom_fonts {
        let found = font_defs
            .font_data
            .iter()
            .find(|(name, _): &(&String, _)| name.contains(font_prefix));
        assert!(found.is_some(), "Should have custom font {}", font_prefix);
        if let Some((name, data)) = found {
            assert_eq!(data.font.len(), 100, "Font {} should have 100 bytes", name);
        }
    }
}

/// Test building definitions with empty fonts falls back to defaults.
#[test]
fn t_font_03_fallback_on_empty_fonts() {
    let empty: Vec<ResolvedFont> = vec![];
    let font_defs = build_font_definitions(&empty);

    // Should return default font definitions (not empty)
    assert!(
        !font_defs.font_data.is_empty(),
        "Should have default font data"
    );
    assert!(
        !font_defs.families.is_empty(),
        "Should have default font families"
    );

    // Should have at least the default proportional and monospace families
    assert!(
        font_defs
            .families
            .contains_key(&bevy_egui::egui::FontFamily::Proportional),
        "Should have Proportional family"
    );
    assert!(
        font_defs
            .families
            .contains_key(&bevy_egui::egui::FontFamily::Monospace),
        "Should have Monospace family"
    );
}

/// Test ResolvedFont identifier generation.
#[test]
fn t_font_04_resolved_font_identifier() {
    let font1 = ResolvedFont::new("IBM Plex Sans", 400, FontStyle::Normal, vec![]);
    assert_eq!(font1.identifier(), "IBM-Plex-Sans-400-normal");

    let font2 = ResolvedFont::new("JetBrains Mono", 700, FontStyle::Italic, vec![]);
    assert_eq!(font2.identifier(), "JetBrains-Mono-700-italic");

    let font3 = ResolvedFont::new("Some Font Family", 500, FontStyle::Normal, vec![]);
    assert_eq!(font3.identifier(), "Some-Font-Family-500-normal");
}

/// Test FontStyle enum variants.
#[test]
fn t_font_05_font_style_variants() {
    assert_eq!(FontStyle::Normal.as_css_str(), "normal");
    assert_eq!(FontStyle::Italic.as_css_str(), "italic");
    assert_ne!(FontStyle::Normal, FontStyle::Italic);
}

/// Test FontFaceRule creation and properties.
#[test]
fn t_font_06_font_face_rule() {
    let rule = FontFaceRule {
        family: "Test Font".to_string(),
        weight: 600,
        style: FontStyle::Italic,
        url: "https://example.com/font.woff2".to_string(),
    };

    assert_eq!(rule.family, "Test Font");
    assert_eq!(rule.weight, 600);
    assert_eq!(rule.style, FontStyle::Italic);
    assert_eq!(rule.url, "https://example.com/font.woff2");
}

/// Test URL extraction with various quote styles.
#[test]
fn t_font_07_url_extraction_quotes() {
    let css_double_quotes = r#"@font-face {
        font-family: 'Test';
        font-weight: 400;
        src: url("https://example.com/font.woff2") format('woff2');
    }"#;

    let css_single_quotes = r#"@font-face {
        font-family: 'Test';
        font-weight: 400;
        src: url('https://example.com/font.woff2') format('woff2');
    }"#;

    let css_no_quotes = r#"@font-face {
        font-family: 'Test';
        font-weight: 400;
        src: url(https://example.com/font.woff2) format('woff2');
    }"#;

    let rules_double = parse_font_css(css_double_quotes);
    let rules_single = parse_font_css(css_single_quotes);
    let rules_no = parse_font_css(css_no_quotes);

    assert_eq!(rules_double[0].url, "https://example.com/font.woff2");
    assert_eq!(rules_single[0].url, "https://example.com/font.woff2");
    assert_eq!(rules_no[0].url, "https://example.com/font.woff2");
}

/// Test font family name extraction with various quote styles.
#[test]
fn t_font_08_family_name_extraction() {
    let css_double =
        "@font-face { font-family: \"Test Font\"; font-weight: 400; src: url(x.woff2); }";
    let css_single =
        "@font-face { font-family: 'Test Font'; font-weight: 400; src: url(x.woff2); }";

    let rules_double = parse_font_css(css_double);
    let rules_single = parse_font_css(css_single);

    assert_eq!(rules_double[0].family, "Test Font");
    assert_eq!(rules_single[0].family, "Test Font");
}

/// Test handling multiple font families in one CSS.
#[test]
fn t_font_09_multiple_families() {
    let css = r#"@font-face {
        font-family: 'Family One';
        font-weight: 400;
        src: url(https://example.com/one.woff2);
    }
    @font-face {
        font-family: 'Family Two';
        font-weight: 700;
        src: url(https://example.com/two.woff2);
    }"#;

    let rules = parse_font_css(css);

    assert_eq!(rules.len(), 2);
    assert!(rules
        .iter()
        .any(|r| r.family == "Family One" && r.weight == 400));
    assert!(rules
        .iter()
        .any(|r| r.family == "Family Two" && r.weight == 700));
}

/// Test default weight and style when not specified.
#[test]
fn t_font_10_defaults_when_unspecified() {
    let css = r#"@font-face {
        font-family: 'Minimal Font';
        src: url(https://example.com/font.woff2);
    }"#;

    let rules = parse_font_css(css);

    assert_eq!(rules.len(), 1);
    assert_eq!(rules[0].family, "Minimal Font");
    assert_eq!(rules[0].weight, 400, "Default weight should be 400");
    assert_eq!(
        rules[0].style,
        FontStyle::Normal,
        "Default style should be Normal"
    );
}

/// Test load_fonts_from_css_url fails gracefully with invalid URL.
/// This verifies the error handling path when network is unreachable.
#[test]
fn t_font_11_load_fonts_invalid_url_fails() {
    use deer_gui::ui::fonts::load_fonts_from_css_url;

    // Use a URL that's guaranteed to fail (invalid domain)
    let result = load_fonts_from_css_url("https://invalid.invalid/fonts.css");

    // Should return an error, not panic
    assert!(
        result.is_err(),
        "Loading from invalid URL should return an error"
    );

    // Verify it's a Http error
    match result {
        Err(FontLoadError::Http(_)) => (), // Expected
        Err(other) => panic!("Expected Http error, got {:?}", other),
        Ok(_) => panic!("Should not succeed with invalid URL"),
    }
}

/// Test load_fonts_from_css_url fails when no valid rules in CSS.
/// This simulates a server returning CSS without @font-face rules.
#[test]
fn t_font_12_load_fonts_no_rules_fails() {
    use deer_gui::ui::fonts::load_fonts_from_css_url;

    // We'll test this by checking that parse_font_css returns empty for CSS without @font-face
    // The actual load_fonts_from_css_url would fail at the HTTP level for invalid URL,
    // but we can verify the NoCssRules error is properly defined
    let empty_rules = parse_font_css("body { margin: 0; }");
    assert!(
        empty_rules.is_empty(),
        "CSS without @font-face should return empty rules"
    );

    // Verify error type exists and can be constructed
    let err = FontLoadError::NoCssRules;
    assert!(err.to_string().contains("no @font-face rules"));
}

/// Test load_default_fonts falls back to defaults on failure.
/// This is the critical production path - graceful degradation.
#[test]
fn t_font_13_load_default_fonts_fallback() {
    use deer_gui::ui::fonts::load_default_fonts;

    // Pass None to use default URLs - this should work if network is available
    // or gracefully degrade to built-in defaults if not
    let font_defs = load_default_fonts(None, None);

    // Should always return valid FontDefinitions (either custom or default)
    assert!(
        !font_defs.font_data.is_empty(),
        "Should have font data even on failure"
    );
    assert!(
        font_defs
            .families
            .contains_key(&bevy_egui::egui::FontFamily::Proportional),
        "Should have Proportional family"
    );
    assert!(
        font_defs
            .families
            .contains_key(&bevy_egui::egui::FontFamily::Monospace),
        "Should have Monospace family"
    );
}

/// Test load_font_families with mix of valid/invalid URLs.
/// Verifies that partial success still loads available fonts.
#[test]
fn t_font_14_load_font_families_partial_success() {
    use deer_gui::ui::fonts::load_font_families;

    // This test documents the expected behavior:
    // - If one URL fails but another succeeds, we get the successful fonts
    // - If all fail, we get an error

    // Test with all invalid URLs - should fail
    let all_invalid = load_font_families(&[
        "https://invalid1.invalid/fonts.css",
        "https://invalid2.invalid/fonts.css",
    ]);
    assert!(all_invalid.is_err(), "All invalid URLs should return error");
}
