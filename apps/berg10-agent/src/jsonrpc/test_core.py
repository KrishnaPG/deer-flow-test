"""Tests for JSON-RPC 3.0 core module."""

import pytest

from jsonrpc.core import (
    ErrorCode,
    JsonRpcError,
    Request,
    RequestId,
    Response,
    StreamResponse,
    batch,
    parse_json,
    parse_message,
    request,
    serialize,
    stream_data,
    stream_done,
    stream_error,
    success_response,
    error_response,
    ack_response,
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
        assert len(str(req_id1.value)) == 36  # UUID length

    def test_string_conversion(self):
        req_id = RequestId("abc")
        assert str(req_id) == "abc"

    def test_hashable(self):
        req_id = RequestId("test")
        assert hash(req_id) == hash("test")

    def test_equality(self):
        assert RequestId("123") == RequestId("123")
        assert RequestId("123") == "123"
        assert RequestId(123) == 123


class TestJsonRpcError:
    def test_to_dict(self):
        error = JsonRpcError(ErrorCode.INVALID_PARAMS, "Bad params", "Title", {"field": "name"})
        assert error.to_dict() == {
            "code": -32602,
            "message": "Bad params",
            "title": "Title",
            "data": {"field": "name"},
        }

    def test_from_dict(self):
        data = {"code": -32601, "message": "Not found", "title": "Not Found", "data": {}}
        error = JsonRpcError.from_dict(data)
        assert error.code == ErrorCode.METHOD_NOT_FOUND
        assert error.message == "Not found"
        assert error.title == "Not Found"


class TestRequest:
    def test_to_dict(self):
        req = Request("echo", {"message": "hello"}, RequestId("123"), options={"stream": True})
        assert req.to_dict() == {
            "jsonrpc": "3.0",
            "method": "echo",
            "params": {"message": "hello"},
            "id": "123",
            "options": {"stream": True},
        }

    def test_from_dict(self):
        data = {
            "jsonrpc": "3.0",
            "method": "echo",
            "params": {"message": "hello"},
            "id": "456",
            "options": {"stream": True},
        }
        req = Request.from_dict(data)
        assert req.method == "echo"
        assert req.params == {"message": "hello"}
        assert req.id == "456"
        assert req.options == {"stream": True}


class TestResponse:
    def test_success_response(self):
        resp = Response.success("123", {"result": "ok"})
        assert resp.is_success
        assert not resp.is_error
        assert not resp.is_ack
        assert resp.result == {"result": "ok"}
        assert resp.error is None

    def test_error_response(self):
        resp = Response.error_response("123", ErrorCode.METHOD_NOT_FOUND, "Not found")
        assert not resp.is_success
        assert resp.is_error
        assert not resp.is_ack
        assert resp.error.code == ErrorCode.METHOD_NOT_FOUND

    def test_ack_response(self):
        resp = Response.ack_response("123", {"progress": 50})
        assert not resp.is_success
        assert not resp.is_error
        assert resp.is_ack
        assert resp.ack == {"progress": 50}

    def test_validation(self):
        with pytest.raises(ValueError):
            Response(
                id=RequestId("123"),
                result="ok",
                error=JsonRpcError(ErrorCode.INTERNAL_ERROR, "err"),
            )

    def test_to_dict_success(self):
        resp = Response.success("123", "hello")
        assert resp.to_dict() == {
            "jsonrpc": "3.0",
            "id": "123",
            "result": "hello",
        }

    def test_from_dict(self):
        data = {"jsonrpc": "3.0", "id": "789", "result": "success"}
        resp = Response.from_dict(data)
        assert resp.is_success
        assert resp.result == "success"


class TestStreamMessages:
    def test_stream_data_to_dict(self):
        data_chunk = stream_data("s123", "partial")
        data = data_chunk.to_dict()
        assert data["stream"] == {"id": "s123"}
        assert data["data"] == "partial"
        assert "result" not in data
        assert "error" not in data

    def test_stream_done_to_dict(self):
        done = stream_done("s123", "final")
        data = done.to_dict()
        assert data["stream"] == {"id": "s123"}
        assert data["result"] == "final"
        assert "data" not in data

    def test_stream_error_to_dict(self):
        err = stream_error("s123", ErrorCode.INTERNAL_ERROR, "oops")
        data = err.to_dict()
        assert data["stream"] == {"id": "s123"}
        assert data["error"]["code"] == -32603

    def test_validation(self):
        with pytest.raises(ValueError):
            StreamResponse(stream={"id": "123"}, data="a", result="b")
        with pytest.raises(ValueError):
            StreamResponse(stream={}, data="a")  # missing id


class TestMessageBuilders:
    def test_request_builder(self):
        req = request("echo", {"msg": "hi"}, "id-1", options={"stream": True})
        assert req.method == "echo"
        assert req.params == {"msg": "hi"}
        assert req.options == {"stream": True}

    def test_success_response_builder(self):
        resp = success_response("id-1", "result")
        assert resp.is_success
        assert resp.result == "result"


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

    def test_parse_stream(self):
        data = {"jsonrpc": "3.0", "stream": {"id": "123"}, "data": "hello"}
        msg = parse_message(data)
        assert isinstance(msg, StreamResponse)
        assert msg.data == "hello"

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
