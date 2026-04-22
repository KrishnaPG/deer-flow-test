"""Tests for JSON-RPC 3.0 handler and server modules."""

import pytest
import asyncio

from jsonrpc.core import (
    ErrorCode,
    Request,
    RequestId,
    Response,
    StreamResponse,
    request,
)
from jsonrpc.handler import (
    CancellationError,
    CancellableRequest,
    Registry,
    StreamingDispatcher,
    logging_middleware,
    validation_middleware,
)
from jsonrpc.server import Client, Server
from jsonrpc.transport import InMemoryTransport


class TestRegistry:
    @pytest.fixture
    def registry(self):
        return Registry()

    def test_register_handler(self, registry):
        def handler(x: int) -> int:
            return x * 2

        registry.register("double", handler)
        assert "double" in registry.list_methods()

    def test_unregister_handler(self, registry):
        def handler():
            pass

        registry.register("test", handler)
        registry.unregister("test")
        assert "test" not in registry.list_methods()

    def test_method_decorator(self, registry):
        @registry.method("echo")
        def echo(message: str) -> str:
            return message

        assert "echo" in registry.list_methods()

    def test_method_decorator_auto_name(self, registry):
        @registry.method()
        def my_handler():
            pass

        assert "my_handler" in registry.list_methods()

    @pytest.mark.asyncio
    async def test_dispatch_sync_handler(self, registry):
        @registry.method("add")
        def add(a: int, b: int) -> int:
            return a + b

        req = Request("add", {"a": 2, "b": 3}, RequestId("123"))
        response = await registry.dispatch(req)

        assert response.is_success
        assert response.result == 5

    @pytest.mark.asyncio
    async def test_dispatch_async_handler(self, registry):
        @registry.method("async_echo")
        async def async_echo(message: str) -> str:
            return message

        req = Request("async_echo", {"message": "hello"}, RequestId("123"))
        response = await registry.dispatch(req)

        assert response.is_success
        assert response.result == "hello"

    @pytest.mark.asyncio
    async def test_dispatch_method_not_found(self, registry):
        req = Request("unknown", {}, RequestId("123"))
        response = await registry.dispatch(req)

        assert response.is_error
        assert response.error.code == ErrorCode.METHOD_NOT_FOUND

    @pytest.mark.asyncio
    async def test_dispatch_invalid_params(self, registry):
        @registry.method("needs_param")
        def needs_param(required: str) -> str:
            return required

        req = Request("needs_param", {}, RequestId("123"))
        response = await registry.dispatch(req)

        assert response.is_error
        assert response.error.code == ErrorCode.INVALID_PARAMS


class TestMiddleware:
    @pytest.fixture
    def registry(self):
        reg = Registry()

        @reg.method("echo")
        def echo(message: str) -> str:
            return message

        return reg

    @pytest.mark.asyncio
    async def test_logging_middleware(self, registry, capsys):
        registry.add_middleware(logging_middleware)

        req = Request("echo", {"message": "test"}, RequestId("123"))
        response = await registry.dispatch(req)

        assert response.is_success
        captured = capsys.readouterr()
        assert "echo" in captured.out

    @pytest.mark.asyncio
    async def test_validation_middleware(self, registry):
        registry.add_middleware(validation_middleware)

        # Valid request
        req = Request("echo", {"message": "test"}, RequestId("123"))
        response = await registry.dispatch(req)
        assert response.is_success

    @pytest.mark.asyncio
    async def test_validation_middleware_missing_method(self, registry):
        registry.add_middleware(validation_middleware)

        # Invalid request - empty method
        req = Request("", {}, RequestId("123"))
        response = await registry.dispatch(req)
        assert response.is_error


class TestStreamingDispatcher:
    @pytest.fixture
    def registry(self):
        reg = Registry()
        from jsonrpc.core import stream_data

        @reg.method("stream_count", streaming=True)
        async def stream_count(n: int, _emit, _request_id):
            for i in range(n):
                await _emit(stream_data(_request_id, str(i)))
            return {"count": n}

        return reg

    @pytest.fixture
    def dispatcher(self, registry):
        return StreamingDispatcher(registry)

    @pytest.mark.asyncio
    async def test_stream_dispatch(self, dispatcher):
        emitted = []

        async def emit(chunk):
            emitted.append(chunk)

        req = Request("stream_count", {"n": 3}, RequestId("123"))
        response = await dispatcher.dispatch_stream(req, emit)

        # Should have: 3 data chunks, 1 done chunk
        assert len(emitted) == 4
        assert response is None  # returns None because stream finishes with StreamDone
        assert emitted[-1].is_done
        assert emitted[-1].result == {"count": 3}

    @pytest.mark.asyncio
    async def test_stream_not_found(self, dispatcher):
        async def emit(chunk):
            pass

        req = Request("unknown", {}, RequestId("123"))
        response = await dispatcher.dispatch_stream(req, emit)

        assert response.is_error
        assert response.error.code == ErrorCode.METHOD_NOT_FOUND


class TestCancellableRequest:
    def test_initially_not_cancelled(self):
        req = Request("test", {}, RequestId("123"))
        cancellable = CancellableRequest(req)
        assert not cancellable.is_cancelled

    def test_cancel(self):
        req = Request("test", {}, RequestId("123"))
        cancellable = CancellableRequest(req)
        cancellable.cancel("user request")

        assert cancellable.is_cancelled
        assert cancellable.cancel_reason == "user request"

    def test_check_cancelled_raises(self):
        req = Request("test", {}, RequestId("123"))
        cancellable = CancellableRequest(req)
        cancellable.cancel()

        with pytest.raises(CancellationError):
            cancellable.check_cancelled()


class TestServer:
    @pytest.fixture
    def server(self):
        return Server()

    def test_method_registration(self, server):
        @server.method("echo")
        async def echo(message: str) -> str:
            return message

        assert "echo" in server.registry.list_methods()

    def test_streaming_registration(self, server):
        @server.method("stream", streaming=True)
        async def stream(_emit, _request_id):
            pass

        handler = server.registry.get_handler("stream")
        assert handler.is_streaming

    @pytest.mark.asyncio
    async def test_handle_request(self, server):
        @server.method("add")
        async def add(a: int, b: int) -> int:
            return a + b

        req = Request("add", {"a": 1, "b": 2}, RequestId("123"))
        response = await server._handle_message(req)

        assert isinstance(response, Response)
        assert response.result == 3

    @pytest.mark.asyncio
    async def test_handle_cancellation_options(self, server):
        # Track a dummy request
        server._cancellable_requests["123"] = CancellableRequest(
            Request("slow_op", {}, RequestId("123"))
        )

        # Cancel it via options
        cancel_req = Request(
            "does_not_matter", {}, RequestId("999"), options={"abort": True, "stream": "123"}
        )
        response = await server._handle_message(cancel_req)

        assert response.is_success
        assert response.result["cancelled"]

    @pytest.mark.asyncio
    async def test_handle_cancellation_method(self, server):
        # Track a dummy request
        server._cancellable_requests["123"] = CancellableRequest(
            Request("slow_op", {}, RequestId("123"))
        )

        # Cancel it via method
        cancel_req = Request("request.cancel", {"id": "123"}, RequestId("999"))
        response = await server._handle_message(cancel_req)

        assert response.is_success
        assert response.result["cancelled"]


class TestClient:
    @pytest.fixture
    async def client_pair(self):
        """Create a pair of connected in-memory transports."""
        transport_a = InMemoryTransport()
        transport_b = InMemoryTransport()
        transport_a.pair_with(transport_b)

        # Create server on one side
        server = Server()

        @server.method("echo")
        async def echo(message: str) -> str:
            return message

        conn_server = server.add_connection(transport_a)

        # Create client on other side
        from jsonrpc.transport import Connection

        conn_client = Connection(transport_b)
        client = Client(conn_client)


        server_task = asyncio.create_task(conn_server.start())
        client_task = asyncio.create_task(client.connect())
        # Let tasks start
        await asyncio.sleep(0)

        yield client, server

        await client.disconnect()
        await server.stop()
        server_task.cancel()
        client_task.cancel()
        try:
            await server_task
            await client_task
        except asyncio.CancelledError:
            pass

    @pytest.mark.asyncio
    async def test_client_call(self, client_pair):
        client, server = client_pair
        response = await client.call("echo", {"message": "hello"})
        assert response.result == "hello"

    @pytest.mark.asyncio
    async def test_client_notify(self, client_pair):
        client, server = client_pair
        # Notify should not raise
        await client.notify("echo", {"message": "test"})


class TestIntegration:
    @pytest.mark.asyncio
    async def test_full_request_response_cycle(self):
        """Test a complete request-response cycle."""
        transport_a = InMemoryTransport()
        transport_b = InMemoryTransport()
        transport_a.pair_with(transport_b)

        # Server setup
        server = Server()

        @server.method("greet")
        async def greet(name: str) -> str:
            return f"Hello, {name}!"

        from jsonrpc.core import stream_data

        @server.method("stream_hello", streaming=True)
        async def stream_hello(name: str, _emit, _request_id):
            await _emit(stream_data(_request_id, "Hello"))
            await _emit(stream_data(_request_id, name))

        conn_server = server.add_connection(transport_a)

        # Client setup
        from jsonrpc.transport import Connection

        conn_client = Connection(transport_b)
        client = Client(conn_client)

        # Run server in background

        server_task = asyncio.create_task(conn_server.start())
        client_task = asyncio.create_task(client.connect())
        # Let tasks start
        await asyncio.sleep(0)

        # Make request
        response = await client.call("greet", {"name": "World"})
        assert response.result == "Hello, World!"

        # Stream request
        chunks = []
        async for chunk in client.stream("stream_hello", {"name": "World"}):
            if chunk.is_data:
                chunks.append(chunk.data)

        assert chunks == ["Hello", "World"]

        # Cleanup
        await client.disconnect()
        await conn_server.stop()
        server_task.cancel()
        client_task.cancel()
        try:
            await server_task
            await client_task
        except asyncio.CancelledError:
            pass
