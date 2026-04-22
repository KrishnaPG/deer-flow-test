"""Terminal node that marks task completion."""

from __future__ import annotations

from typing import Any

from pocketflow import AsyncNode


class DoneNode(AsyncNode):
    """Terminal node for when the agent provides a final answer."""

    async def prep_async(self, shared: dict[str, Any]) -> dict[str, Any]:
        return {}

    async def exec_async(self, prep_res: dict[str, Any]) -> dict[str, Any]:
        return {}

    async def post_async(
        self, shared: dict[str, Any], prep_res: dict[str, Any], exec_res: dict[str, Any]
    ) -> str:
        return "done"
