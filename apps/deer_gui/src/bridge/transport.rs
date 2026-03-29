//! Transport abstraction for bridge communication.
//!
//! Decouples the bridge protocol from any specific transport mechanism
//! (subprocess stdio, gRPC, HTTP, test harness, etc.).

use std::io;

/// A bidirectional transport for sending and receiving line-delimited JSON.
///
/// Implementations handle the raw byte-level I/O; the protocol layer
/// (envelope parsing, command serialization) lives above this.
///
/// # `recv_line` semantics
///
/// Not all transports implement `recv_line`. Transports that use a
/// background reader thread (e.g. [`ProcessTransport`](super::client))
/// push events through an internal channel instead of exposing a
/// blocking `recv_line`. Such implementations return
/// `Err(io::ErrorKind::Unsupported)`. Callers should use the
/// [`BridgeEventReceiver`](super::plugin::BridgeEventReceiver) resource
/// to consume events rather than calling `recv_line` directly.
pub trait Transport: Send + Sync + 'static {
    /// Send a line of JSON to the remote end. Must append newline and flush.
    fn send_line(&self, line: &str) -> io::Result<()>;

    /// Receive the next line from the remote end (blocking).
    ///
    /// Returns `Ok(Some(line))` on success, `Ok(None)` when the transport
    /// is closed, or `Err(Unsupported)` if the transport uses a push-based
    /// model (see trait-level docs).
    fn recv_line(&self) -> io::Result<Option<String>>;
}
