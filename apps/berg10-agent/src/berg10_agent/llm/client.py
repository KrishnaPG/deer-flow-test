"""LiteLLM wrapper with streaming support for multi-provider routing."""

from __future__ import annotations

from dataclasses import dataclass, field
from typing import Any, AsyncIterator, Optional


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

    def _build_kwargs(self, **extra: Any) -> dict[str, Any]:
        kw: dict[str, Any] = {"model": self.model}
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
    ) -> CompletionResult:
        """Non-streaming completion."""
        import litellm

        msg_dicts = [_msg_to_dict(m) for m in messages]
        kw = self._build_kwargs(
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
    ) -> AsyncIterator[CompletionChunk]:
        """Streaming completion - yields chunks as they arrive."""
        import litellm

        msg_dicts = [_msg_to_dict(m) for m in messages]
        kw = self._build_kwargs(
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


def _msg_to_dict(msg: ChatMessage) -> dict[str, Any]:
    d: dict[str, Any] = {"role": msg.role, "content": msg.content}
    if msg.tool_call_id:
        d["tool_call_id"] = msg.tool_call_id
    if msg.name:
        d["name"] = msg.name
    return d


def _parse_tool_calls(choice: Any) -> list[ToolCall]:
    calls: list[ToolCall] = []
    for tc in choice.message.tool_calls:
        import json as _json

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
