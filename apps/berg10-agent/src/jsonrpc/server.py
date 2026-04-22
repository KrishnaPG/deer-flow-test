"""
JSON-RPC 3.0 Server

High-level server implementation with handler registry, streaming support,
and connection management.
"""

from __future__ import annotations

from typing import Any, AsyncIterator, Callable, Optional, Union

from .core import (
    ErrorCode,
    Message,
    Request,
    RequestId,
    Response,
    StreamResponse,
    error_response,
    success_response,
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
    ) -> Callable[..., Any]:
        """Decorator to register a method handler."""
        return self.registry.method(name, streaming)

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

        async def connection_handler(message: Message) -> Optional[Response]:
            if isinstance(message, Request):
                if message.method == "request.cancel":
                    return await self._handle_cancel(message)
                if message.options and message.options.get("abort"):
                    return await self._handle_cancel_options(message)

                handler_info = self.registry.get_handler(message.method)
                if handler_info and handler_info.is_streaming:
                    # Route to stream handler
                    async def emit(chunk: Union[StreamResponse, Response]):
                        await connection.transport.send(chunk)

                    return await self.handle_stream(message, emit)

                return await self._handle_request(message)
            return None

        connection.add_handler(connection_handler)
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
        """Handle incoming messages (used when not bound to a specific connection)."""
        if isinstance(message, Request):
            # Check for cancellation
            if message.method == "request.cancel":
                return await self._handle_cancel(message)
            if message.options and message.options.get("abort"):
                return await self._handle_cancel_options(message)

            return await self._handle_request(message)

        # Responses are handled by the connection layer
        return None

    async def _handle_request(self, request: Request) -> Response:
        """Handle a request, supporting streaming if applicable."""
        handler_info = self.registry.get_handler(request.method)

        if not handler_info:
            return error_response(
                request.id,
                ErrorCode.METHOD_NOT_FOUND,
                f"Method not found: {request.method}",
            )

        # Track cancellable request
        cancellable = CancellableRequest(request)
        if request.id is not None:
            self._cancellable_requests[str(request.id.value)] = cancellable

        try:
            # Check for cancellation before executing
            cancellable.check_cancelled()

            # Execute handler
            return await self.registry.dispatch(request)

        except Exception as e:
            return error_response(
                request.id,
                ErrorCode.INTERNAL_ERROR,
                str(e),
            )
        finally:
            if request.id is not None and str(request.id.value) in self._cancellable_requests:
                del self._cancellable_requests[str(request.id.value)]

    async def _handle_cancel(self, request: Request) -> Optional[Response]:
        """Handle standard request.cancel method."""
        params = request.params
        if not isinstance(params, dict) or "id" not in params:
            return error_response(
                request.id,
                ErrorCode.INVALID_PARAMS,
                "Cancel request must include 'id' in params",
            )

        target_id = str(params["id"])
        return await self._execute_cancel(target_id, request.id)

    async def _handle_cancel_options(self, request: Request) -> Optional[Response]:
        """Handle cancellation via options.abort."""
        target_id = str(request.options["stream"]) if "stream" in request.options else None
        if not target_id and request.id:
            target_id = str(request.id.value)

        if target_id:
            return await self._execute_cancel(target_id, request.id)

        return error_response(
            request.id,
            ErrorCode.INVALID_REQUEST,
            "Cannot determine target for cancellation",
        )

    async def _execute_cancel(self, target_id: str, request_id: Optional[RequestId]) -> Response:
        """Execute the cancellation on tracked requests and streams."""
        cancelled = False

        if target_id in self._cancellable_requests:
            self._cancellable_requests[target_id].cancel("Request cancelled by client")
            cancelled = True

        if self.streaming.cancel_stream(target_id):
            cancelled = True

        if cancelled:
            return success_response(request_id, {"cancelled": True})

        return error_response(
            request_id,
            ErrorCode.INVALID_REQUEST,
            "Request not found or already completed",
        )

    # -------------------------------------------------------------------------
    # Streaming Support
    # -------------------------------------------------------------------------

    async def handle_stream(
        self,
        request: Request,
        emit: Callable[[Union[StreamResponse, Response]], Any],
    ) -> Optional[Response]:
        """Handle a streaming request.

        This is typically called from a WebSocket endpoint where
        the emit function sends chunks to the client.
        """
        # Track cancellable request
        cancellable = CancellableRequest(request)
        req_id_str = str(request.id.value) if request.id else str(uuid4())
        self._cancellable_requests[req_id_str] = cancellable

        async def wrapped_emit(
            chunk: Union[StreamResponse, Response],
        ) -> None:
            """Emit wrapper that checks for cancellation."""
            cancellable.check_cancelled()
            await emit(chunk)

        try:
            return await self.streaming.dispatch_stream(request, wrapped_emit)
        except Exception as e:
            return error_response(
                request.id,
                ErrorCode.INTERNAL_ERROR,
                str(e),
            )
        finally:
            if req_id_str in self._cancellable_requests:
                del self._cancellable_requests[req_id_str]


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
        options: Optional[dict[str, Any]] = None,
    ) -> Response:
        """Make a synchronous call."""
        return await self.connection.call(method, params, options)

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
        options: Optional[dict[str, Any]] = None,
    ) -> AsyncIterator[StreamResponse]:
        """Make a streaming call."""
        stream_opts = options or {}
        stream_opts["stream"] = True
        async for chunk in self.connection.stream(method, params, stream_opts):
            yield chunk

    async def cancel(
        self,
        request_id: Union[RequestId, str, int],
    ) -> Response:
        """Cancel a pending request."""
        await self.connection.cancel(request_id)
        # The server will send a response
        return success_response(request_id, {"cancelled": True})
