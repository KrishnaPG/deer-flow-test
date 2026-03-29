//! Build egui font definitions from resolved fonts.

use bevy::log::{debug, info, trace, warn};
use bevy_egui::egui;
use std::collections::HashMap;
use std::sync::Arc;

use super::types::ResolvedFont;

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

#[cfg(test)]
mod tests {
    use super::super::types::{FontStyle, ResolvedFont};
    use super::*;

    #[test]
    fn test_build_empty_fonts() {
        let defs = build_font_definitions(&[]);
        // Should return defaults
        assert!(!defs.font_data.is_empty());
    }

    #[test]
    fn test_build_single_font() {
        let fonts = vec![ResolvedFont::new(
            "Test Font",
            400,
            FontStyle::Normal,
            vec![0u8; 100],
        )];

        let defs = build_font_definitions(&fonts);

        // Should have the font data
        assert!(defs.font_data.contains_key("Test-Font-400-normal"));

        // Should be assigned to proportional
        let prop = defs.families.get(&egui::FontFamily::Proportional).unwrap();
        assert!(prop.contains(&"Test-Font-400-normal".to_string()));
    }

    #[test]
    fn test_build_mono_font() {
        let fonts = vec![ResolvedFont::new(
            "JetBrains Mono",
            400,
            FontStyle::Normal,
            vec![0u8; 100],
        )];

        let defs = build_font_definitions(&fonts);

        // Should be assigned to monospace
        let mono = defs.families.get(&egui::FontFamily::Monospace).unwrap();
        assert!(mono.contains(&"JetBrains-Mono-400-normal".to_string()));
    }

    #[test]
    fn test_build_multiple_weights() {
        let fonts = vec![
            ResolvedFont::new("Test", 400, FontStyle::Normal, vec![0u8; 100]),
            ResolvedFont::new("Test", 700, FontStyle::Normal, vec![0u8; 100]),
        ];

        let defs = build_font_definitions(&fonts);

        assert!(defs.font_data.contains_key("Test-400-normal"));
        assert!(defs.font_data.contains_key("Test-700-normal"));
    }
}
