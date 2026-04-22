"""
Standalone JSON-RPC 3.0 Protocol Implementation

A complete, reusable JSON-RPC 3.0 implementation supporting:
- Request/Response with unique IDs
- Bidirectional streaming with multiple stream types
- Batch operations
- Cancellation with reason
- Progress notifications
- Structured error handling
- Transport abstraction (WebSocket, HTTP, etc.)
"""

from __future__ import annotations

import json
from dataclasses import dataclass, field
from enum import Enum, IntEnum
from typing import Any, AsyncIterator, Callable, Coroutine, Optional, Protocol, TypeVar, Union
from uuid import uuid4


# =============================================================================
# Protocol Version
# =============================================================================

VERSION = "3.0"


# =============================================================================
# Error Codes (JSON-RPC 3.0 Standard + Extensions)
# =============================================================================


class ErrorCode(IntEnum):
    """JSON-RPC 3.0 error codes."""

    # Standard codes (-32768 to -32000 reserved)
    PARSE_ERROR = -32700
    INVALID_REQUEST = -32600
    METHOD_NOT_FOUND = -32601
    INVALID_PARAMS = -32602
    INTERNAL_ERROR = -32603

    # Server error range (-32000 to -32099)
    SERVER_ERROR = -32000
    RESOURCE_NOT_FOUND = -32001
    RESOURCE_EXISTS = -32002
    STREAM_ERROR = -32003
    CANCELLED = -32004
    TIMEOUT = -32005
    AUTHENTICATION_ERROR = -32006
    AUTHORIZATION_ERROR = -32007
    RATE_LIMITED = -32008
    VALIDATION_ERROR = -32009


# =============================================================================
# Stream Types
# =============================================================================


class StreamType(str, Enum):
    """Types of streams supported in JSON-RPC 3.0."""

    CONTENT = "content"  # Primary response content
    PROGRESS = "progress"  # Progress updates
    LOG = "log"  # Log messages
    ERROR = "error"  # Error stream
    METADATA = "metadata"  # Metadata/information


# =============================================================================
# Data Classes
# =============================================================================


@dataclass(frozen=True)
class RequestId:
    """Immutable request identifier."""

    value: str

    def __init__(self, value: Optional[str] = None):
        object.__setattr__(self, "value", value or str(uuid4()))

    def __str__(self) -> str:
        return self.value

    def __hash__(self) -> int:
        return hash(self.value)


@dataclass(frozen=True)
class JsonRpcError:
    """JSON-RPC error structure."""

    code: ErrorCode
    message: str
    data: dict[str, Any] = field(default_factory=dict)

    def to_dict(self) -> dict[str, Any]:
        result = {"code": self.code.value, "message": self.message}
        if self.data:
            result["data"] = self.data
        return result

    @classmethod
    def from_dict(cls, data: dict[str, Any]) -> JsonRpcError:
        return cls(
            code=ErrorCode(data.get("code", ErrorCode.INTERNAL_ERROR)),
            message=data.get("message", "Unknown error"),
            data=data.get("data", {}),
        )


@dataclass(frozen=True)
class Request:
    """JSON-RPC request."""

    method: str
    params: dict[str, Any] = field(default_factory=dict)
    id: RequestId = field(default_factory=RequestId)
    jsonrpc: str = VERSION

    def to_dict(self) -> dict[str, Any]:
        return {
            "jsonrpc": self.jsonrpc,
            "method": self.method,
            "params": self.params,
            "id": str(self.id),
        }

    @classmethod
    def from_dict(cls, data: dict[str, Any]) -> Request:
        return cls(
            method=data["method"],
            params=data.get("params", {}),
            id=RequestId(data.get("id")),
            jsonrpc=data.get("jsonrpc", VERSION),
        )


@dataclass(frozen=True)
class Response:
    """JSON-RPC response (success or error)."""

    id: RequestId
    result: Optional[Any] = None
    error: Optional[JsonRpcError] = None
    jsonrpc: str = VERSION

    def __post_init__(self):
        if self.result is not None and self.error is not None:
            raise ValueError("Response cannot have both result and error")

    @property
    def is_success(self) -> bool:
        return self.error is None

    @property
    def is_error(self) -> bool:
        return self.error is not None

    def to_dict(self) -> dict[str, Any]:
        result: dict[str, Any] = {"jsonrpc": self.jsonrpc, "id": str(self.id)}
        if self.error:
            result["error"] = self.error.to_dict()
        else:
            result["result"] = self.result
        return result

    @classmethod
    def success(cls, id: RequestId | str, result: Any) -> Response:
        """Create a successful response."""
        if isinstance(id, str):
            id = RequestId(id)
        # Create without using the error classmethod to avoid name conflict
        return cls.__new__(cls, id=id, result=result, error=None, jsonrpc=VERSION)

    @classmethod
    def error_response(
        cls,
        id: RequestId | str,
        code: ErrorCode,
        message: str,
        data: Optional[dict] = None,
    ) -> Response:
        """Create an error response."""
        if isinstance(id, str):
            id = RequestId(id)
        error = JsonRpcError(code, message, data or {})
        # Create without using cls() directly to avoid __init__ issues with frozen dataclass
        obj = cls.__new__(cls)
        object.__setattr__(obj, "id", id)
        object.__setattr__(obj, "result", None)
        object.__setattr__(obj, "error", error)
        object.__setattr__(obj, "jsonrpc", VERSION)
        return obj

    @classmethod
    def from_dict(cls, data: dict[str, Any]) -> Response:
        error_data = data.get("error")
        error = JsonRpcError.from_dict(error_data) if error_data else None
        return cls(
            id=RequestId(data.get("id")),
            result=data.get("result"),
            error=error,
            jsonrpc=data.get("jsonrpc", VERSION),
        )


@dataclass(frozen=True)
class StreamStart:
    """Stream initialization message."""

    id: RequestId
    stream_types: list[StreamType] = field(default_factory=lambda: [StreamType.CONTENT])
    metadata: dict[str, Any] = field(default_factory=dict)

    def to_dict(self) -> dict[str, Any]:
        return {
            "jsonrpc": VERSION,
            "method": "$/stream/start",
            "params": {
                "id": str(self.id),
                "streams": [t.value for t in self.stream_types],
                **self.metadata,
            },
        }


@dataclass(frozen=True)
class StreamChunk:
    """Stream chunk message."""

    request_id: RequestId
    content: str
    stream_type: StreamType = StreamType.CONTENT
    index: Optional[int] = None
    metadata: dict[str, Any] = field(default_factory=dict)

    def to_dict(self) -> dict[str, Any]:
        params: dict[str, Any] = {
            "id": str(self.request_id),
            "type": self.stream_type.value,
            "content": self.content,
        }
        if self.index is not None:
            params["index"] = self.index
        params.update(self.metadata)
        return {
            "jsonrpc": VERSION,
            "method": "$/stream/chunk",
            "params": params,
        }


@dataclass(frozen=True)
class StreamEnd:
    """Stream completion message."""

    request_id: RequestId
    stream_type: Optional[StreamType] = None
    metadata: dict[str, Any] = field(default_factory=dict)

    def to_dict(self) -> dict[str, Any]:
        params: dict[str, Any] = {"id": str(self.request_id)}
        if self.stream_type:
            params["type"] = self.stream_type.value
        params.update(self.metadata)
        return {
            "jsonrpc": VERSION,
            "method": "$/stream/end",
            "params": params,
        }


@dataclass(frozen=True)
class Progress:
    """Progress notification."""

    request_id: RequestId
    current: int
    total: int
    message: Optional[str] = None
    metadata: dict[str, Any] = field(default_factory=dict)

    @property
    def percent(self) -> float:
        """Calculate progress percentage."""
        if self.total <= 0:
            return 0.0
        return round((self.current / self.total) * 100, 1)

    def to_dict(self) -> dict[str, Any]:
        params: dict[str, Any] = {
            "id": str(self.request_id),
            "current": self.current,
            "total": self.total,
            "percent": self.percent,
        }
        if self.message:
            params["message"] = self.message
        params.update(self.metadata)
        return {
            "jsonrpc": VERSION,
            "method": "$/progress",
            "params": params,
        }


@dataclass(frozen=True)
class Cancel:
    """Cancellation message."""

    request_id: RequestId
    reason: Optional[str] = None

    def to_dict(self) -> dict[str, Any]:
        params: dict[str, Any] = {"id": str(self.request_id)}
        if self.reason:
            params["reason"] = self.reason
        return {
            "jsonrpc": VERSION,
            "method": "$/cancel",
            "params": params,
        }


# Type alias for any JSON-RPC message
Message = Union[Request, Response, StreamStart, StreamChunk, StreamEnd, Progress, Cancel]


# =============================================================================
# Message Builders
# =============================================================================


def request(
    method: str,
    params: Optional[dict[str, Any]] = None,
    msg_id: Optional[str] = None,
) -> Request:
    """Build a JSON-RPC request."""
    return Request(
        method=method,
        params=params or {},
        id=RequestId(msg_id),
    )


def success_response(request_id: str | RequestId, result: Any) -> Response:
    """Build a successful response."""
    if isinstance(request_id, str):
        request_id = RequestId(request_id)
    return Response.success(request_id, result)


def error_response(
    request_id: str | RequestId,
    code: ErrorCode,
    message: str,
    data: Optional[dict[str, Any]] = None,
) -> Response:
    """Build an error response."""
    if isinstance(request_id, str):
        request_id = RequestId(request_id)
    return Response.error_response(request_id, code, message, data)


def stream_start(
    request_id: str | RequestId,
    stream_types: Optional[list[StreamType]] = None,
    metadata: Optional[dict[str, Any]] = None,
) -> StreamStart:
    """Build a stream start message."""
    if isinstance(request_id, str):
        request_id = RequestId(request_id)
    return StreamStart(
        id=request_id,
        stream_types=stream_types or [StreamType.CONTENT],
        metadata=metadata or {},
    )


def stream_chunk(
    request_id: str | RequestId,
    content: str,
    stream_type: StreamType = StreamType.CONTENT,
    index: Optional[int] = None,
    metadata: Optional[dict[str, Any]] = None,
) -> StreamChunk:
    """Build a stream chunk message."""
    if isinstance(request_id, str):
        request_id = RequestId(request_id)
    return StreamChunk(
        request_id=request_id,
        content=content,
        stream_type=stream_type,
        index=index,
        metadata=metadata or {},
    )


def stream_end(
    request_id: str | RequestId,
    stream_type: Optional[StreamType] = None,
    metadata: Optional[dict[str, Any]] = None,
) -> StreamEnd:
    """Build a stream end message."""
    if isinstance(request_id, str):
        request_id = RequestId(request_id)
    return StreamEnd(
        request_id=request_id,
        stream_type=stream_type,
        metadata=metadata or {},
    )


def progress(
    request_id: str | RequestId,
    current: int,
    total: int,
    message: Optional[str] = None,
    metadata: Optional[dict[str, Any]] = None,
) -> Progress:
    """Build a progress notification."""
    if isinstance(request_id, str):
        request_id = RequestId(request_id)
    return Progress(
        request_id=request_id,
        current=current,
        total=total,
        message=message,
        metadata=metadata or {},
    )


def cancel(request_id: str | RequestId, reason: Optional[str] = None) -> Cancel:
    """Build a cancellation message."""
    if isinstance(request_id, str):
        request_id = RequestId(request_id)
    return Cancel(request_id=request_id, reason=reason)


# =============================================================================
# Parsing & Validation
# =============================================================================


class ParseError(Exception):
    """Error parsing JSON-RPC message."""

    def __init__(self, message: str, code: ErrorCode = ErrorCode.PARSE_ERROR):
        self.code = code
        super().__init__(message)


def parse_message(data: dict[str, Any]) -> Message:
    """Parse a JSON-RPC message from a dictionary.

    Args:
        data: The parsed JSON object

    Returns:
        A Message subtype (Request, Response, etc.)

    Raises:
        ParseError: If the message is invalid
    """
    # Validate version
    version = data.get("jsonrpc")
    if version != VERSION:
        raise ParseError(
            f"Invalid JSON-RPC version: {version}, expected {VERSION}",
            ErrorCode.INVALID_REQUEST,
        )

    # Determine message type
    method = data.get("method")

    # Stream control messages
    if method and method.startswith("$/"):
        params = data.get("params", {})
        request_id = RequestId(params.get("id"))

        if method == "$/stream/start":
            stream_types = [StreamType(t) for t in params.get("streams", ["content"])]
            metadata = {k: v for k, v in params.items() if k not in ("id", "streams")}
            return StreamStart(id=request_id, stream_types=stream_types, metadata=metadata)

        elif method == "$/stream/chunk":
            stream_type = StreamType(params.get("type", "content"))
            index = params.get("index")
            content = params.get("content", "")
            metadata = {
                k: v for k, v in params.items() if k not in ("id", "type", "content", "index")
            }
            return StreamChunk(
                request_id=request_id,
                content=content,
                stream_type=stream_type,
                index=index,
                metadata=metadata,
            )

        elif method == "$/stream/end":
            stream_type = params.get("type")
            if stream_type:
                stream_type = StreamType(stream_type)
            metadata = {k: v for k, v in params.items() if k not in ("id", "type")}
            return StreamEnd(request_id=request_id, stream_type=stream_type, metadata=metadata)

        elif method == "$/progress":
            current = params.get("current", 0)
            total = params.get("total", 0)
            message = params.get("message")
            metadata = {
                k: v
                for k, v in params.items()
                if k not in ("id", "current", "total", "message", "percent")
            }
            return Progress(
                request_id=request_id,
                current=current,
                total=total,
                message=message,
                metadata=metadata,
            )

        elif method == "$/cancel":
            reason = params.get("reason")
            return Cancel(request_id=request_id, reason=reason)

        else:
            raise ParseError(f"Unknown method: {method}", ErrorCode.METHOD_NOT_FOUND)

    # Request (has method)
    if method:
        return Request.from_dict(data)

    # Response (has result or error)
    if "result" in data or "error" in data:
        return Response.from_dict(data)

    raise ParseError("Invalid message: neither request nor response", ErrorCode.INVALID_REQUEST)


def parse_json(text: str) -> Message:
    """Parse JSON-RPC message from JSON string."""
    try:
        data = json.loads(text)
    except json.JSONDecodeError as e:
        raise ParseError(f"Invalid JSON: {e}", ErrorCode.PARSE_ERROR) from e

    if not isinstance(data, dict):
        raise ParseError("Message must be a JSON object", ErrorCode.INVALID_REQUEST)

    return parse_message(data)


def validate_request(data: dict[str, Any]) -> tuple[bool, Optional[str]]:
    """Validate request structure without parsing.

    Returns (is_valid, error_message)
    """
    if not isinstance(data, dict):
        return False, "Message must be a JSON object"

    if data.get("jsonrpc") != VERSION:
        return False, f"Invalid jsonrpc version"

    if "method" not in data:
        return False, "Request must have 'method' field"

    if not isinstance(data["method"], str):
        return False, "Method must be a string"

    params = data.get("params")
    if params is not None and not isinstance(params, (dict, list)):
        return False, "Params must be an object or array"

    return True, None


def validate_response(data: dict[str, Any]) -> tuple[bool, Optional[str]]:
    """Validate response structure without parsing."""
    if not isinstance(data, dict):
        return False, "Message must be a JSON object"

    if data.get("jsonrpc") != VERSION:
        return False, f"Invalid jsonrpc version"

    has_result = "result" in data
    has_error = "error" in data

    if not has_result and not has_error:
        return False, "Response must have 'result' or 'error'"

    if has_result and has_error:
        return False, "Cannot have both result and error"

    if has_error:
        error = data["error"]
        if not isinstance(error, dict):
            return False, "Error must be an object"
        if "code" not in error or "message" not in error:
            return False, "Error must have 'code' and 'message'"

    return True, None


# =============================================================================
# Batch Operations
# =============================================================================


BatchItem = Union[Request, Response]


def batch(items: list[BatchItem]) -> list[dict[str, Any]]:
    """Convert list of messages to batch format."""
    return [item.to_dict() for item in items]


def parse_batch(data: list[dict[str, Any]]) -> list[Message]:
    """Parse a batch of messages."""
    results = []
    for item in data:
        try:
            results.append(parse_message(item))
        except ParseError as e:
            # Create error response for invalid items
            req_id = item.get("id", str(uuid4()))
            results.append(
                Response.error(
                    RequestId(req_id),
                    e.code,
                    str(e),
                )
            )
    return results


# =============================================================================
# Serialization
# =============================================================================


def serialize(message: Message) -> str:
    """Serialize a message to JSON string."""
    return json.dumps(message.to_dict())


def serialize_batch(messages: list[Message]) -> str:
    """Serialize multiple messages as a batch."""
    return json.dumps([m.to_dict() for m in messages])


# =============================================================================
# Error Helpers
# =============================================================================


def parse_error(message: str = "Parse error") -> JsonRpcError:
    """Create a parse error."""
    return JsonRpcError(ErrorCode.PARSE_ERROR, message)


def invalid_request(message: str = "Invalid request") -> JsonRpcError:
    """Create an invalid request error."""
    return JsonRpcError(ErrorCode.INVALID_REQUEST, message)


def method_not_found(method: str) -> JsonRpcError:
    """Create a method not found error."""
    return JsonRpcError(ErrorCode.METHOD_NOT_FOUND, f"Method not found: {method}")


def invalid_params(message: str = "Invalid params") -> JsonRpcError:
    """Create an invalid params error."""
    return JsonRpcError(ErrorCode.INVALID_PARAMS, message)


def internal_error(message: str = "Internal error") -> JsonRpcError:
    """Create an internal error."""
    return JsonRpcError(ErrorCode.INTERNAL_ERROR, message)


def cancelled(reason: str = "Request cancelled") -> JsonRpcError:
    """Create a cancelled error."""
    return JsonRpcError(ErrorCode.CANCELLED, reason)
