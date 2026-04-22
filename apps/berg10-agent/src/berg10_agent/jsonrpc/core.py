"""
Standalone JSON-RPC 3.0 Protocol Implementation

A complete, reusable JSON-RPC 3.0 implementation supporting:
- Request/Response with options
- Native Bidirectional streaming
- Acknowledgements
- Stream cancellations
- Structured error handling
- Transport abstraction
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
    CLIENT_CANCELLED = -32800

    # Extended Server error range (-32000 to -32099)
    SERVER_ERROR = -32000
    UNAUTHENTICATED = -32001
    FORBIDDEN = -32003
    RESOURCE_NOT_FOUND = -32004
    METHOD_NOT_SUPPORTED = -32005
    TIMEOUT = -32008
    CONFLICT = -32009
    PRECONDITION_FAILED = -32012
    PAYLOAD_TOO_LARGE = -32013
    TOO_MANY_REQUESTS = -32029
    CONNECTION_FAILURE = -32030


# =============================================================================
# Data Classes
# =============================================================================


@dataclass(frozen=True)
class RequestId:
    """Immutable request identifier."""

    value: Union[str, int]

    def __init__(self, value: Optional[Union[str, int]] = None):
        object.__setattr__(self, "value", value if value is not None else str(uuid4()))

    def __str__(self) -> str:
        return str(self.value)

    def __hash__(self) -> int:
        return hash(self.value)

    def __eq__(self, other: Any) -> bool:
        if isinstance(other, RequestId):
            return self.value == other.value
        return self.value == other


@dataclass(frozen=True)
class JsonRpcError:
    """JSON-RPC error structure."""

    code: ErrorCode
    message: str
    title: Optional[str] = None
    data: dict[str, Any] = field(default_factory=dict)

    def to_dict(self) -> dict[str, Any]:
        result: dict[str, Any] = {"code": self.code.value, "message": self.message}
        if self.title:
            result["title"] = self.title
        if self.data:
            result["data"] = self.data
        return result

    @classmethod
    def from_dict(cls, data: dict[str, Any]) -> JsonRpcError:
        return cls(
            code=ErrorCode(data.get("code", ErrorCode.INTERNAL_ERROR)),
            message=data.get("message", "Unknown error"),
            title=data.get("title"),
            data=data.get("data", {}),
        )


@dataclass(frozen=True)
class Request:
    """JSON-RPC request."""

    method: str
    params: Union[dict[str, Any], list[Any], None] = None
    id: Optional[RequestId] = None
    options: Optional[dict[str, Any]] = None
    jsonrpc: str = VERSION

    def to_dict(self) -> dict[str, Any]:
        result: dict[str, Any] = {
            "jsonrpc": self.jsonrpc,
            "method": self.method,
        }
        if self.params is not None:
            result["params"] = self.params
        if self.id is not None:
            result["id"] = self.id.value
        if self.options is not None:
            result["options"] = self.options
        return result

    @classmethod
    def from_dict(cls, data: dict[str, Any]) -> Request:
        return cls(
            method=data["method"],
            params=data.get("params"),
            id=RequestId(data["id"]) if "id" in data else None,
            options=data.get("options"),
            jsonrpc=data.get("jsonrpc", VERSION),
        )


@dataclass(frozen=True)
class Response:
    """JSON-RPC response (success, error, or ack)."""

    id: Optional[RequestId] = None
    result: Optional[Any] = None
    error: Optional[JsonRpcError] = None
    ack: Optional[dict[str, Any]] = None
    jsonrpc: str = VERSION

    def __post_init__(self):
        counts = sum(1 for x in (self.result, self.error, self.ack) if x is not None)
        if counts != 1:
            raise ValueError("Exactly one of result, error, or ack must be present")

    @property
    def is_success(self) -> bool:
        return self.result is not None

    @property
    def is_error(self) -> bool:
        return self.error is not None

    @property
    def is_ack(self) -> bool:
        return self.ack is not None

    def to_dict(self) -> dict[str, Any]:
        res: dict[str, Any] = {"jsonrpc": self.jsonrpc}
        if self.id is not None:
            res["id"] = self.id.value

        if self.error is not None:
            res["error"] = self.error.to_dict()
        elif self.ack is not None:
            res["ack"] = self.ack
        else:
            res["result"] = self.result

        return res

    @classmethod
    def success(cls, id: Union[RequestId, str, int, None], result: Any) -> Response:
        if id is not None and not isinstance(id, RequestId):
            id = RequestId(id)
        return cls(id=id, result=result, error=None, ack=None, jsonrpc=VERSION)

    @classmethod
    def error_response(
        cls,
        id: Union[RequestId, str, int, None],
        code: ErrorCode,
        message: str,
        title: Optional[str] = None,
        data: Optional[dict] = None,
    ) -> Response:
        if id is not None and not isinstance(id, RequestId):
            id = RequestId(id)
        error = JsonRpcError(code, message, title, data or {})
        return cls(id=id, result=None, error=error, ack=None, jsonrpc=VERSION)

    @classmethod
    def ack_response(cls, id: Union[RequestId, str, int, None], ack: dict[str, Any]) -> Response:
        if id is not None and not isinstance(id, RequestId):
            id = RequestId(id)
        return cls(id=id, result=None, error=None, ack=ack, jsonrpc=VERSION)

    @classmethod
    def from_dict(cls, data: dict[str, Any]) -> Response:
        error_data = data.get("error")
        error = JsonRpcError.from_dict(error_data) if error_data else None
        req_id = RequestId(data["id"]) if "id" in data else None
        return cls(
            id=req_id,
            result=data.get("result") if "result" in data else None,
            error=error,
            ack=data.get("ack"),
            jsonrpc=data.get("jsonrpc", VERSION),
        )


@dataclass(frozen=True)
class StreamResponse:
    """JSON-RPC streaming response (incremental data, completion, or error)."""

    stream: dict[str, Any]
    data: Optional[Any] = None
    result: Optional[Any] = None
    error: Optional[JsonRpcError] = None
    jsonrpc: str = VERSION

    def __post_init__(self):
        if "id" not in self.stream:
            raise ValueError("Stream object must contain 'id'")
        counts = sum(1 for x in (self.data, self.result, self.error) if x is not None)
        if counts != 1:
            raise ValueError(
                "Exactly one of data, result, or error must be present in StreamResponse"
            )

    @property
    def stream_id(self) -> Union[str, int]:
        return self.stream["id"]

    @property
    def is_data(self) -> bool:
        return self.data is not None

    @property
    def is_done(self) -> bool:
        return self.result is not None

    @property
    def is_error(self) -> bool:
        return self.error is not None

    def to_dict(self) -> dict[str, Any]:
        res: dict[str, Any] = {"jsonrpc": self.jsonrpc, "stream": self.stream}
        if self.error is not None:
            res["error"] = self.error.to_dict()
        elif self.result is not None:
            res["result"] = self.result
        else:
            res["data"] = self.data
        return res

    @classmethod
    def from_dict(cls, data: dict[str, Any]) -> StreamResponse:
        error_data = data.get("error")
        error = JsonRpcError.from_dict(error_data) if error_data else None
        return cls(
            stream=data["stream"],
            data=data.get("data") if "data" in data else None,
            result=data.get("result") if "result" in data else None,
            error=error,
            jsonrpc=data.get("jsonrpc", VERSION),
        )


# Type alias for any JSON-RPC message
Message = Union[Request, Response, StreamResponse]


# =============================================================================
# Message Builders
# =============================================================================


def request(
    method: str,
    params: Union[dict[str, Any], list[Any], None] = None,
    msg_id: Union[str, int, None] = None,
    options: Optional[dict[str, Any]] = None,
) -> Request:
    """Build a JSON-RPC request."""
    return Request(
        method=method,
        params=params,
        id=RequestId(msg_id) if msg_id is not None else None,
        options=options,
    )


def success_response(request_id: Union[str, int, RequestId, None], result: Any) -> Response:
    """Build a successful response."""
    return Response.success(request_id, result)


def error_response(
    request_id: Union[str, int, RequestId, None],
    code: ErrorCode,
    message: str,
    title: Optional[str] = None,
    data: Optional[dict[str, Any]] = None,
) -> Response:
    """Build an error response."""
    return Response.error_response(request_id, code, message, title, data)


def ack_response(request_id: Union[str, int, RequestId, None], ack: dict[str, Any]) -> Response:
    """Build an acknowledgement response."""
    return Response.ack_response(request_id, ack)


def stream_data(
    stream_id: Union[str, int],
    data: Any,
    stream_meta: Optional[dict[str, Any]] = None,
) -> StreamResponse:
    """Build a stream data chunk."""
    stream_obj = {"id": stream_id}
    if stream_meta:
        stream_obj.update(stream_meta)
    return StreamResponse(stream=stream_obj, data=data)


def stream_done(
    stream_id: Union[str, int],
    result: Any,
    stream_meta: Optional[dict[str, Any]] = None,
) -> StreamResponse:
    """Build a stream completion message."""
    stream_obj = {"id": stream_id}
    if stream_meta:
        stream_obj.update(stream_meta)
    return StreamResponse(stream=stream_obj, result=result)


def stream_error(
    stream_id: Union[str, int],
    code: ErrorCode,
    message: str,
    title: Optional[str] = None,
    data: Optional[dict[str, Any]] = None,
    stream_meta: Optional[dict[str, Any]] = None,
) -> StreamResponse:
    """Build a stream error message."""
    stream_obj = {"id": stream_id}
    if stream_meta:
        stream_obj.update(stream_meta)
    error = JsonRpcError(code, message, title, data or {})
    return StreamResponse(stream=stream_obj, error=error)


# =============================================================================
# Parsing & Validation
# =============================================================================


class ParseError(Exception):
    """Error parsing JSON-RPC message."""

    def __init__(self, message: str, code: ErrorCode = ErrorCode.PARSE_ERROR):
        self.code = code
        super().__init__(message)


def parse_message(data: dict[str, Any]) -> Message:
    """Parse a JSON-RPC message from a dictionary."""
    if data.get("jsonrpc") != VERSION:
        raise ParseError(
            f"Invalid JSON-RPC version: {data.get('jsonrpc')}, expected {VERSION}",
            ErrorCode.INVALID_REQUEST,
        )

    if "method" in data:
        return Request.from_dict(data)

    if "stream" in data:
        return StreamResponse.from_dict(data)

    if any(k in data for k in ("result", "error", "ack")):
        return Response.from_dict(data)

    raise ParseError(
        "Invalid message: must contain 'method', 'stream', 'result', 'error', or 'ack'",
        ErrorCode.INVALID_REQUEST,
    )


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
    """Validate request structure without parsing."""
    if not isinstance(data, dict):
        return False, "Message must be a JSON object"
    if data.get("jsonrpc") != VERSION:
        return False, "Invalid jsonrpc version"
    if "method" not in data or not isinstance(data["method"], str):
        return False, "Request must have a string 'method' field"
    params = data.get("params")
    if params is not None and not isinstance(params, (dict, list)):
        return False, "Params must be an object or array"
    options = data.get("options")
    if options is not None and not isinstance(options, dict):
        return False, "Options must be an object"
    return True, None


def validate_response(data: dict[str, Any]) -> tuple[bool, Optional[str]]:
    """Validate response structure without parsing."""
    if not isinstance(data, dict):
        return False, "Message must be a JSON object"
    if data.get("jsonrpc") != VERSION:
        return False, "Invalid jsonrpc version"

    has_result = "result" in data
    has_error = "error" in data
    has_ack = "ack" in data

    if sum(1 for x in (has_result, has_error, has_ack) if x) != 1:
        return False, "Response must have exactly one of 'result', 'error', or 'ack'"

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


def batch(items: list[Message]) -> list[dict[str, Any]]:
    """Convert list of messages to batch format."""
    return [item.to_dict() for item in items]


def parse_batch(data: list[dict[str, Any]]) -> list[Message]:
    """Parse a batch of messages."""
    results = []
    for item in data:
        try:
            results.append(parse_message(item))
        except ParseError as e:
            req_id = item.get("id")
            results.append(Response.error_response(req_id, e.code, str(e)))
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
    return JsonRpcError(ErrorCode.PARSE_ERROR, message)


def invalid_request(message: str = "Invalid request") -> JsonRpcError:
    return JsonRpcError(ErrorCode.INVALID_REQUEST, message)


def method_not_found(method: str) -> JsonRpcError:
    return JsonRpcError(ErrorCode.METHOD_NOT_FOUND, f"Method not found: {method}")


def invalid_params(message: str = "Invalid params") -> JsonRpcError:
    return JsonRpcError(ErrorCode.INVALID_PARAMS, message)


def internal_error(message: str = "Internal error") -> JsonRpcError:
    return JsonRpcError(ErrorCode.INTERNAL_ERROR, message)


def cancelled(reason: str = "Request cancelled") -> JsonRpcError:
    return JsonRpcError(ErrorCode.CLIENT_CANCELLED, reason, title="Client Cancelled")
