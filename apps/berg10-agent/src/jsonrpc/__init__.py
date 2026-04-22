"""
JSON-RPC 3.0 - A Complete, Standalone Implementation

This module provides a full implementation of JSON-RPC 3.0 with support for:
- Standard request/response
- Bidirectional streaming
- Cancellation
- Batch operations
- Multiple transports (WebSocket, HTTP, in-memory)

Example Usage:
    # Server
    from berg10_agent.jsonrpc import Server

    server = Server()

    @server.method("echo")
    async def echo(message: str) -> str:
        return message

    @server.method("stream_count", streaming=True)
    async def stream_count(n: int, _emit, _request_id):
        from berg10_agent.jsonrpc import stream_data
        for i in range(n):
            await _emit(stream_data(_request_id, str(i)))

    # Client
    from berg10_agent.jsonrpc import Client, WebSocketTransport

    transport = WebSocketTransport("ws://localhost:8765")
    client = Client(transport)
    await client.connect()

    # Regular call
    response = await client.call("echo", {"message": "Hello"})
    print(response.result)  # "Hello"

    # Streaming call
    async for chunk in client.stream("stream_count", {"n": 5}):
        print(chunk.data)
"""

# Core types and functions
from .core import (
    VERSION,
    ErrorCode,
    JsonRpcError,
    Message,
    ParseError,
    Request,
    RequestId,
    Response,
    StreamResponse,
    batch,
    cancelled,
    error_response,
    internal_error,
    invalid_params,
    invalid_request,
    method_not_found,
    parse_error,
    parse_json,
    parse_message,
    request,
    serialize,
    serialize_batch,
    stream_data,
    stream_done,
    stream_error,
    success_response,
    ack_response,
    validate_request,
    validate_response,
)

# Handler registry
from .handler import (
    CancellationError,
    CancellableRequest,
    Handler,
    HandlerInfo,
    Middleware,
    Registry,
    StreamingDispatcher,
    logging_middleware,
    validation_middleware,
)

# Server and Client
from .server import Client, Server

# Transport (may be None if dependencies not installed)
try:
    from .transport import (
        Connection,
        HTTPTransport,
        InMemoryTransport,
        MessageHandler,
        Transport,
        WebSocketTransport,
    )
except ImportError:
    # Some transports may not be available without optional deps
    Transport = None  # type: ignore
    Connection = None  # type: ignore
    InMemoryTransport = None  # type: ignore
    MessageHandler = None  # type: ignore
    WebSocketTransport = None  # type: ignore
    HTTPTransport = None  # type: ignore

__version__ = "3.0.0"
__all__ = [
    # Version
    "VERSION",
    # Core Types
    "RequestId",
    "JsonRpcError",
    "Request",
    "Response",
    "StreamResponse",
    "Message",
    # Enums
    "ErrorCode",
    # Exceptions
    "ParseError",
    "CancellationError",
    # Core Functions
    "request",
    "success_response",
    "error_response",
    "ack_response",
    "stream_data",
    "stream_done",
    "stream_error",
    "parse_json",
    "parse_message",
    "serialize",
    "serialize_batch",
    "batch",
    "validate_request",
    "validate_response",
    # Error Helpers
    "parse_error",
    "invalid_request",
    "method_not_found",
    "invalid_params",
    "internal_error",
    "cancelled",
    # Handler
    "Handler",
    "HandlerInfo",
    "Middleware",
    "Registry",
    "StreamingDispatcher",
    "CancellableRequest",
    "logging_middleware",
    "validation_middleware",
    # Server/Client
    "Server",
    "Client",
    # Transport
    "Transport",
    "Connection",
    "InMemoryTransport",
    "MessageHandler",
    "WebSocketTransport",
    "HTTPTransport",
]
