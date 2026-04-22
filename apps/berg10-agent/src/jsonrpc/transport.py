"""
JSON-RPC 3.0 Transport Layer

Abstract transport interface and concrete implementations for
WebSocket and HTTP transports.
"""

from __future__ import annotations

from abc import ABC, abstractmethod
from typing import Any, AsyncIterator, Callable, Optional, Protocol

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


# =============================================================================
# Transport Interface
# =============================================================================


class Transport(ABC):
    """Abstract base class for JSON-RPC transports."""

    @abstractmethod
    async def connect(self) -> None:
        """Establish connection."""
        pass

    @abstractmethod
    async def disconnect(self) -> None:
        """Close connection."""
        pass

    @abstractmethod
    async def send(self, message: Message) -> None:
        """Send a message."""
        pass

    @abstractmethod
    async def receive(self) -> AsyncIterator[Message]:
        """Receive messages (async iterator)."""
        pass

    @property
    @abstractmethod
    def is_connected(self) -> bool:
        """Check if transport is connected."""
        pass


class MessageHandler(Protocol):
    """Protocol for message handlers."""

    async def __call__(self, message: Message) -> Optional[Response]:
        """Handle a message and optionally return a response."""
        ...


# =============================================================================
# In-Memory Transport (for testing)
# =============================================================================


class InMemoryTransport(Transport):
    """In-memory transport for testing."""

    def __init__(self):
        self._connected = False
        self._send_queue: list[Message] = []
        self._receive_queue: list[Message] = []
        self._paired: Optional[InMemoryTransport] = None

    def pair_with(self, other: InMemoryTransport) -> None:
        """Pair this transport with another for bidirectional communication."""
        self._paired = other
        other._paired = self

    async def connect(self) -> None:
        self._connected = True

    async def disconnect(self) -> None:
        self._connected = False

    async def send(self, message: Message) -> None:
        if not self._connected:
            raise ConnectionError("Transport not connected")
        if self._paired:
            self._paired._receive_queue.append(message)
        else:
            self._send_queue.append(message)

    async def receive(self) -> AsyncIterator[Message]:
        while self._connected:
            if self._receive_queue:
                yield self._receive_queue.pop(0)
            else:
                import asyncio

                await asyncio.sleep(0.01)

    @property
    def is_connected(self) -> bool:
        return self._connected


# =============================================================================
# Connection Manager
# =============================================================================


class Connection:
    """Manages a single JSON-RPC connection with request tracking."""

    def __init__(self, transport: Transport):
        self.transport = transport
        self._pending_requests: dict[str, RequestId] = {}
        self._handlers: list[MessageHandler] = []
        self._running = False

    def add_handler(self, handler: MessageHandler) -> None:
        """Add a message handler."""
        self._handlers.append(handler)

    def remove_handler(self, handler: MessageHandler) -> None:
        """Remove a message handler."""
        if handler in self._handlers:
            self._handlers.remove(handler)

    async def start(self) -> None:
        """Start processing messages."""
        await self.transport.connect()
        self._running = True

        async for message in self.transport.receive():
            if not self._running:
                break

            # Handle the message
            for handler in self._handlers:
                try:
                    response = await handler(message)
                    if response:
                        await self.transport.send(response)
                except Exception as e:
                    # Send error response
                    if isinstance(message, Request):
                        error_response = Response.error(
                            message.id,
                            ErrorCode.INTERNAL_ERROR,
                            f"Handler error: {e}",
                        )
                        await self.transport.send(error_response)

    async def stop(self) -> None:
        """Stop processing messages."""
        self._running = False
        await self.transport.disconnect()

    async def call(
        self,
        method: str,
        params: Optional[dict[str, Any]] = None,
    ) -> Response:
        """Make a request and wait for response."""
        request = Request(method=method, params=params or {})
        self._pending_requests[str(request.id)] = request.id

        await self.transport.send(request)

        # Wait for response with matching ID
        async for message in self.transport.receive():
            if isinstance(message, Response) and str(message.id) == str(request.id):
                del self._pending_requests[str(request.id)]
                return message

        raise ConnectionError("Connection closed before response received")

    async def notify(
        self,
        method: str,
        params: Optional[dict[str, Any]] = None,
    ) -> None:
        """Send a notification (no response expected)."""
        request = Request(method=method, params=params or {})
        await self.transport.send(request)

    async def stream(
        self,
        method: str,
        params: Optional[dict[str, Any]] = None,
    ) -> AsyncIterator[StreamChunk]:
        """Make a streaming request and yield chunks."""
        request = Request(method=method, params=params or {})
        await self.transport.send(request)

        async for message in self.transport.receive():
            if isinstance(message, StreamChunk):
                yield message
            elif isinstance(message, StreamEnd):
                if str(message.request_id) == str(request.id):
                    break
            elif isinstance(message, Response):
                if str(message.id) == str(request.id):
                    # Non-streaming response
                    break

    async def cancel(self, request_id: RequestId, reason: Optional[str] = None) -> None:
        """Cancel a pending request."""
        cancel_msg = Cancel(request_id=request_id, reason=reason)
        await self.transport.send(cancel_msg)
        if str(request_id) in self._pending_requests:
            del self._pending_requests[str(request_id)]


# =============================================================================
# WebSocket Transport (using websockets library)
# =============================================================================


try:
    import websockets
    from websockets.exceptions import ConnectionClosed

    class WebSocketTransport(Transport):
        """WebSocket transport implementation."""

        def __init__(self, uri: str, headers: Optional[dict[str, str]] = None):
            self.uri = uri
            self.headers = headers or {}
            self._ws: Optional[websockets.WebSocketClientProtocol] = None

        async def connect(self) -> None:
            self._ws = await websockets.connect(self.uri, extra_headers=self.headers)

        async def disconnect(self) -> None:
            if self._ws:
                await self._ws.close()
                self._ws = None

        async def send(self, message: Message) -> None:
            if not self._ws:
                raise ConnectionError("Not connected")
            await self._ws.send(serialize(message))

        async def receive(self) -> AsyncIterator[Message]:
            if not self._ws:
                raise ConnectionError("Not connected")

            try:
                async for raw_message in self._ws:
                    try:
                        yield parse_json(raw_message)
                    except Exception as e:
                        # Yield error as response
                        yield Response.error(
                            RequestId(),
                            ErrorCode.PARSE_ERROR,
                            f"Failed to parse message: {e}",
                        )
            except ConnectionClosed:
                pass

        @property
        def is_connected(self) -> bool:
            return self._ws is not None and self._ws.open

except ImportError:
    WebSocketTransport = None  # type: ignore


# =============================================================================
# HTTP Transport
# =============================================================================


try:
    import httpx

    class HTTPTransport(Transport):
        """HTTP POST transport for JSON-RPC."""

        def __init__(
            self,
            endpoint: str,
            headers: Optional[dict[str, str]] = None,
            timeout: float = 30.0,
        ):
            self.endpoint = endpoint
            self.headers = headers or {}
            self.timeout = timeout
            self._client: Optional[httpx.AsyncClient] = None

        async def connect(self) -> None:
            self._client = httpx.AsyncClient(
                headers=self.headers,
                timeout=self.timeout,
            )

        async def disconnect(self) -> None:
            if self._client:
                await self._client.aclose()
                self._client = None

        async def send(self, message: Message) -> None:
            """HTTP transport doesn't support async send/receive pattern."""
            raise NotImplementedError("HTTPTransport uses call() method instead of send/receive")

        async def receive(self) -> AsyncIterator[Message]:
            """HTTP transport doesn't support streaming receive."""
            raise NotImplementedError("HTTPTransport uses call() method instead of send/receive")

        async def call(self, request: Request) -> Response:
            """Make a synchronous HTTP call."""
            if not self._client:
                raise ConnectionError("Not connected")

            response = await self._client.post(
                self.endpoint,
                json=request.to_dict(),
            )
            response.raise_for_status()
            return parse_json(response.text)  # type: ignore

        @property
        def is_connected(self) -> bool:
            return self._client is not None

except ImportError:
    HTTPTransport = None  # type: ignore
