"""
JSON-RPC 3.0 Handler Registry and Dispatcher

Provides method registration, middleware support, and request dispatch.
"""

from __future__ import annotations

import inspect
from dataclasses import dataclass
from typing import Any, Awaitable, Callable, Optional, TypeVar

from .core import (
    Cancel,
    ErrorCode,
    JsonRpcError,
    Message,
    Progress,
    Request,
    RequestId,
    Response,
    StreamChunk,
    StreamEnd,
    StreamStart,
    StreamType,
    cancelled,
    invalid_params,
    method_not_found,
)


# =============================================================================
# Handler Types
# =============================================================================


T = TypeVar("T")

# Simple handler: returns result directly
SyncHandler = Callable[..., Any]
AsyncHandler = Callable[..., Awaitable[Any]]
Handler = SyncHandler | AsyncHandler

# Streaming handler: yields chunks
AsyncStreamHandler = Callable[..., Awaitable[Any]]

# Middleware type
Middleware = Callable[[Request, Callable[[Request], Awaitable[Response]]], Awaitable[Response]]


# =============================================================================
# Handler Registration
# =============================================================================


@dataclass
class HandlerInfo:
    """Information about a registered handler."""

    name: str
    handler: Handler
    is_streaming: bool = False
    streaming_types: list[StreamType] = None  # type: ignore

    def __post_init__(self):
        if self.streaming_types is None:
            self.streaming_types = [StreamType.CONTENT]


class Registry:
    """Registry for JSON-RPC method handlers."""

    def __init__(self):
        self._handlers: dict[str, HandlerInfo] = {}
        self._middleware: list[Middleware] = []

    def register(
        self,
        name: str,
        handler: Handler,
        streaming: bool = False,
        stream_types: Optional[list[StreamType]] = None,
    ) -> None:
        """Register a method handler.

        Args:
            name: Method name
            handler: Function to handle the method
            streaming: Whether this handler supports streaming
            stream_types: Types of streams this handler produces
        """
        self._handlers[name] = HandlerInfo(
            name=name,
            handler=handler,
            is_streaming=streaming,
            streaming_types=list(stream_types) if stream_types else [StreamType.CONTENT],
        )

    def unregister(self, name: str) -> None:
        """Unregister a method handler."""
        if name in self._handlers:
            del self._handlers[name]

    def method(
        self,
        name: Optional[str] = None,
        streaming: bool = False,
        stream_types: Optional[list[StreamType]] = None,
    ) -> Callable[[Handler], Handler]:
        """Decorator to register a method handler.

        Usage:
            @registry.method("echo")
            async def echo_handler(params):
                return params.get("message")

            @registry.method("stream_data", streaming=True)
            async def stream_handler(params, emit):
                for i in range(10):
                    await emit(StreamChunk(...))
        """

        def decorator(handler: Handler) -> Handler:
            method_name = name or handler.__name__
            self.register(method_name, handler, streaming, stream_types)
            return handler

        return decorator

    def add_middleware(self, middleware: Middleware) -> None:
        """Add middleware to the handler chain."""
        self._middleware.append(middleware)

    def get_handler(self, name: str) -> Optional[HandlerInfo]:
        """Get handler info by method name."""
        return self._handlers.get(name)

    def list_methods(self) -> list[str]:
        """List all registered method names."""
        return list(self._handlers.keys())

    async def dispatch(self, request: Request) -> Response:
        """Dispatch a request to the appropriate handler.

        Args:
            request: The JSON-RPC request

        Returns:
            Response from the handler
        """
        handler_info = self._handlers.get(request.method)

        if not handler_info:
            return Response.error(
                request.id,
                ErrorCode.METHOD_NOT_FOUND,
                f"Method not found: {request.method}",
            )

        # Build handler chain with middleware
        async def execute(req: Request) -> Response:
            return await self._execute_handler(handler_info, req)

        handler = execute
        for mw in reversed(self._middleware):
            next_handler = handler
            handler = lambda req, mw=mw, nh=next_handler: mw(req, nh)  # type: ignore

        try:
            return await handler(request)
        except Exception as e:
            return Response.error(
                request.id,
                ErrorCode.INTERNAL_ERROR,
                str(e),
            )

    async def _execute_handler(
        self,
        handler_info: HandlerInfo,
        request: Request,
    ) -> Response:
        """Execute a handler with parameter validation."""
        handler = handler_info.handler
        params = request.params

        try:
            # Check if handler is async
            if inspect.iscoroutinefunction(handler):
                result = await handler(**params)
            else:
                result = handler(**params)

            return Response.success(request.id, result)

        except TypeError as e:
            # Parameter mismatch
            return Response.error(
                request.id,
                ErrorCode.INVALID_PARAMS,
                f"Invalid parameters: {e}",
            )
        except Exception as e:
            return Response.error(
                request.id,
                ErrorCode.INTERNAL_ERROR,
                str(e),
            )


# =============================================================================
# Streaming Dispatcher
# =============================================================================


class StreamingDispatcher:
    """Dispatcher for streaming handlers with backpressure support."""

    def __init__(self, registry: Registry):
        self.registry = registry
        self._active_streams: dict[str, bool] = {}

    async def dispatch_stream(
        self,
        request: Request,
        emit: Callable[[StreamChunk | StreamEnd | Progress], Awaitable[None]],
    ) -> Optional[Response]:
        """Dispatch a streaming request.

        Args:
            request: The JSON-RPC request
            emit: Callback to emit stream chunks

        Returns:
            Final response (or None if streaming only)
        """
        handler_info = self.registry.get_handler(request.method)

        if not handler_info:
            return Response.error(
                request.id,
                ErrorCode.METHOD_NOT_FOUND,
                f"Method not found: {request.method}",
            )

        if not handler_info.is_streaming:
            # Fall back to regular dispatch
            return await self.registry.dispatch(request)

        # Mark stream as active
        self._active_streams[str(request.id)] = True

        try:
            # Send stream start
            await emit(StreamStart(id=request.id, stream_types=handler_info.streaming_types))

            # Execute handler
            handler = handler_info.handler
            params = dict(request.params)
            params["_request_id"] = request.id
            params["_emit"] = emit

            if inspect.iscoroutinefunction(handler):
                result = await handler(**params)
            else:
                result = handler(**params)

            # Send stream end
            await emit(StreamEnd(request_id=request.id))

            # Return final response if there's a result
            if result is not None:
                return Response.success(request.id, result)
            return None

        except Exception as e:
            await emit(StreamEnd(request_id=request.id))
            return Response.error(
                request.id,
                ErrorCode.INTERNAL_ERROR,
                str(e),
            )
        finally:
            if str(request.id) in self._active_streams:
                del self._active_streams[str(request.id)]

    def cancel_stream(self, request_id: RequestId) -> bool:
        """Cancel an active stream.

        Returns:
            True if stream was found and cancelled
        """
        req_id_str = str(request_id)
        if req_id_str in self._active_streams:
            self._active_streams[req_id_str] = False
            return True
        return False

    def is_stream_active(self, request_id: RequestId) -> bool:
        """Check if a stream is still active."""
        return self._active_streams.get(str(request_id), False)


# =============================================================================
# Cancellation Support
# =============================================================================


class CancellableRequest:
    """Wrapper for requests that support cancellation."""

    def __init__(self, request: Request):
        self.request = request
        self._cancelled = False
        self._reason: Optional[str] = None

    @property
    def is_cancelled(self) -> bool:
        return self._cancelled

    @property
    def cancel_reason(self) -> Optional[str]:
        return self._reason

    def cancel(self, reason: Optional[str] = None) -> None:
        """Mark request as cancelled."""
        self._cancelled = True
        self._reason = reason

    def check_cancelled(self) -> None:
        """Raise exception if cancelled."""
        if self._cancelled:
            raise CancellationError(self._reason or "Request cancelled")


class CancellationError(Exception):
    """Exception raised when a request is cancelled."""

    def __init__(self, reason: str = "Request cancelled"):
        self.reason = reason
        super().__init__(reason)

    def to_error(self) -> JsonRpcError:
        """Convert to JSON-RPC error."""
        return cancelled(self.reason)


# =============================================================================
# Common Middleware
# =============================================================================


async def logging_middleware(
    request: Request,
    handler: Callable[[Request], Awaitable[Response]],
) -> Response:
    """Middleware that logs requests and responses."""
    print(f"[JSON-RPC] {request.method} {request.id}")
    response = await handler(request)
    status = "OK" if response.is_success else "ERROR"
    print(f"[JSON-RPC] {request.method} -> {status}")
    return response


async def validation_middleware(
    request: Request,
    handler: Callable[[Request], Awaitable[Response]],
) -> Response:
    """Middleware that validates request structure."""
    if not request.method:
        return Response.error(
            request.id,
            ErrorCode.INVALID_REQUEST,
            "Missing method",
        )

    if not isinstance(request.params, dict):
        return Response.error(
            request.id,
            ErrorCode.INVALID_PARAMS,
            "Params must be an object",
        )

    return await handler(request)
