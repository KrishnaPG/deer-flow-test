"""List files and directories in a given path."""

from __future__ import annotations

from pathlib import Path
from typing import Any

from pocketflow import AsyncNode


class ListFilesNode(AsyncNode):
    """List files and directories with optional glob pattern filtering."""

    def __init__(self, work_dir: str = ".") -> None:
        super().__init__()
        self._work_dir = Path(work_dir).resolve()

    async def prep(self, shared: dict[str, Any]) -> dict[str, Any]:
        tc = shared.get("current_tool_call", {})
        args = tc.arguments if hasattr(tc, "arguments") else tc.get("arguments", {})
        return {
            "path": args.get("path", "."),
            "pattern": args.get("pattern", "*"),
        }

    async def exec(self, prep_res: dict[str, Any]) -> dict[str, Any]:
        target = (self._work_dir / prep_res["path"]).resolve()

        if not str(target).startswith(str(self._work_dir)):
            return {"error": f"Path escapes work directory: {prep_res['path']}"}

        if not target.exists():
            return {"error": f"Path does not exist: {prep_res['path']}"}

        entries = []
        for item in sorted(target.iterdir()):
            if item.is_dir():
                entries.append({"name": item.name, "type": "directory"})
            else:
                entries.append({"name": item.name, "type": "file", "size": item.stat().st_size})

        if prep_res["pattern"] != "*":
            import fnmatch

            entries = [e for e in entries if fnmatch.fnmatch(e["name"], prep_res["pattern"])]

        return {"entries": entries, "path": str(target.relative_to(self._work_dir))}

    async def post(
        self, shared: dict[str, Any], prep_res: dict[str, Any], exec_res: dict[str, Any]
    ) -> str:
        tc = shared.get("current_tool_call", {})
        tool_name = tc.name if hasattr(tc, "name") else tc.get("name", "list_files")
        tool_result = exec_res.get("error") or exec_res.get("entries", [])

        history = shared.get("history", [])
        history.append(
            {
                "role": "tool",
                "tool_call_id": tc.id if hasattr(tc, "id") else tc.get("id", ""),
                "name": tool_name,
                "content": str(tool_result),
            }
        )
        shared["history"] = history
        shared["last_tool_result"] = exec_res
        return "decide"
