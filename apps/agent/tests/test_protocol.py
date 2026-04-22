"""Tests for protocol message builders and parsing helpers."""

import pytest

from berg10_agent.protocol import (
    generate_id,
    get_message_content,
    get_message_id,
    get_message_type,
    is_json_rpc,
    is_request,
    is_response,
    is_stream_control,
    json_rpc_cancel,
    json_rpc_error,
    json_rpc_progress,
    json_rpc_request,
    json_rpc_response,
    json_rpc_stream_chunk,
    json_rpc_stream_end,
    json_rpc_stream_start,
    message_chunk,
    message_done,
    message_error,
    message_interrupt,
    message_stream_end,
    message_stream_start,
    message_tool_end,
    message_tool_start,
    message_user,
    validate_json_rpc,
)
from berg10_agent.constants import ErrorCode, MessageType, StreamType, ToolName, ToolStatus


class TestMessageIdGeneration:
    def test_generate_id_returns_string(self):
        result = generate_id()
        assert isinstance(result, str)

    def test_generate_id_unique(self):
        ids = {generate_id() for _ in range(100)}
        assert len(ids) == 100


class TestJsonRpcRequest:
    def test_basic_request(self):
        req = json_rpc_request("test_method")
        assert req["jsonrpc"] == "3.0"
        assert req["method"] == "test_method"
        assert "id" in req

    def test_request_with_params(self):
        req = json_rpc_request("echo", params={"msg": "hello"})
        assert req["params"] == {"msg": "hello"}

    def test_request_with_id(self):
        req = json_rpc_request("test", msg_id="abc-123")
        assert req["id"] == "abc-123"


class TestJsonRpcResponse:
    def test_success_response(self):
        resp = json_rpc_response({"data": "ok"}, msg_id="1")
        assert resp["jsonrpc"] == "3.0"
        assert resp["result"] == {"data": "ok"}
        assert resp["id"] == "1"

    def test_error_response(self):
        resp = json_rpc_error(ErrorCode.INVALID_PARAMS, "Bad params", msg_id="2")
        assert resp["error"]["code"] == ErrorCode.INVALID_PARAMS.value
        assert resp["error"]["message"] == "Bad params"
        assert resp["id"] == "2"


class TestStreamMessages:
    def test_stream_start(self):
        msg = json_rpc_stream_start("stream-1")
        assert msg["method"] == "$/stream/start"
        assert msg["params"]["id"] == "stream-1"

    def test_stream_chunk(self):
        msg = json_rpc_stream_chunk("stream-1", "hello", StreamType.CONTENT.value)
        assert msg["method"] == "$/stream/chunk"
        assert msg["params"]["content"] == "hello"

    def test_stream_end(self):
        msg = json_rpc_stream_end("stream-1")
        assert msg["method"] == "$/stream/end"

    def test_stream_chunk_with_index(self):
        msg = json_rpc_stream_chunk("s1", "data", index=5)
        assert msg["params"]["index"] == 5

    def test_progress(self):
        msg = json_rpc_progress("s1", 50, 100, "Half done")
        assert msg["params"]["percent"] == 50.0
        assert msg["params"]["message"] == "Half done"

    def test_cancel(self):
        msg = json_rpc_cancel("s1", reason="User requested")
        assert msg["params"]["reason"] == "User requested"


class TestApplicationMessages:
    def test_message_user(self):
        msg = message_user("Hello")
        assert msg["type"] == MessageType.MESSAGE.value
        assert msg["content"] == "Hello"

    def test_message_chunk(self):
        msg = message_chunk("token")
        assert msg["type"] == MessageType.CHUNK.value
        assert msg["content"] == "token"

    def test_message_tool_start(self):
        msg = message_tool_start(ToolName.READ_FILE, {"path": "test.py"})
        assert msg["type"] == MessageType.TOOL.value
        assert msg["name"] == "read_file"
        assert msg["status"] == ToolStatus.RUNNING.value

    def test_message_tool_end(self):
        msg = message_tool_end(ToolName.READ_FILE, "content", success=True)
        assert msg["status"] == ToolStatus.COMPLETED.value

    def test_message_tool_end_failed(self):
        msg = message_tool_end(ToolName.RUN_COMMAND, "error", success=False)
        assert msg["status"] == ToolStatus.FAILED.value

    def test_message_error(self):
        msg = message_error("Something broke")
        assert msg["type"] == MessageType.ERROR.value

    def test_message_done(self):
        msg = message_done({"answer": "42"})
        assert msg["type"] == MessageType.DONE.value
        assert msg["result"] == {"answer": "42"}

    def test_message_interrupt(self):
        msg = message_interrupt()
        assert msg["type"] == MessageType.INTERRUPT.value

    def test_message_stream_start_end(self):
        start = message_stream_start()
        end = message_stream_end()
        assert start["type"] == MessageType.STREAM_START.value
        assert end["type"] == MessageType.STREAM_END.value


class TestParsing:
    def test_get_message_type(self):
        assert get_message_type({"type": "chunk"}) == "chunk"

    def test_get_message_id(self):
        assert get_message_id({"id": "abc"}) == "abc"

    def test_get_message_content(self):
        assert get_message_content({"content": "hello"}) == "hello"

    def test_is_json_rpc(self):
        assert is_json_rpc({"jsonrpc": "3.0"}) is True
        assert is_json_rpc({"type": "msg"}) is False

    def test_is_request(self):
        assert is_request({"jsonrpc": "3.0", "method": "test"}) is True
        assert is_request({"jsonrpc": "3.0", "result": "ok"}) is False

    def test_is_response(self):
        assert is_response({"jsonrpc": "3.0", "result": "ok"}) is True
        assert is_response({"jsonrpc": "3.0", "error": {"code": -1, "message": "err"}}) is True

    def test_is_stream_control(self):
        assert is_stream_control({"jsonrpc": "3.0", "method": "$/stream/start"}) is True
        assert is_stream_control({"jsonrpc": "3.0", "method": "$/cancel"}) is True
        assert is_stream_control({"jsonrpc": "3.0", "method": "regular"}) is False


class TestValidation:
    def test_valid_request(self):
        ok, err = validate_json_rpc({"jsonrpc": "3.0", "method": "test"})
        assert ok is True
        assert err is None

    def test_invalid_version(self):
        ok, err = validate_json_rpc({"jsonrpc": "1.0", "method": "test"})
        assert ok is False
        assert "version" in err.lower()

    def test_both_result_and_error(self):
        ok, err = validate_json_rpc(
            {
                "jsonrpc": "3.0",
                "result": "ok",
                "error": {"code": -1, "message": "err"},
            }
        )
        assert ok is False

    def test_not_dict(self):
        ok, err = validate_json_rpc("not a dict")
        assert ok is False
