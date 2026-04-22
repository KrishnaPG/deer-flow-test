"""
JSON-RPC 3.0 Protocol helpers and message builders.

Provides standardized message construction for bidirectional WebSocket
communication with streaming support.
"""

from typing import Any, Optional
from uuid import uuid4

from .constants import (
    DEFAULT_JSON_RPC_VERSION,
    ErrorCode,
    JsonRpcVersion,
    MessageType,
    StreamType,
    ToolName,
    ToolStatus,
)


# =============================================================================
# Message ID Generation
# =============================================================================


def generate_id() -> str:
    """Generate a unique message/request ID."""
    return str(uuid4())


# =============================================================================
# JSON-RPC 3.0 Base Messages
# =============================================================================


def json_rpc_request(
    method: str,
    params: Optional[dict] = None,
    msg_id: Optional[str] = None,
) -> dict[str, Any]:
    """Build a JSON-RPC 3.0 request object."""
    return {
        "jsonrpc": JsonRpcVersion.V3_0.value,
        "method": method,
        "params": params or {},
        "id": msg_id or generate_id(),
    }


def json_rpc_response(
    result: Any,
    msg_id: str,
) -> dict[str, Any]:
    """Build a JSON-RPC 3.0 success response."""
    return {
        "jsonrpc": JsonRpcVersion.V3_0.value,
        "result": result,
        "id": msg_id,
    }


def json_rpc_error(
    code: ErrorCode,
    message: str,
    msg_id: Optional[str] = None,
    data: Optional[dict] = None,
) -> dict[str, Any]:
    """Build a JSON-RPC 3.0 error response."""
    error_obj = {
        "jsonrpc": JsonRpcVersion.V3_0.value,
        "error": {
            "code": code.value,
            "message": message,
            "data": data or {},
        },
        "id": msg_id or generate_id(),
    }
    return error_obj


# =============================================================================
# JSON-RPC 3.0 Streaming Messages
# =============================================================================


def json_rpc_stream_start(
    msg_id: str,
    stream_types: Optional[list[str]] = None,
) -> dict[str, Any]:
    """Build a stream initialization message."""
    return {
        "jsonrpc": JsonRpcVersion.V3_0.value,
        "method": "$/stream/start",
        "params": {
            "id": msg_id,
            "streams": stream_types or [StreamType.CONTENT.value],
        },
    }


def json_rpc_stream_chunk(
    msg_id: str,
    content: str,
    stream_type: str = StreamType.CONTENT.value,
    index: Optional[int] = None,
) -> dict[str, Any]:
    """Build a stream chunk message."""
    chunk = {
        "jsonrpc": JsonRpcVersion.V3_0.value,
        "method": "$/stream/chunk",
        "params": {
            "id": msg_id,
            "type": stream_type,
            "content": content,
        },
    }
    if index is not None:
        chunk["params"]["index"] = index
    return chunk


def json_rpc_stream_end(
    msg_id: str,
    stream_type: Optional[str] = None,
) -> dict[str, Any]:
    """Build a stream completion message."""
    params = {"id": msg_id}
    if stream_type:
        params["type"] = stream_type
    return {
        "jsonrpc": JsonRpcVersion.V3_0.value,
        "method": "$/stream/end",
        "params": params,
    }


def json_rpc_progress(
    msg_id: str,
    current: int,
    total: int,
    message: Optional[str] = None,
) -> dict[str, Any]:
    """Build a progress notification."""
    params = {
        "id": msg_id,
        "current": current,
        "total": total,
        "percent": round((current / total) * 100, 1) if total > 0 else 0,
    }
    if message:
        params["message"] = message
    return {
        "jsonrpc": JsonRpcVersion.V3_0.value,
        "method": "$/progress",
        "params": params,
    }


def json_rpc_cancel(
    msg_id: str,
    reason: Optional[str] = None,
) -> dict[str, Any]:
    """Build a cancellation message."""
    params = {"id": msg_id}
    if reason:
        params["reason"] = reason
    return {
        "jsonrpc": JsonRpcVersion.V3_0.value,
        "method": "$/cancel",
        "params": params,
    }


# =============================================================================
# Application-Level Messages (Higher-level than raw JSON-RPC)
# =============================================================================


def message_user(content: str, msg_id: Optional[str] = None) -> dict[str, Any]:
    """Build a user input message."""
    return {
        "type": MessageType.MESSAGE.value,
        "content": content,
        "id": msg_id or generate_id(),
    }


def message_chunk(content: str, msg_id: Optional[str] = None) -> dict[str, Any]:
    """Build a streaming token chunk."""
    return {
        "type": MessageType.CHUNK.value,
        "content": content,
        "id": msg_id or generate_id(),
    }


def message_tool_start(
    tool_name: ToolName | str,
    args: dict,
    msg_id: Optional[str] = None,
) -> dict[str, Any]:
    """Build a tool execution start notification."""
    return {
        "type": MessageType.TOOL.value,
        "name": tool_name.value if isinstance(tool_name, ToolName) else tool_name,
        "status": ToolStatus.RUNNING.value,
        "args": args,
        "id": msg_id or generate_id(),
    }


def message_tool_end(
    tool_name: ToolName | str,
    output: str,
    success: bool = True,
    msg_id: Optional[str] = None,
) -> dict[str, Any]:
    """Build a tool execution completion notification."""
    return {
        "type": MessageType.TOOL.value,
        "name": tool_name.value if isinstance(tool_name, ToolName) else tool_name,
        "status": ToolStatus.COMPLETED.value if success else ToolStatus.FAILED.value,
        "output": output,
        "id": msg_id or generate_id(),
    }


def message_error(
    message: str,
    code: ErrorCode = ErrorCode.INTERNAL_ERROR,
    msg_id: Optional[str] = None,
) -> dict[str, Any]:
    """Build an error message."""
    return {
        "type": MessageType.ERROR.value,
        "code": code.value,
        "message": message,
        "id": msg_id or generate_id(),
    }


def message_interrupt(msg_id: Optional[str] = None) -> dict[str, Any]:
    """Build an interrupt/cancel message."""
    return {
        "type": MessageType.INTERRUPT.value,
        "id": msg_id or generate_id(),
    }


def message_done(result: Optional[Any] = None, msg_id: Optional[str] = None) -> dict[str, Any]:
    """Build a completion message."""
    msg = {
        "type": MessageType.DONE.value,
        "id": msg_id or generate_id(),
    }
    if result is not None:
        msg["result"] = result
    return msg


def message_stream_start(msg_id: Optional[str] = None) -> dict[str, Any]:
    """Build a stream start notification."""
    return {
        "type": MessageType.STREAM_START.value,
        "id": msg_id or generate_id(),
    }


def message_stream_end(msg_id: Optional[str] = None) -> dict[str, Any]:
    """Build a stream end notification."""
    return {
        "type": MessageType.STREAM_END.value,
        "id": msg_id or generate_id(),
    }


# =============================================================================
# Message Parsing Helpers
# =============================================================================


def get_message_type(msg: dict) -> Optional[str]:
    """Extract message type from a message dict."""
    return msg.get("type")


def get_message_id(msg: dict) -> Optional[str]:
    """Extract message ID from a message dict."""
    return msg.get("id") or msg.get("jsonrpc", {}).get("id")


def get_message_content(msg: dict) -> Optional[str]:
    """Extract content from a message dict."""
    return msg.get("content")


def is_json_rpc(msg: dict) -> bool:
    """Check if message follows JSON-RPC format."""
    return "jsonrpc" in msg


def is_request(msg: dict) -> bool:
    """Check if JSON-RPC message is a request (has method)."""
    return is_json_rpc(msg) and "method" in msg


def is_response(msg: dict) -> bool:
    """Check if JSON-RPC message is a response (has result or error)."""
    return is_json_rpc(msg) and ("result" in msg or "error" in msg)


def is_stream_control(msg: dict) -> bool:
    """Check if message is a stream control message."""
    if not is_json_rpc(msg):
        return False
    method = msg.get("method", "")
    return method.startswith("$/stream/") or method == "$/cancel"


# =============================================================================
# Validation Helpers
# =============================================================================


def validate_json_rpc(msg: dict) -> tuple[bool, Optional[str]]:
    """Validate JSON-RPC message structure.

    Returns (is_valid, error_message)
    """
    if not isinstance(msg, dict):
        return False, "Message must be a JSON object"

    version = msg.get("jsonrpc")
    if version not in (JsonRpcVersion.V2_0.value, JsonRpcVersion.V3_0.value):
        return False, f"Invalid JSON-RPC version: {version}"

    # Request must have method
    if "method" in msg and not isinstance(msg.get("method"), str):
        return False, "Method must be a string"

    # Response must have result or error
    if "result" in msg and "error" in msg:
        return False, "Cannot have both result and error"

    # Error must have code and message
    if "error" in msg:
        error = msg["error"]
        if not isinstance(error, dict):
            return False, "Error must be an object"
        if "code" not in error or "message" not in error:
            return False, "Error must have code and message"

    return True, None
