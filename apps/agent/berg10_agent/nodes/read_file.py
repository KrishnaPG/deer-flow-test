"""Read file contents with optional line range."""

from __future__ import annotations

from pathlib import Path
from typing import Any

from pocketflow import AsyncNode


class ReadFileNode(AsyncNode):
    """Read file contents with optional start/end line selection."""

    def __init__(self, work_dir: str = ".", max_bytes: int = 1024 * 1024) -> None:
        super().__init__()
        self._work_dir = Path(work_dir).resolve()
        self._max_bytes = max_bytes

    async def prep_async(self, shared: dict[str, Any]) -> dict[str, Any]:
        tc = shared.get("current_tool_call", {})
        args = tc.arguments if hasattr(tc, "arguments") else tc.get("arguments", {})
        return {
            "path": args.get("path", ""),
            "start_line": args.get("start_line"),
            "end_line": args.get("end_line"),
        }

    async def exec_async(self, prep_res: dict[str, Any]) -> dict[str, Any]:
        file_path = prep_res["path"]
        if not file_path:
            return {"error": "File path is required"}

        resolved = (self._work_dir / file_path).resolve()
        if not str(resolved).startswith(str(self._work_dir)):
            return {"error": f"Path escapes work directory: {file_path}"}

        if not resolved.exists():
            return {"error": f"File does not exist: {file_path}"}

        if not resolved.is_file():
            return {"error": f"Path is not a file: {file_path}"}

        if resolved.stat().st_size > self._max_bytes:
            return {
                "error": f"File too large ({resolved.stat().st_size} bytes, max {self._max_bytes})"
            }

        try:
            content = resolved.read_text(errors="replace")
        except (PermissionError, OSError) as e:
            return {"error": f"Cannot read file: {e}"}

        lines = content.splitlines()
        start = (prep_res["start_line"] or 1) - 1
        end = prep_res["end_line"] or len(lines)

        selected = lines[max(0, start) : min(len(lines), end)]

        return {
            "content": "\n".join(selected),
            "total_lines": len(lines),
            "start_line": max(1, start + 1),
            "end_line": min(len(lines), end),
            "path": file_path,
        }

    async def post_async(
        self, shared: dict[str, Any], prep_res: dict[str, Any], exec_res: dict[str, Any]
    ) -> str:
        tc = shared.get("current_tool_call", {})
        tool_name = tc.name if hasattr(tc, "name") else tc.get("name", "read_file")
        content = exec_res.get("error") or exec_res.get("content", "")

        history = shared.get("history", [])
        history.append(
            {
                "role": "tool",
                "tool_call_id": tc.id if hasattr(tc, "id") else tc.get("id", ""),
                "name": tool_name,
                "content": content,
            }
        )
        shared["history"] = history
        shared["last_tool_result"] = exec_res
        return "decide"
