//! Bridge module — communication layer between Bevy and the Python bridge process.
//!
//! Submodules:
//! - [`events`] — `BridgeEvent` / `BridgeResponse` enums
//! - [`commands`] — `OutboundCommand` / `InboundEnvelope` wire types
//! - [`client`] — `BridgeClient` subprocess manager
//! - [`plugin`] — Bevy plugin, resources, and systems

mod client;
mod commands;
mod dispatch;
mod events;
mod plugin;

pub use client::BridgeClient;
pub use events::{BridgeEvent, BridgeResponse};
pub use plugin::{BridgeClientResource, BridgeEventQueue, BridgePlugin};
