"""Tests for JSON-RPC 3.0 handler and server modules."""

import pytest

from berg10_agent.jsonrpc.core import (
    ErrorCode,
    Request,
    RequestId,
    Response,
    StreamChunk,
    StreamType,
)
from berg10_agent.jsonrpc.handler import (
    CancellationError,
    CancellableRequest,
    Registry,
    StreamingDispatcher,
    logging_middleware,
    validation_middleware,
)
from berg10_agent.jsonrpc.server import Client, Server
from berg10_agent.jsonrpc.transport import InMemoryTransport


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

        request = Request("add", {"a": 2, "b": 3}, RequestId("123"))
        response = await registry.dispatch(request)

        assert response.is_success
        assert response.result == 5

    @pytest.mark.asyncio
    async def test_dispatch_async_handler(self, registry):
        @registry.method("async_echo")
        async def async_echo(message: str) -> str:
            return message

        request = Request("async_echo", {"message": "hello"}, RequestId("123"))
        response = await registry.dispatch(request)

        assert response.is_success
        assert response.result == "hello"

    @pytest.mark.asyncio
    async def test_dispatch_method_not_found(self, registry):
        request = Request("unknown", {}, RequestId("123"))
        response = await registry.dispatch(request)

        assert response.is_error
        assert response.error.code == ErrorCode.METHOD_NOT_FOUND

    @pytest.mark.asyncio
    async def test_dispatch_invalid_params(self, registry):
        @registry.method("needs_param")
        def needs_param(required: str) -> str:
            return required

        request = Request("needs_param", {}, RequestId("123"))
        response = await registry.dispatch(request)

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

        request = Request("echo", {"message": "test"}, RequestId("123"))
        response = await registry.dispatch(request)

        assert response.is_success
        captured = capsys.readouterr()
        assert "echo" in captured.out

    @pytest.mark.asyncio
    async def test_validation_middleware(self, registry):
        registry.add_middleware(validation_middleware)

        # Valid request
        request = Request("echo", {"message": "test"}, RequestId("123"))
        response = await registry.dispatch(request)
        assert response.is_success

    @pytest.mark.asyncio
    async def test_validation_middleware_missing_method(self, registry):
        registry.add_middleware(validation_middleware)

        # Invalid request - empty method
        request = Request("", {}, RequestId("123"))
        response = await registry.dispatch(request)
        assert response.is_error


class TestStreamingDispatcher:
    @pytest.fixture
    def registry(self):
        reg = Registry()

        @reg.method("stream_count", streaming=True)
        async def stream_count(n: int, _emit, _request_id):
            for i in range(n):
                await _emit(StreamChunk(_request_id, str(i)))
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

        request = Request("stream_count", {"n": 3}, RequestId("123"))
        response = await dispatcher.dispatch_stream(request, emit)

        # Should have: stream_start, 3 chunks, stream_end
        assert len(emitted) >= 4
        assert response is not None
        assert response.result == {"count": 3}

    @pytest.mark.asyncio
    async def test_stream_not_found(self, dispatcher):
        async def emit(chunk):
            pass

        request = Request("unknown", {}, RequestId("123"))
        response = await dispatcher.dispatch_stream(request, emit)

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

        request = Request("add", {"a": 1, "b": 2}, RequestId("123"))
        response = await server._handle_message(request)

        assert isinstance(response, Response)
        assert response.result == 3

    @pytest.mark.asyncio
    async def test_handle_cancellation(self, server):
        # First make a request to track it
        @server.method("slow_op")
        async def slow_op():
            return "done"

        request = Request("slow_op", {}, RequestId("123"))
        # Manually add to cancellable requests
        from berg10_agent.jsonrpc.handler import CancellableRequest

        server._cancellable_requests["123"] = CancellableRequest(request)

        # Now cancel it
        from berg10_agent.jsonrpc.core import Cancel

        cancel_msg = Cancel(RequestId("123"), "test")
        response = await server._handle_message(cancel_msg)

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
        from berg10_agent.jsonrpc.transport import Connection

        conn_client = Connection(transport_b)
        client = Client(conn_client)

        await conn_server.start()
        await client.connect()

        yield client, server

        await client.disconnect()
        await server.stop()

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

        conn_server = server.add_connection(transport_a)

        # Client setup
        from berg10_agent.jsonrpc.transport import Connection

        conn_client = Connection(transport_b)
        client = Client(conn_client)

        # Run server in background
        import asyncio

        server_task = asyncio.create_task(conn_server.start())

        await client.connect()

        # Make request
        response = await client.call("greet", {"name": "World"})
        assert response.result == "Hello, World!"

        # Cleanup
        await client.disconnect()
        await conn_server.stop()
        server_task.cancel()
        try:
            await server_task
        except asyncio.CancelledError:
            pass
