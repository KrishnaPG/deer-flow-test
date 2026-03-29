# PR G — CSS Font Loading from Google Fonts

**Parent:** [00-index.md](./00-index.md)
**Status:** Approved
**Depends on:** None (fully independent)

---

## Problem

The application needs custom typography (IBM Plex Sans + JetBrains Mono).
The user requires CSS @font-face standards: accept Google Fonts CSS URLs,
resolve concrete font file URLs, fetch bytes at runtime. No bundling, no
custom URI schemes.

## Design

### Font Resolution Pipeline

```
Google Fonts CSS URL (e.g. "https://fonts.googleapis.com/css2?family=IBM+Plex+Sans:wght@400;500;700")
  |
  v
1. HTTP GET the CSS URL (with browser-like User-Agent to get .woff2/.ttf URLs)
  |
  v
2. Parse @font-face rules, extract font-file URLs + metadata (family, weight, style)
  |
  v
3. HTTP GET each font-file URL, receive raw bytes
  |
  v
4. Register font bytes with egui FontDefinitions
  |
  v
5. Apply to egui context via ctx.set_fonts()
```

### Module: `src/ui/fonts.rs` (~150 LOC)

```rust
/// A resolved font face with metadata and raw bytes.
#[derive(Debug)]
pub struct ResolvedFont {
    pub family: String,
    pub weight: u16,
    pub style: FontStyle,
    pub bytes: Vec<u8>,
}

/// Parse a Google Fonts CSS response and extract font-file URLs.
pub fn parse_font_css(css: &str) -> Vec<FontFaceRule> { … }

/// Fetch font bytes from a resolved URL.
/// Uses `ureq` or Bevy's async asset loading.
pub async fn fetch_font_bytes(url: &str) -> Result<Vec<u8>, FontLoadError> { … }

/// Build egui FontDefinitions from a set of resolved fonts.
pub fn build_font_definitions(fonts: &[ResolvedFont]) -> egui::FontDefinitions { … }

/// High-level: given a Google Fonts CSS URL, resolve and fetch all fonts.
pub async fn load_fonts_from_css_url(url: &str) -> Result<Vec<ResolvedFont>, FontLoadError> { … }
```

### Module: `src/ui/mod.rs`

```rust
pub mod fonts;
```

### Fallback Behaviour

If font loading fails (no network, bad URL), the system falls back to
Bevy's `default_font` feature (already enabled in Cargo.toml). A warning
is logged but the application continues.

### Theme Integration

The `ThemeDescriptor` has an optional `font_css_url` field. When present,
`ThemePlugin` loads fonts from that URL on theme activation. This ties
font choice to the active theme — different themes can use different
typography.

### HTTP Client

Add `ureq` to Cargo.toml for synchronous HTTP requests (font loading
happens once at startup, not per-frame):

```toml
ureq = "3"
```

`ureq` is minimal, no-async, and well-tested. The font CSS fetch and
file downloads happen in a blocking Bevy startup system.

### CSS Parsing

The Google Fonts CSS response is simple enough that a regex-based parser
suffices. No full CSS parser dependency needed. The format is:

```css
@font-face {
  font-family: 'IBM Plex Sans';
  font-style: normal;
  font-weight: 400;
  src: url(https://fonts.gstatic.com/s/...) format('woff2');
}
```

Extract: `font-family`, `font-weight`, `font-style`, `src url(…)`.

## Constants

Added to `constants.rs`:

```rust
pub mod fonts {
    /// Default Google Fonts CSS URL for body text.
    pub const DEFAULT_BODY_FONT_CSS: &str =
        "https://fonts.googleapis.com/css2?family=IBM+Plex+Sans:wght@400;500;700&display=swap";
    /// Default Google Fonts CSS URL for monospace/code text.
    pub const DEFAULT_MONO_FONT_CSS: &str =
        "https://fonts.googleapis.com/css2?family=JetBrains+Mono:wght@400&display=swap";
    /// User-Agent string for Google Fonts CSS requests (to get .ttf URLs).
    pub const FONT_CSS_USER_AGENT: &str =
        "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36";
}
```

## Tests

New file: `tests/integration/fonts.rs`

| Test                                  | Verifies                                                  |
| ------------------------------------- | --------------------------------------------------------- |
| `t_font_01_parse_css_extracts_rules`  | `parse_font_css` extracts family, weight, URL from sample CSS |
| `t_font_02_build_definitions`         | `build_font_definitions` produces non-empty FontDefinitions   |
| `t_font_03_fallback_on_empty_fonts`   | Empty font list produces valid (default) FontDefinitions      |

Note: Network-dependent tests (actual Google Fonts fetch) are gated
behind `#[cfg(feature = "network-tests")]` and not run in CI.
