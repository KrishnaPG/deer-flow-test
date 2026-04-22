"""Decide the next agent action using LLM with streaming support."""

from __future__ import annotations

from collections.abc import Callable
from typing import TYPE_CHECKING, Any

from pocketflow import AsyncNode

from ..constants import ToolName
from ..llm import LLMClient
from ..llm.client import ChatMessage

if TYPE_CHECKING:
    from ..models import ModelConfig

TOOL_DEFINITIONS = [
    {
        "type": "function",
        "function": {
            "name": ToolName.LIST_FILES.value,
            "description": "List files and directories in a given path.",
            "parameters": {
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Directory path to list",
                        "default": ".",
                    },
                    "pattern": {
                        "type": "string",
                        "description": "Glob pattern filter",
                        "default": "*",
                    },
                },
            },
        },
    },
    {
        "type": "function",
        "function": {
            "name": ToolName.GREP_SEARCH.value,
            "description": "Search file contents using regex pattern.",
            "parameters": {
                "type": "object",
                "properties": {
                    "pattern": {"type": "string", "description": "Regex pattern to search"},
                    "path": {"type": "string", "description": "Path to search in", "default": "."},
                    "glob": {"type": "string", "description": "File glob filter", "default": "*"},
                },
                "required": ["pattern"],
            },
        },
    },
    {
        "type": "function",
        "function": {
            "name": ToolName.READ_FILE.value,
            "description": "Read the contents of a file.",
            "parameters": {
                "type": "object",
                "properties": {
                    "path": {"type": "string", "description": "File path to read"},
                    "start_line": {
                        "type": "integer",
                        "description": "Start line number (1-indexed)",
                    },
                    "end_line": {"type": "integer", "description": "End line number (inclusive)"},
                },
                "required": ["path"],
            },
        },
    },
    {
        "type": "function",
        "function": {
            "name": ToolName.PATCH_FILE.value,
            "description": "Apply a unified diff patch to a file.",
            "parameters": {
                "type": "object",
                "properties": {
                    "path": {"type": "string", "description": "File to patch"},
                    "diff": {"type": "string", "description": "Unified diff content"},
                },
                "required": ["path", "diff"],
            },
        },
    },
    {
        "type": "function",
        "function": {
            "name": ToolName.RUN_COMMAND.value,
            "description": "Run a shell command.",
            "parameters": {
                "type": "object",
                "properties": {
                    "command": {"type": "string", "description": "Shell command to execute"},
                    "timeout": {"type": "integer", "description": "Timeout in seconds"},
                },
                "required": ["command"],
            },
        },
    },
]


class DecideActionNode(AsyncNode):
    """LLM-powered decision node that picks the next action with streaming."""

    def __init__(
        self,
        llm_client: LLMClient,
        system_prompt: str = "",
        tools: list[dict] | None = None,
    ) -> None:
        super().__init__()
        self.llm = llm_client
        self.system_prompt = system_prompt or self._default_system_prompt()
        self.tools = tools or TOOL_DEFINITIONS

    def _default_system_prompt(self) -> str:
        return (
            "You are an autonomous coding agent. Analyze the user's request and current "
            "context, then decide what to do next. You can use tools to explore and modify "
            "code, or provide a final answer when done. Always think step by step."
        )

    async def prep_async(self, shared: dict[str, Any]) -> dict[str, Any]:
        history = shared.get("history", [])
        memory = shared.get("memory_content", "")
        skills = shared.get("skills_content", "")
        model_config: ModelConfig | None = shared.get("model_config")

        messages = [ChatMessage(role="system", content=self.system_prompt)]
        if memory:
            messages.append(ChatMessage(role="system", content=f"[User Memory]\n{memory}"))
        if skills:
            messages.append(ChatMessage(role="system", content=f"[Skills]\n{skills}"))
        messages.extend(history)
        return {"messages": messages, "model_config": model_config}

    async def exec_async(self, prep_res: dict[str, Any]) -> dict[str, Any]:
        model_config: ModelConfig | None = prep_res.get("model_config")
        result = await self.llm.complete(
            messages=prep_res["messages"],
            tools=self.tools,
            model_config=model_config,
        )
        return {
            "content": result.content,
            "tool_calls": result.tool_calls,
            "finish_reason": result.finish_reason,
            "model_config": model_config,
        }

    async def exec_stream(
        self,
        prep_res: dict[str, Any],
        emit: Callable[[str], Any] | None = None,
    ) -> dict[str, Any]:
        """Streaming execution - emits tokens as they arrive."""
        model_config: ModelConfig | None = prep_res.get("model_config")
        content_parts: list[str] = []
        async for chunk in self.llm.stream(
            messages=prep_res["messages"],
            tools=self.tools,
            model_config=model_config,
        ):
            if chunk.content:
                content_parts.append(chunk.content)
                if emit:
                    await emit(chunk.content)

        full_content = "".join(content_parts)
        return {
            "content": full_content,
            "tool_calls": [],
            "finish_reason": "stop",
            "model_config": model_config,
        }

    async def post_async(
        self, shared: dict[str, Any], prep_res: dict[str, Any], exec_res: dict[str, Any]
    ) -> str:
        content = exec_res["content"]
        tool_calls = exec_res["tool_calls"]
        model_config: ModelConfig | None = exec_res.get("model_config")

        # Append assistant message to history with model_id
        history = shared.get("history", [])
        assistant_msg: dict[str, Any] = {
            "role": "assistant",
            "content": content,
        }
        if model_config:
            assistant_msg["model_id"] = model_config.id
        if tool_calls:
            assistant_msg["tool_calls"] = [
                {"id": tc.id, "name": tc.name, "arguments": tc.arguments} for tc in tool_calls
            ]
        history.append(assistant_msg)

        shared["history"] = history
        shared["last_content"] = content
        shared["last_tool_calls"] = tool_calls

        if tool_calls:
            # Route to the first tool
            tc = tool_calls[0]
            shared["current_tool_call"] = tc
            return f"tool_{tc.name}"

        return "answer"
