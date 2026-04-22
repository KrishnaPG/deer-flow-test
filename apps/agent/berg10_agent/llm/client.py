"""LiteLLM wrapper with streaming support for multi-provider routing."""

from __future__ import annotations

import json as _json
from collections.abc import AsyncIterator
from dataclasses import dataclass, field
from typing import TYPE_CHECKING, Any

import litellm

# Suppress LiteLLM debug output (Provider List messages, etc.)
litellm.suppress_debug_info = True

if TYPE_CHECKING:
    from ..models import ModelConfig


@dataclass
class ChatMessage:
    """A single chat message."""

    role: str
    content: str
    tool_call_id: str | None = None
    name: str | None = None


@dataclass
class CompletionChunk:
    """A streaming completion chunk."""

    content: str
    finish_reason: str | None = None


@dataclass
class ToolCall:
    """A tool call from the LLM."""

    id: str
    name: str
    arguments: dict[str, Any]


@dataclass
class CompletionResult:
    """Full completion result."""

    content: str
    tool_calls: list[ToolCall] = field(default_factory=list)
    usage: dict[str, int] = field(default_factory=dict)
    finish_reason: str | None = None


class LLMClient:
    """LiteLLM wrapper for multi-provider LLM routing with streaming support."""

    def __init__(
        self,
        model: str,
        api_key: str = "",
        base_url: str = "",
        api_base: str = "",
    ) -> None:
        self.model = model
        self.api_key = api_key
        self.base_url = base_url or api_base

    def _build_kwargs(
        self, model_config: ModelConfig | None = None, **extra: Any
    ) -> dict[str, Any]:
        """Build kwargs for LiteLLM completion call.

        If model_config is provided, use its settings. Otherwise fall back to defaults.
        """
        if model_config:
            kw: dict[str, Any] = {"model": model_config.model}
            if model_config.api_key:
                kw["api_key"] = model_config.api_key
            if model_config.base_url:
                kw["api_base"] = model_config.base_url
        else:
            kw = {"model": self.model}
            if self.api_key:
                kw["api_key"] = self.api_key
            if self.base_url:
                kw["api_base"] = self.base_url
        kw.update(extra)
        return kw

    async def complete(
        self,
        messages: list[ChatMessage],
        tools: list[dict] | None = None,
        temperature: float = 0.7,
        max_tokens: int = 4096,
        model_config: ModelConfig | None = None,
    ) -> CompletionResult:
        """Non-streaming completion."""

        msg_dicts = [_msg_to_dict(m) for m in messages]
        kw = self._build_kwargs(
            model_config=model_config,
            messages=msg_dicts,
            temperature=temperature,
            max_tokens=max_tokens,
        )
        if tools:
            kw["tools"] = tools

        response = await litellm.acompletion(**kw)
        choice = response.choices[0]
        tool_calls = _parse_tool_calls(choice) if choice.message.tool_calls else []
        return CompletionResult(
            content=choice.message.content or "",
            tool_calls=tool_calls,
            usage=_parse_usage(response),
            finish_reason=choice.finish_reason,
        )

    async def stream(
        self,
        messages: list[ChatMessage],
        tools: list[dict] | None = None,
        temperature: float = 0.7,
        max_tokens: int = 4096,
        model_config: ModelConfig | None = None,
    ) -> AsyncIterator[CompletionChunk]:
        """Streaming completion - yields chunks as they arrive."""

        msg_dicts = [_msg_to_dict(m) for m in messages]
        kw = self._build_kwargs(
            model_config=model_config,
            messages=msg_dicts,
            temperature=temperature,
            max_tokens=max_tokens,
            stream=True,
        )
        if tools:
            kw["tools"] = tools

        response = await litellm.acompletion(**kw)
        async for chunk in response:
            if chunk.choices:
                delta = chunk.choices[0].delta
                content = delta.content or ""
                finish = chunk.choices[0].finish_reason
                if content or finish:
                    yield CompletionChunk(content=content, finish_reason=finish)


def _msg_to_dict(msg: ChatMessage | dict[str, Any]) -> dict[str, Any]:
    if isinstance(msg, dict):
        return msg
    d: dict[str, Any] = {"role": msg.role, "content": msg.content}
    if msg.tool_call_id:
        d["tool_call_id"] = msg.tool_call_id
    if msg.name:
        d["name"] = msg.name
    return d


def _parse_tool_calls(choice: Any) -> list[ToolCall]:
    calls: list[ToolCall] = []
    for tc in choice.message.tool_calls:
        args = (
            _json.loads(tc.function.arguments)
            if isinstance(tc.function.arguments, str)
            else tc.function.arguments
        )
        calls.append(ToolCall(id=tc.id, name=tc.function.name, arguments=args))
    return calls


def _parse_usage(response: Any) -> dict[str, int]:
    usage = getattr(response, "usage", None)
    if not usage:
        return {}
    return {
        "prompt_tokens": getattr(usage, "prompt_tokens", 0),
        "completion_tokens": getattr(usage, "completion_tokens", 0),
        "total_tokens": getattr(usage, "total_tokens", 0),
    }
