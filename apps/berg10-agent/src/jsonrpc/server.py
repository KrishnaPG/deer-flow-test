"""
JSON-RPC 3.0 Server

High-level server implementation with handler registry, streaming support,
and connection management.
"""

from __future__ import annotations

from typing import Any, AsyncIterator, Callable, Optional

from .core import (
    Cancel,
    ErrorCode,
    Message,
    Progress,
    Request,
    RequestId,
    Response,
    StreamChunk,
    StreamEnd,
    StreamStart,
    parse_json,
    serialize,
)
from .handler import CancellableRequest, Registry, StreamingDispatcher
from .transport import Connection, Transport


# =============================================================================
# Server
# =============================================================================


class Server:
    """JSON-RPC 3.0 Server with streaming and cancellation support."""

    def __init__(self):
        self.registry = Registry()
        self.streaming = StreamingDispatcher(self.registry)
        self._connections: list[Connection] = []
        self._cancellable_requests: dict[str, CancellableRequest] = {}

    # -------------------------------------------------------------------------
    # Handler Registration
    # -------------------------------------------------------------------------

    def method(
        self,
        name: Optional[str] = None,
        streaming: bool = False,
        stream_types: Optional[list[str]] = None,
    ) -> Callable[..., Any]:
        """Decorator to register a method handler.

        Usage:
            @server.method("echo")
            async def echo(message: str) -> str:
                return message

            @server.method("stream_chat", streaming=True)
            async def stream_chat(prompt: str, _emit, _request_id):
                for chunk in generate_chunks(prompt):
                    await _emit(StreamChunk(_request_id, chunk))
        """
        from .core import StreamType

        types = [StreamType(t) for t in stream_types] if stream_types else None
        return self.registry.method(name, streaming, types)

    def register(
        self,
        name: str,
        handler: Callable[..., Any],
        streaming: bool = False,
    ) -> None:
        """Register a handler programmatically."""
        self.registry.register(name, handler, streaming)

    # -------------------------------------------------------------------------
    # Connection Management
    # -------------------------------------------------------------------------

    def add_connection(self, transport: Transport) -> Connection:
        """Add a new connection to the server."""
        connection = Connection(transport)
        connection.add_handler(self._handle_message)
        self._connections.append(connection)
        return connection

    async def remove_connection(self, connection: Connection) -> None:
        """Remove a connection from the server."""
        if connection in self._connections:
            await connection.stop()
            self._connections.remove(connection)

    async def start(self) -> None:
        """Start all connections."""
        import asyncio

        await asyncio.gather(
            *[conn.start() for conn in self._connections],
            return_exceptions=True,
        )

    async def stop(self) -> None:
        """Stop all connections."""
        for conn in self._connections:
            await conn.stop()
        self._connections.clear()

    # -------------------------------------------------------------------------
    # Message Handling
    # -------------------------------------------------------------------------

    async def _handle_message(self, message: Message) -> Optional[Response]:
        """Handle incoming messages."""
        # Handle cancellation
        if isinstance(message, Cancel):
            return await self._handle_cancel(message)

        # Handle requests
        if isinstance(message, Request):
            return await self._handle_request(message)

        # Responses are handled by the connection layer
        return None

    async def _handle_request(self, request: Request) -> Response:
        """Handle a request, supporting streaming if applicable."""
        handler_info = self.registry.get_handler(request.method)

        if not handler_info:
            return Response.error(
                request.id,
                ErrorCode.METHOD_NOT_FOUND,
                f"Method not found: {request.method}",
            )

        # Track cancellable request
        cancellable = CancellableRequest(request)
        self._cancellable_requests[str(request.id)] = cancellable

        try:
            if handler_info.is_streaming:
                # Streaming not supported in standard request/response
                # Client should use streaming endpoint
                return Response.error(
                    request.id,
                    ErrorCode.METHOD_NOT_FOUND,
                    f"Method {request.method} requires streaming",
                )

            # Check for cancellation before executing
            cancellable.check_cancelled()

            # Execute handler
            return await self.registry.dispatch(request)

        except Exception as e:
            return Response.error(
                request.id,
                ErrorCode.INTERNAL_ERROR,
                str(e),
            )
        finally:
            if str(request.id) in self._cancellable_requests:
                del self._cancellable_requests[str(request.id)]

    async def _handle_cancel(self, cancel: Cancel) -> Optional[Response]:
        """Handle cancellation request."""
        request_id = str(cancel.request_id)

        if request_id in self._cancellable_requests:
            self._cancellable_requests[request_id].cancel(cancel.reason)
            self.streaming.cancel_stream(cancel.request_id)
            return Response.success(
                cancel.request_id,
                {"cancelled": True, "reason": cancel.reason},
            )

        return Response.error(
            cancel.request_id,
            ErrorCode.INVALID_REQUEST,
            "Request not found or already completed",
        )

    # -------------------------------------------------------------------------
    # Streaming Support
    # -------------------------------------------------------------------------

    async def handle_stream(
        self,
        request: Request,
        emit: Callable[[StreamChunk | StreamEnd | Progress], Any],
    ) -> Optional[Response]:
        """Handle a streaming request.

        This is typically called from a WebSocket endpoint where
        the emit function sends chunks to the client.
        """
        # Track cancellable request
        cancellable = CancellableRequest(request)
        self._cancellable_requests[str(request.id)] = cancellable

        async def wrapped_emit(
            chunk: StreamChunk | StreamEnd | Progress,
        ) -> None:
            """Emit wrapper that checks for cancellation."""
            cancellable.check_cancelled()
            await emit(chunk)

        try:
            return await self.streaming.dispatch_stream(request, wrapped_emit)
        except Exception as e:
            return Response.error(
                request.id,
                ErrorCode.INTERNAL_ERROR,
                str(e),
            )
        finally:
            if str(request.id) in self._cancellable_requests:
                del self._cancellable_requests[str(request.id)]


# =============================================================================
# Client
# =============================================================================


class Client:
    """JSON-RPC 3.0 Client with streaming support."""

    def __init__(self, connection: Connection):
        self.connection = connection

    async def connect(self) -> None:
        """Connect to the server."""
        await self.connection.start()

    async def disconnect(self) -> None:
        """Disconnect from the server."""
        await self.connection.stop()

    async def call(
        self,
        method: str,
        params: Optional[dict[str, Any]] = None,
    ) -> Response:
        """Make a synchronous call."""
        return await self.connection.call(method, params)

    async def notify(
        self,
        method: str,
        params: Optional[dict[str, Any]] = None,
    ) -> None:
        """Send a notification (no response expected)."""
        await self.connection.notify(method, params)

    async def stream(
        self,
        method: str,
        params: Optional[dict[str, Any]] = None,
    ) -> AsyncIterator[StreamChunk]:
        """Make a streaming call."""
        async for chunk in self.connection.stream(method, params):
            yield chunk

    async def cancel(
        self,
        request_id: RequestId,
        reason: Optional[str] = None,
    ) -> Response:
        """Cancel a pending request."""
        await self.connection.cancel(request_id, reason)
        # The server will send a response
        return Response.success(request_id, {"cancelled": True})
