//! HTTP fetching for font files and CSS.

use crate::constants::fonts::FONT_CSS_USER_AGENT;
use bevy::log::{debug, trace};
use std::io::Read;

use super::types::FontLoadError;

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

/// Check if a URL is reachable (for testing connectivity).
///
/// # Arguments
///
/// * `url` - The URL to check.
///
/// # Returns
///
/// `true` if the URL returns a 200 status, `false` otherwise.
pub fn is_url_reachable(url: &str) -> bool {
    ureq::head(url)
        .header("User-Agent", FONT_CSS_USER_AGENT)
        .call()
        .map(|r| r.status() == 200)
        .unwrap_or(false)
}
