"""Compact conversation history when approaching token limits."""

from __future__ import annotations

from typing import Any

from pocketflow import AsyncNode

from ..llm import LLMClient


class CompactHistoryNode(AsyncNode):
    """Summarize older messages when history grows too large."""

    def __init__(
        self,
        llm_client: LLMClient,
        max_tokens: int = 8000,
        compact_threshold: int = 6000,
    ) -> None:
        super().__init__()
        self.llm = llm_client
        self.max_tokens = max_tokens
        self.compact_threshold = compact_threshold

    async def prep(self, shared: dict[str, Any]) -> dict[str, Any]:
        history = shared.get("history", [])
        token_count = _estimate_tokens(history)
        return {
            "history": history,
            "token_count": token_count,
            "needs_compaction": token_count > self.compact_threshold,
        }

    async def exec(self, prep_res: dict[str, Any]) -> dict[str, Any]:
        if not prep_res["needs_compaction"]:
            return {"compacted": False, "history": prep_res["history"]}

        history = prep_res["history"]
        # Keep recent messages, summarize older ones
        keep_count = max(2, len(history) // 3)
        recent = history[-keep_count:]
        older = history[:-keep_count]

        summary_msgs = [
            {
                "role": "system",
                "content": "Summarize the following conversation concisely, preserving key facts, decisions, and context.",
            },
            *[_msg_dict(m) for m in older],
            {"role": "user", "content": "Provide a concise summary of the conversation above."},
        ]

        result = await self.llm.complete(
            messages=[_dict_to_msg(m) for m in summary_msgs],
            max_tokens=1024,
            temperature=0.3,
        )

        compacted = [
            {"role": "system", "content": f"[Conversation Summary]\n{result.content}"},
            *[_msg_dict(m) for m in recent],
        ]

        return {"compacted": True, "history": compacted}

    async def post(
        self, shared: dict[str, Any], prep_res: dict[str, Any], exec_res: dict[str, Any]
    ) -> str:
        shared["history"] = exec_res["history"]
        shared["history_compacted"] = exec_res["compacted"]
        return "compacted" if exec_res["compacted"] else "ok"


def _estimate_tokens(history: list[dict]) -> int:
    return sum(len(str(m.get("content", ""))) for m in history) // 4


def _msg_dict(msg: Any) -> dict:
    if isinstance(msg, dict):
        return msg
    return {"role": getattr(msg, "role", "user"), "content": getattr(msg, "content", str(msg))}


def _dict_to_msg(d: dict):
    from ..llm.client import ChatMessage

    return ChatMessage(role=d.get("role", "user"), content=d.get("content", ""))
