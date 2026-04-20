pub mod client;
pub mod process;
pub mod raw_capture;
pub mod types;

pub use client::{TuiGatewayClient, TuiGatewayError};
pub use process::{spawn_tui_gateway, TuiGatewayProcess};
pub use raw_capture::capture_gateway_stream;
pub use types::{
    TerminalColumnCount, TuiGatewayCommand, TuiGatewayProcessId, TuiGatewayRequestId,
    TuiGatewayRunId, TuiGatewaySessionId,
};
