"""
JSON-RPC 3.0 Transport Layer

Abstract transport interface and concrete implementations for
WebSocket and HTTP transports.
"""

from __future__ import annotations

from abc import ABC, abstractmethod
from typing import Any, AsyncIterator, Callable, Optional, Protocol, Union
import asyncio
import uuid

from .core import (
    ErrorCode,
    Message,
    Request,
    RequestId,
    Response,
    StreamResponse,
    error_response,
    parse_json,
    serialize,
    request as build_request,
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
        self._pending_requests: dict[str, asyncio.Future] = {}
        self._pending_streams: dict[str, asyncio.Queue] = {}
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

            # If it's a Response, fulfill future
            if isinstance(message, Response) and message.id:
                req_id = str(message.id.value)
                if req_id in self._pending_requests:
                    if not self._pending_requests[req_id].done():
                        self._pending_requests[req_id].set_result(message)
                    continue
                # For stream responses that are just Response (like error or final), we might put it in queue
                if req_id in self._pending_streams:
                    await self._pending_streams[req_id].put(message)
                    continue

            # If it's a StreamResponse, put in queue
            if isinstance(message, StreamResponse):
                req_id = str(message.stream_id)
                if req_id in self._pending_streams:
                    await self._pending_streams[req_id].put(message)
                    continue

            # Handle as standard message (request or notification)
            for handler in self._handlers:
                try:
                    response = await handler(message)
                    if response:
                        await self.transport.send(response)
                except Exception as e:
                    # Send error response
                    if isinstance(message, Request) and message.id:
                        err_res = error_response(
                            message.id,
                            ErrorCode.INTERNAL_ERROR,
                            f"Handler error: {e}",
                        )
                        await self.transport.send(err_res)

    async def stop(self) -> None:
        """Stop processing messages."""
        self._running = False
        await self.transport.disconnect()

    async def call(
        self,
        method: str,
        params: Union[dict[str, Any], list[Any], None] = None,
        options: Optional[dict[str, Any]] = None,
    ) -> Response:
        """Make a request and wait for response."""
        msg_id = str(uuid.uuid4())
        req_msg = build_request(method=method, params=params, msg_id=msg_id, options=options)

        loop = asyncio.get_running_loop()
        future = loop.create_future()
        self._pending_requests[msg_id] = future

        await self.transport.send(req_msg)

        try:
            return await future
        finally:
            if msg_id in self._pending_requests:
                del self._pending_requests[msg_id]

    async def notify(
        self,
        method: str,
        params: Union[dict[str, Any], list[Any], None] = None,
    ) -> None:
        """Send a notification (no response expected)."""
        req_msg = build_request(method=method, params=params)
        await self.transport.send(req_msg)

    async def stream(
        self,
        method: str,
        params: Union[dict[str, Any], list[Any], None] = None,
        options: Optional[dict[str, Any]] = None,
    ) -> AsyncIterator[StreamResponse]:
        """Make a streaming request and yield chunks."""
        msg_id = str(uuid.uuid4())
        req_msg = build_request(method=method, params=params, msg_id=msg_id, options=options)

        queue: asyncio.Queue = asyncio.Queue()
        self._pending_streams[msg_id] = queue

        await self.transport.send(req_msg)

        try:
            while True:
                message = await queue.get()
                if isinstance(message, StreamResponse):
                    yield message
                    if message.is_done or message.is_error:
                        break
                elif isinstance(message, Response):
                    break
        finally:
            if msg_id in self._pending_streams:
                del self._pending_streams[msg_id]

    async def cancel(self, request_id: Union[RequestId, str, int]) -> None:
        """Cancel a pending request."""
        req_id_str = str(request_id.value) if isinstance(request_id, RequestId) else str(request_id)

        cancel_msg = build_request(
            method="request.cancel", params={"stream": True, "id": req_id_str}
        )

        await self.transport.send(cancel_msg)
        if req_id_str in self._pending_requests:
            del self._pending_requests[req_id_str]


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
                        yield error_response(
                            None,
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
