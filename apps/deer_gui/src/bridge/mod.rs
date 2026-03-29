//! Bridge module — communication layer between Bevy and the Python bridge process.
//!
//! Submodules:
//! - [`events`] — `BridgeEvent` / `BridgeResponse` enums
//! - [`commands`] — `OutboundCommand` / `InboundEnvelope` wire types
//! - [`adapter`] — centralized JSON→Domain parsing
//! - [`client`] — `BridgeClient` subprocess manager
//! - [`orchestrator`] — `OrchestratorClient` trait
//! - [`transport`] — `Transport` trait
//! - [`payloads`] — Typed outbound command payloads (zero-allocation)
//! - [`plugin`] — Bevy plugin, resources, and systems

mod adapter;
mod client;
mod commands;
mod dispatch;
mod events;
mod orchestrator;
mod payloads;
mod plugin;
mod transport;

pub use client::BridgeClient;
pub use commands::InboundEnvelope;
pub use dispatch::dispatch_envelope;
pub use events::{BridgeEvent, BridgeResponse};
pub use orchestrator::OrchestratorClient;
pub use plugin::{BridgeClientResource, BridgeEventQueue, BridgeEventReceiver, BridgePlugin};
pub use transport::Transport;
