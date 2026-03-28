//! Runtime diagnostics collection for Deer GUI.
//!
//! Registers frame-time and entity-count diagnostics so that HUD panels
//! and log output can surface performance data without per-module wiring.

mod plugin;

pub use plugin::DiagnosticsPlugin;
