"""Tests for JSON-RPC 3.0 core module."""

import pytest

from berg10_agent.jsonrpc.core import (
    Cancel,
    ErrorCode,
    JsonRpcError,
    Progress,
    Request,
    RequestId,
    Response,
    StreamChunk,
    StreamEnd,
    StreamStart,
    StreamType,
    batch,
    cancel,
    parse_json,
    parse_message,
    progress,
    request,
    serialize,
    stream_chunk,
    stream_end,
    stream_start,
    success_response,
    validate_request,
    validate_response,
)


class TestRequestId:
    def test_creation_with_value(self):
        req_id = RequestId("test-123")
        assert req_id.value == "test-123"

    def test_creation_auto_generates_uuid(self):
        req_id1 = RequestId()
        req_id2 = RequestId()
        assert req_id1.value != req_id2.value
        assert len(req_id1.value) == 36  # UUID length

    def test_string_conversion(self):
        req_id = RequestId("abc")
        assert str(req_id) == "abc"

    def test_hashable(self):
        req_id = RequestId("test")
        assert hash(req_id) == hash("test")


class TestJsonRpcError:
    def test_to_dict(self):
        error = JsonRpcError(ErrorCode.INVALID_PARAMS, "Bad params", {"field": "name"})
        assert error.to_dict() == {
            "code": -32602,
            "message": "Bad params",
            "data": {"field": "name"},
        }

    def test_from_dict(self):
        data = {"code": -32601, "message": "Not found", "data": {}}
        error = JsonRpcError.from_dict(data)
        assert error.code == ErrorCode.METHOD_NOT_FOUND
        assert error.message == "Not found"


class TestRequest:
    def test_to_dict(self):
        req = Request("echo", {"message": "hello"}, RequestId("123"))
        assert req.to_dict() == {
            "jsonrpc": "3.0",
            "method": "echo",
            "params": {"message": "hello"},
            "id": "123",
        }

    def test_from_dict(self):
        data = {
            "jsonrpc": "3.0",
            "method": "echo",
            "params": {"message": "hello"},
            "id": "456",
        }
        req = Request.from_dict(data)
        assert req.method == "echo"
        assert req.params == {"message": "hello"}
        assert str(req.id) == "456"


class TestResponse:
    def test_success_response(self):
        resp = Response.success("123", {"result": "ok"})
        assert resp.is_success
        assert not resp.is_error
        assert resp.result == {"result": "ok"}
        assert resp.error is None

    def test_error_response(self):
        resp = Response.error("123", ErrorCode.METHOD_NOT_FOUND, "Not found")
        assert not resp.is_success
        assert resp.is_error
        assert resp.error.code == ErrorCode.METHOD_NOT_FOUND

    def test_cannot_have_both_result_and_error(self):
        with pytest.raises(ValueError):
            Response(
                RequestId("123"), result="ok", error=JsonRpcError(ErrorCode.INTERNAL_ERROR, "err")
            )

    def test_to_dict_success(self):
        resp = Response.success("123", "hello")
        assert resp.to_dict() == {
            "jsonrpc": "3.0",
            "id": "123",
            "result": "hello",
        }

    def test_to_dict_error(self):
        resp = Response.error("123", ErrorCode.INVALID_PARAMS, "Bad params")
        assert resp.to_dict() == {
            "jsonrpc": "3.0",
            "id": "123",
            "error": {"code": -32602, "message": "Bad params"},
        }

    def test_from_dict(self):
        data = {"jsonrpc": "3.0", "id": "789", "result": "success"}
        resp = Response.from_dict(data)
        assert resp.is_success
        assert resp.result == "success"


class TestStreamMessages:
    def test_stream_start_to_dict(self):
        start = StreamStart(RequestId("123"), [StreamType.CONTENT, StreamType.PROGRESS])
        data = start.to_dict()
        assert data["method"] == "$/stream/start"
        assert data["params"]["id"] == "123"
        assert data["params"]["streams"] == ["content", "progress"]

    def test_stream_chunk_to_dict(self):
        chunk = StreamChunk(RequestId("123"), "hello", StreamType.CONTENT, 5)
        data = chunk.to_dict()
        assert data["method"] == "$/stream/chunk"
        assert data["params"]["content"] == "hello"
        assert data["params"]["index"] == 5

    def test_stream_end_to_dict(self):
        end = StreamEnd(RequestId("123"), StreamType.CONTENT)
        data = end.to_dict()
        assert data["method"] == "$/stream/end"
        assert data["params"]["type"] == "content"


class TestProgress:
    def test_percent_calculation(self):
        prog = Progress(RequestId("123"), 50, 100)
        assert prog.percent == 50.0

    def test_percent_zero_total(self):
        prog = Progress(RequestId("123"), 0, 0)
        assert prog.percent == 0.0

    def test_to_dict(self):
        prog = Progress(RequestId("123"), 75, 100, "Processing...")
        data = prog.to_dict()
        assert data["params"]["current"] == 75
        assert data["params"]["total"] == 100
        assert data["params"]["percent"] == 75.0
        assert data["params"]["message"] == "Processing..."


class TestMessageBuilders:
    def test_request_builder(self):
        req = request("echo", {"msg": "hi"}, "id-1")
        assert req.method == "echo"
        assert req.params == {"msg": "hi"}

    def test_success_response_builder(self):
        resp = success_response("id-1", "result")
        assert resp.is_success
        assert resp.result == "result"

    def test_stream_start_builder(self):
        start = stream_start("id-1", [StreamType.CONTENT])
        assert start.id.value == "id-1"

    def test_stream_chunk_builder(self):
        chunk = stream_chunk("id-1", "data", StreamType.CONTENT, 0)
        assert chunk.content == "data"

    def test_progress_builder(self):
        prog = progress("id-1", 10, 20, "halfway")
        assert prog.current == 10
        assert prog.message == "halfway"

    def test_cancel_builder(self):
        can = cancel("id-1", "user requested")
        assert can.request_id.value == "id-1"
        assert can.reason == "user requested"


class TestParsing:
    def test_parse_request(self):
        data = {"jsonrpc": "3.0", "method": "echo", "params": {}, "id": "123"}
        msg = parse_message(data)
        assert isinstance(msg, Request)
        assert msg.method == "echo"

    def test_parse_response(self):
        data = {"jsonrpc": "3.0", "id": "123", "result": "ok"}
        msg = parse_message(data)
        assert isinstance(msg, Response)
        assert msg.result == "ok"

    def test_parse_stream_start(self):
        data = {"jsonrpc": "3.0", "method": "$/stream/start", "params": {"id": "123"}}
        msg = parse_message(data)
        assert isinstance(msg, StreamStart)

    def test_parse_invalid_version(self):
        data = {"jsonrpc": "2.0", "method": "echo", "id": "123"}
        with pytest.raises(Exception):
            parse_message(data)

    def test_parse_json_string(self):
        json_str = '{"jsonrpc": "3.0", "method": "echo", "id": "123"}'
        msg = parse_json(json_str)
        assert isinstance(msg, Request)


class TestValidation:
    def test_validate_request_valid(self):
        data = {"jsonrpc": "3.0", "method": "echo", "params": {}, "id": "123"}
        valid, error = validate_request(data)
        assert valid
        assert error is None

    def test_validate_request_missing_method(self):
        data = {"jsonrpc": "3.0", "params": {}, "id": "123"}
        valid, error = validate_request(data)
        assert not valid
        assert "method" in error.lower()

    def test_validate_response_valid(self):
        data = {"jsonrpc": "3.0", "id": "123", "result": "ok"}
        valid, error = validate_response(data)
        assert valid

    def test_validate_response_both_result_and_error(self):
        data = {"jsonrpc": "3.0", "id": "123", "result": "ok", "error": {}}
        valid, error = validate_response(data)
        assert not valid


class TestBatch:
    def test_batch_creation(self):
        items = [Request("echo", {}), Request("ping", {})]
        batch_data = batch(items)
        assert len(batch_data) == 2

    def test_serialize(self):
        req = request("echo", {"msg": "hi"})
        json_str = serialize(req)
        assert "jsonrpc" in json_str
        assert "echo" in json_str
