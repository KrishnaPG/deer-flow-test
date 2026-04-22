"""FastAPI WebSocket server for berg10-agent."""

from __future__ import annotations

import json
import logging
from contextlib import asynccontextmanager
from typing import Any

from fastapi import FastAPI, WebSocket, WebSocketDisconnect
from fastapi.middleware.cors import CORSMiddleware

from .config import AgentConfig
from .constants import ErrorCode, MessageType
from .flow import create_agent_flow
from .models import ModelConfigRegistry
from .models.api import create_models_router
from .protocol import (
    json_rpc_error,
    message_done,
    message_error,
)
from .session import SessionManager
from .utils.memory import load_memory
from .utils.skills import load_skills
from .validators import ValidatorRegistry
from .validators.operation import DangerousCommandValidator, PathTraversalValidator

logger = logging.getLogger("berg10_agent")


def create_app(config: AgentConfig | None = None) -> FastAPI:
    """Create the FastAPI application."""
    if config is None:
        config = AgentConfig.from_env()

    session_mgr = SessionManager()
    validator_registry = _build_validator_registry(config)
    model_registry = ModelConfigRegistry()

    @asynccontextmanager
    async def lifespan(app: FastAPI):
        logger.info("Berg10 Agent starting on %s:%d", config.host, config.port)
        yield
        logger.info("Berg10 Agent shutting down")

    app = FastAPI(
        title="Berg10 Agent",
        description="Headless LLM agent with WebSocket transport",
        lifespan=lifespan,
    )

    app.add_middleware(
        CORSMiddleware,
        allow_origins=["*"],
        allow_methods=["*"],
        allow_headers=["*"],
    )

    app.state.config = config
    app.state.sessions = session_mgr
    app.state.validators = validator_registry
    app.state.models = model_registry

    # Include model management REST API routes
    app.include_router(create_models_router(model_registry))

    @app.get("/health")
    async def health():
        return {"status": "ok", "model": config.model}

    @app.websocket("/ws")
    async def websocket_endpoint(websocket: WebSocket):
        await websocket.accept()
        session = session_mgr.create()
        logger.info("WebSocket connected, session: %s", session.session_id)

        try:
            while True:
                raw = await websocket.receive_text()
                msg = _parse_message(raw)

                if msg is None:
                    await websocket.send_text(
                        json.dumps(json_rpc_error(ErrorCode.PARSE_ERROR, "Invalid JSON"))
                    )
                    continue

                await _handle_message(
                    websocket, msg, session, config, model_registry, validator_registry
                )

        except WebSocketDisconnect:
            logger.info("WebSocket disconnected, session: %s", session.session_id)
        finally:
            session_mgr.remove(session.session_id)

    return app


def _build_validator_registry(config: AgentConfig) -> ValidatorRegistry:
    registry = ValidatorRegistry()
    if config.enable_validators:
        registry.register("path_traversal", PathTraversalValidator(work_dir=config.work_dir))
        registry.register("dangerous_commands", DangerousCommandValidator())
    return registry


def _parse_message(raw: str) -> dict[str, Any] | None:
    try:
        data = json.loads(raw)
        return data if isinstance(data, dict) else None
    except (json.JSONDecodeError, ValueError):
        return None


async def _handle_message(
    websocket: WebSocket,
    msg: dict[str, Any],
    session: Any,
    config: AgentConfig,
    model_registry: ModelConfigRegistry,
    validators: ValidatorRegistry,
) -> None:
    msg_type = msg.get("type", "")
    content = msg.get("content", "")

    if msg_type == MessageType.MESSAGE.value and content:
        # Extract mode and model_id from message
        mode = msg.get("mode", "build")
        model_id = msg.get("model_id")

        # Update session's current model for this mode if provided
        if model_id:
            session.current_models[mode] = model_id
            model_registry.set_mode_default(mode, model_id)

        # Resolve model: session current > mode default
        resolved_model_id = session.current_models.get(mode) or model_registry.get_mode_default(
            mode
        )
        model_config = model_registry.get(resolved_model_id) if resolved_model_id else None

        if not model_config:
            await websocket.send_text(
                json.dumps(
                    json_rpc_error(
                        ErrorCode.CONFIG_ERROR,
                        f"No model configured for mode: {mode}. Please select a model first.",
                    )
                )
            )
            return

        # Build shared state for the flow
        history = list(session.history)
        history.append({"role": "user", "content": content})

        shared: dict[str, Any] = {
            "history": history,
            "work_dir": config.work_dir,
            "model_config": model_config,
            "mode": mode,
        }

        # Load memory and skills
        if config.enable_memory:
            shared["memory_content"] = load_memory(config.work_dir)
        if config.enable_skills:
            shared["skills_content"] = load_skills(config.work_dir)

        try:
            flow = create_agent_flow(config)
            await flow.run_async(shared)

            # Send final answer if available
            last_content = shared.get("last_content", "")
            if last_content:
                await websocket.send_text(json.dumps(message_done({"answer": last_content})))

            # Update session history
            session.history = shared.get("history", history)

        except Exception as e:
            logger.exception("Flow execution error")
            await websocket.send_text(json.dumps(message_error(str(e))))

    elif msg_type == MessageType.INTERRUPT.value:
        # Cancel current operation
        session.cancelled = True
        await websocket.send_text(
            json.dumps({"type": MessageType.ACK.value, "action": "interrupted"})
        )
