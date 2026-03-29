# PR G — CSS Font Loading from Google Fonts

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add a CSS font resolution pipeline that accepts Google Fonts CSS URLs, parses `@font-face` rules, fetches font bytes over HTTP, and builds `egui::FontDefinitions` for custom typography (IBM Plex Sans + JetBrains Mono).

**Architecture:** A `ui::fonts` module provides a pure-function pipeline: CSS URL → parsed rules → fetched bytes → `FontDefinitions`. HTTP is handled by `ureq` (synchronous, no-async). Fallback to Bevy's `default_font` feature on failure. No Bevy systems in this PR — just the library functions that a future `ThemePlugin` will call.

**Tech Stack:** Rust, Bevy 0.18.1, bevy_egui 0.39, ureq 3, regex

---

## File Map

| Action | Path | Responsibility |
| ------ | ---- | -------------- |
| Modify | `Cargo.toml` | Add `ureq = "3"` and `regex = "1"` dependencies |
| Create | `src/ui/mod.rs` | Module declaration: `pub mod fonts;` |
| Create | `src/ui/fonts.rs` | Font resolution pipeline (~150 LOC) |
| Modify | `src/constants.rs` | Add `pub mod fonts` with CSS URLs and User-Agent |
| Modify | `src/lib.rs` | Add `pub mod ui;` |
| Create | `tests/integration/fonts.rs` | 3 integration tests (non-network) |
| Modify | `tests/integration.rs` | Add `pub mod fonts;` |

---

### Task 1: Add dependencies to `Cargo.toml`

**Files:**
- Modify: `apps/deer_gui/Cargo.toml`

- [ ] **Step 1: Add `ureq` and `regex` to `[dependencies]`**

After the `thiserror = "2"` line, add:

```toml
# ── HTTP / parsing ──────────────────────────────────────────────────
ureq = "3"
regex = "1"
```

- [ ] **Step 2: Verify the dependency resolves**

Run: `cargo check -p deer-gui 2>&1 | head -20`
Expected: Compiles (possibly with warnings about unused deps — that's fine until the module is wired).

---

### Task 2: Add font constants to `src/constants.rs`

**Files:**
- Modify: `apps/deer_gui/src/constants.rs`

- [ ] **Step 1: Append the `fonts` module**

Add at the end of the file, after the `aggregation` module:

```rust
// ---------------------------------------------------------------------------
// Fonts
// ---------------------------------------------------------------------------

/// Google Fonts CSS URLs and HTTP client settings for font loading.
pub mod fonts {
    /// Default Google Fonts CSS URL for body text (IBM Plex Sans).
    pub const DEFAULT_BODY_FONT_CSS: &str =
        "https://fonts.googleapis.com/css2?family=IBM+Plex+Sans:wght@400;500;700&display=swap";
    /// Default Google Fonts CSS URL for monospace / code text (JetBrains Mono).
    pub const DEFAULT_MONO_FONT_CSS: &str =
        "https://fonts.googleapis.com/css2?family=JetBrains+Mono:wght@400&display=swap";
    /// User-Agent header sent with Google Fonts CSS requests.
    ///
    /// A browser-like User-Agent is required so Google returns direct `.ttf`
    /// URLs instead of `.woff2`-only responses.
    pub const FONT_CSS_USER_AGENT: &str =
        "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36";
}
```

---

### Task 3: Create `src/ui/fonts.rs`

**Files:**
- Create: `apps/deer_gui/src/ui/fonts.rs`

- [ ] **Step 1: Write the fonts module**

```rust
//! CSS font resolution pipeline for Google Fonts.
//!
//! Accepts a Google Fonts CSS URL, parses `@font-face` rules to extract
//! font-file URLs, fetches the raw bytes, and builds `egui::FontDefinitions`.
//!
//! The pipeline is synchronous (uses `ureq`) and intended to run once at
//! startup. If any step fails, callers should fall back to Bevy's built-in
//! `default_font`.

use bevy::log::{debug, info, trace, warn};
use bevy_egui::egui;
use regex::Regex;

use crate::constants::fonts::{FONT_CSS_USER_AGENT};

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

/// Errors that can occur during font loading.
#[derive(Debug, thiserror::Error)]
pub enum FontLoadError {
    /// HTTP request failed.
    #[error("HTTP request failed: {0}")]
    Http(String),
    /// CSS contained no parseable `@font-face` rules.
    #[error("CSS parse error: no @font-face rules found")]
    NoCssRules,
    /// An IO error occurred while reading the response body.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

// ---------------------------------------------------------------------------
// Domain types
// ---------------------------------------------------------------------------

/// Font style — normal or italic.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FontStyle {
    /// Upright / roman style.
    Normal,
    /// Italic style.
    Italic,
}

/// A single `@font-face` rule extracted from a Google Fonts CSS response.
#[derive(Debug, Clone)]
pub struct FontFaceRule {
    /// Font family name (e.g. `"IBM Plex Sans"`).
    pub family: String,
    /// Numeric font weight (e.g. `400`, `700`).
    pub weight: u16,
    /// Font style (normal or italic).
    pub style: FontStyle,
    /// Direct URL to the font file.
    pub url: String,
}

/// A fully resolved font — metadata plus raw file bytes.
#[derive(Debug)]
pub struct ResolvedFont {
    /// Font family name.
    pub family: String,
    /// Numeric font weight.
    pub weight: u16,
    /// Font style.
    pub style: FontStyle,
    /// Raw font file bytes (TTF / WOFF2).
    pub bytes: Vec<u8>,
}

// ---------------------------------------------------------------------------
// CSS parsing
// ---------------------------------------------------------------------------

/// Parse a Google Fonts CSS response and extract [`FontFaceRule`]s.
///
/// Uses regex to match `@font-face { … }` blocks and pull out
/// `font-family`, `font-weight`, `font-style`, and `src: url(…)`.
pub fn parse_font_css(css: &str) -> Vec<FontFaceRule> {
    trace!("parse_font_css — input length={}", css.len());

    let block_re = Regex::new(
        r"(?s)@font-face\s*\{(.*?)\}"
    ).expect("block regex is valid");

    let family_re = Regex::new(
        r"font-family:\s*'([^']+)'"
    ).expect("family regex is valid");

    let weight_re = Regex::new(
        r"font-weight:\s*(\d+)"
    ).expect("weight regex is valid");

    let style_re = Regex::new(
        r"font-style:\s*(\w+)"
    ).expect("style regex is valid");

    let url_re = Regex::new(
        r"src:\s*url\(([^)]+)\)"
    ).expect("url regex is valid");

    let mut rules = Vec::new();

    for block_cap in block_re.captures_iter(css) {
        let block = &block_cap[1];

        let family = match family_re.captures(block) {
            Some(cap) => cap[1].to_string(),
            None => {
                debug!("parse_font_css — skipping block: no font-family");
                continue;
            }
        };

        let weight: u16 = match weight_re.captures(block) {
            Some(cap) => cap[1].parse().unwrap_or(400),
            None => 400,
        };

        let style = match style_re.captures(block) {
            Some(cap) if &cap[1] == "italic" => FontStyle::Italic,
            _ => FontStyle::Normal,
        };

        let url = match url_re.captures(block) {
            Some(cap) => cap[1].to_string(),
            None => {
                debug!("parse_font_css — skipping block: no src url");
                continue;
            }
        };

        trace!(
            "parse_font_css — found: family={family}, weight={weight}, style={style:?}, url={url}"
        );
        rules.push(FontFaceRule { family, weight, style, url });
    }

    info!("parse_font_css — extracted {} rules", rules.len());
    rules
}

// ---------------------------------------------------------------------------
// HTTP fetching
// ---------------------------------------------------------------------------

/// Fetch the raw bytes of a font file from `url`.
///
/// Uses `ureq` for a synchronous HTTP GET request.
pub fn fetch_font_bytes(url: &str) -> Result<Vec<u8>, FontLoadError> {
    trace!("fetch_font_bytes — url={url}");

    let agent = ureq::Agent::new_with_defaults();
    let response = agent
        .get(url)
        .call()
        .map_err(|e| FontLoadError::Http(format!("{e}")))?;

    let mut bytes = Vec::new();
    response
        .into_body()
        .read_to_vec(&mut bytes)
        .map_err(|e| FontLoadError::Io(e.into_io()))?;

    debug!("fetch_font_bytes — received {} bytes from {url}", bytes.len());
    Ok(bytes)
}

/// Fetch the CSS text from a Google Fonts CSS URL.
///
/// Sends a browser-like `User-Agent` header so Google returns direct
/// `.ttf` URLs instead of `.woff2`-only responses.
fn fetch_font_css(url: &str) -> Result<String, FontLoadError> {
    trace!("fetch_font_css — url={url}");

    let agent = ureq::Agent::new_with_defaults();
    let response = agent
        .get(url)
        .header("User-Agent", FONT_CSS_USER_AGENT)
        .call()
        .map_err(|e| FontLoadError::Http(format!("{e}")))?;

    let mut body = String::new();
    response
        .into_body()
        .read_to_string(&mut body)
        .map_err(|e| FontLoadError::Io(e.into_io()))?;

    debug!("fetch_font_css — received {} bytes of CSS", body.len());
    Ok(body)
}

// ---------------------------------------------------------------------------
// Font definitions builder
// ---------------------------------------------------------------------------

/// Build `egui::FontDefinitions` from a slice of resolved fonts.
///
/// Each font is registered as a named font-data entry. The first font in the
/// list is inserted at the front of the `Proportional` family; subsequent
/// fonts are appended. If the input slice is empty, default
/// `FontDefinitions` are returned.
pub fn build_font_definitions(fonts: &[ResolvedFont]) -> egui::FontDefinitions {
    trace!("build_font_definitions — {} fonts", fonts.len());

    let mut defs = egui::FontDefinitions::default();

    for font in fonts {
        let name = format!("{}-{}-{:?}", font.family, font.weight, font.style);
        trace!("build_font_definitions — registering '{name}'");

        defs.font_data.insert(
            name.clone(),
            egui::FontData::from_owned(font.bytes.clone()).into(),
        );

        // Add to Proportional family by default.
        let families = defs
            .families
            .entry(egui::FontFamily::Proportional)
            .or_default();

        if families.is_empty() {
            families.push(name);
        } else {
            // Insert after the first entry to keep the egui default fallback.
            families.insert(0, name);
        }
    }

    info!(
        "build_font_definitions — {} font-data entries, {} proportional families",
        defs.font_data.len(),
        defs.families
            .get(&egui::FontFamily::Proportional)
            .map_or(0, |f| f.len()),
    );

    defs
}

// ---------------------------------------------------------------------------
// High-level pipeline
// ---------------------------------------------------------------------------

/// Load fonts from a Google Fonts CSS URL.
///
/// 1. Fetch the CSS from `url`
/// 2. Parse `@font-face` rules
/// 3. Fetch bytes for each font-file URL
/// 4. Return the resolved fonts
///
/// On failure at any step, returns `Err(FontLoadError)`.
pub fn load_fonts_from_css_url(url: &str) -> Result<Vec<ResolvedFont>, FontLoadError> {
    info!("load_fonts_from_css_url — url={url}");

    let css = fetch_font_css(url)?;
    let rules = parse_font_css(&css);

    if rules.is_empty() {
        warn!("load_fonts_from_css_url — no @font-face rules found in CSS");
        return Err(FontLoadError::NoCssRules);
    }

    let mut fonts = Vec::with_capacity(rules.len());

    for rule in &rules {
        match fetch_font_bytes(&rule.url) {
            Ok(bytes) => {
                debug!(
                    "load_fonts_from_css_url — resolved {}/{}/{:?} ({} bytes)",
                    rule.family, rule.weight, rule.style, bytes.len()
                );
                fonts.push(ResolvedFont {
                    family: rule.family.clone(),
                    weight: rule.weight,
                    style: rule.style.clone(),
                    bytes,
                });
            }
            Err(e) => {
                warn!(
                    "load_fonts_from_css_url — failed to fetch {}: {e}",
                    rule.url
                );
                // Continue with remaining fonts — partial success is ok.
            }
        }
    }

    info!(
        "load_fonts_from_css_url — resolved {}/{} fonts",
        fonts.len(),
        rules.len()
    );

    Ok(fonts)
}
```

---

### Task 4: Create `src/ui/mod.rs`

**Files:**
- Create: `apps/deer_gui/src/ui/mod.rs`

- [ ] **Step 1: Write the module file**

```rust
//! UI utilities — font loading, styling helpers.

pub mod fonts;
```

---

### Task 5: Wire `ui` module into `lib.rs`

**Files:**
- Modify: `apps/deer_gui/src/lib.rs`

- [ ] **Step 1: Add `pub mod ui;`**

Add after `pub mod theme;`:

```rust
pub mod ui;
```

- [ ] **Step 2: Verify compilation**

Run: `cargo check -p deer-gui 2>&1 | head -20`
Expected: Clean compilation (possibly with unused import warnings).

---

### Task 6: Write integration tests

**Files:**
- Create: `apps/deer_gui/tests/integration/fonts.rs`
- Modify: `apps/deer_gui/tests/integration.rs`

- [ ] **Step 1: Add module to `integration.rs`**

Add to the `mod integration` block:

```rust
pub mod fonts;
```

- [ ] **Step 2: Write the test file**

```rust
//! Integration tests for CSS font resolution pipeline.
//!
//! These tests verify parsing and builder logic using static CSS samples.
//! No network requests are made — network tests are gated behind the
//! `network-tests` feature.

use deer_gui::ui::fonts::{
    build_font_definitions, parse_font_css, FontFaceRule, FontStyle, ResolvedFont,
};

// ---------------------------------------------------------------------------
// Sample CSS (mirrors real Google Fonts response format)
// ---------------------------------------------------------------------------

const SAMPLE_CSS: &str = r#"
/* latin */
@font-face {
  font-family: 'IBM Plex Sans';
  font-style: normal;
  font-weight: 400;
  src: url(https://fonts.gstatic.com/s/ibmplexsans/v19/zYXgKVElMYYaJe8bpLHnCwDKhdzeFb5N.ttf) format('truetype');
}
/* latin */
@font-face {
  font-family: 'IBM Plex Sans';
  font-style: normal;
  font-weight: 700;
  src: url(https://fonts.gstatic.com/s/ibmplexsans/v19/zYX9KVElMYYaJe8bpLHnCwDKjWr7AIFsdA.ttf) format('truetype');
}
/* latin */
@font-face {
  font-family: 'IBM Plex Sans';
  font-style: italic;
  font-weight: 400;
  src: url(https://fonts.gstatic.com/s/ibmplexsans/v19/zYX-KVElMYYaJe8bpLHnCwDKhdTeEKxI.ttf) format('truetype');
}
"#;

const SAMPLE_CSS_MONO: &str = r#"
/* latin */
@font-face {
  font-family: 'JetBrains Mono';
  font-style: normal;
  font-weight: 400;
  src: url(https://fonts.gstatic.com/s/jetbrainsmono/v18/tDbY2o-flEEny0FZhsfKu5WU4zr3E_BX0PnT.ttf) format('truetype');
}
"#;

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[test]
fn t_font_01_parse_css_extracts_rules() {
    let rules = parse_font_css(SAMPLE_CSS);

    assert_eq!(rules.len(), 3, "should extract 3 @font-face rules");

    // First rule: IBM Plex Sans, 400, normal.
    assert_eq!(rules[0].family, "IBM Plex Sans");
    assert_eq!(rules[0].weight, 400);
    assert_eq!(rules[0].style, FontStyle::Normal);
    assert!(
        rules[0].url.contains("ibmplexsans"),
        "URL should reference ibmplexsans: {}",
        rules[0].url,
    );

    // Second rule: IBM Plex Sans, 700, normal.
    assert_eq!(rules[1].family, "IBM Plex Sans");
    assert_eq!(rules[1].weight, 700);
    assert_eq!(rules[1].style, FontStyle::Normal);

    // Third rule: IBM Plex Sans, 400, italic.
    assert_eq!(rules[2].family, "IBM Plex Sans");
    assert_eq!(rules[2].weight, 400);
    assert_eq!(rules[2].style, FontStyle::Italic);

    // Also test mono CSS.
    let mono_rules = parse_font_css(SAMPLE_CSS_MONO);
    assert_eq!(mono_rules.len(), 1);
    assert_eq!(mono_rules[0].family, "JetBrains Mono");
    assert_eq!(mono_rules[0].weight, 400);
}

#[test]
fn t_font_02_build_definitions() {
    // Create synthetic resolved fonts with minimal byte payloads.
    let fonts = vec![
        ResolvedFont {
            family: "TestFont".to_string(),
            weight: 400,
            style: FontStyle::Normal,
            // Minimal bytes — not a real font file, but enough to test the builder.
            bytes: vec![0u8; 64],
        },
        ResolvedFont {
            family: "TestFont".to_string(),
            weight: 700,
            style: FontStyle::Normal,
            bytes: vec![0u8; 64],
        },
    ];

    let defs = build_font_definitions(&fonts);

    // Should contain at least our 2 fonts plus any egui defaults.
    assert!(
        defs.font_data.len() >= 2,
        "font_data should contain at least 2 entries, got {}",
        defs.font_data.len(),
    );

    // Proportional family should reference our fonts.
    let proportional = defs
        .families
        .get(&bevy_egui::egui::FontFamily::Proportional)
        .expect("Proportional family should exist");
    assert!(
        proportional.len() >= 2,
        "Proportional family should have at least 2 entries, got {}",
        proportional.len(),
    );
}

#[test]
fn t_font_03_fallback_on_empty_fonts() {
    let defs = build_font_definitions(&[]);

    // Empty input should produce valid default FontDefinitions.
    // The default egui FontDefinitions always has at least the built-in font.
    assert!(
        !defs.font_data.is_empty(),
        "default FontDefinitions should have built-in font data",
    );

    let proportional = defs
        .families
        .get(&bevy_egui::egui::FontFamily::Proportional);
    assert!(
        proportional.is_some(),
        "default FontDefinitions should have Proportional family",
    );
}
```

- [ ] **Step 3: Run all tests**

Run: `cargo test -p deer-gui 2>&1`
Expected: All existing tests pass plus the 3 new font tests.

- [ ] **Step 4: Run clippy**

Run: `cargo clippy -p deer-gui -- -D warnings 2>&1`
Expected: No warnings.

- [ ] **Step 5: Commit**

```bash
git add apps/deer_gui/Cargo.toml \
       apps/deer_gui/src/constants.rs \
       apps/deer_gui/src/ui/mod.rs \
       apps/deer_gui/src/ui/fonts.rs \
       apps/deer_gui/src/lib.rs \
       apps/deer_gui/tests/integration/fonts.rs \
       apps/deer_gui/tests/integration.rs
git commit -m "feat(ui): add CSS font resolution pipeline for Google Fonts"
```
