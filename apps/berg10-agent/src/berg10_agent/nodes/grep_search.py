"""Search file contents using regex patterns."""

from __future__ import annotations

import re
from pathlib import Path
from typing import Any

from pocketflow import AsyncNode


class GrepSearchNode(AsyncNode):
    """Regex-based content search across files."""

    def __init__(self, work_dir: str = ".", max_results: int = 100) -> None:
        super().__init__()
        self._work_dir = Path(work_dir).resolve()
        self._max_results = max_results

    async def prep(self, shared: dict[str, Any]) -> dict[str, Any]:
        tc = shared.get("current_tool_call", {})
        args = tc.arguments if hasattr(tc, "arguments") else tc.get("arguments", {})
        return {
            "pattern": args.get("pattern", ""),
            "path": args.get("path", "."),
            "glob": args.get("glob", "*"),
        }

    async def exec(self, prep_res: dict[str, Any]) -> dict[str, Any]:
        pattern = prep_res["pattern"]
        if not pattern:
            return {"error": "Search pattern is required"}

        target = (self._work_dir / prep_res["path"]).resolve()
        if not str(target).startswith(str(self._work_dir)):
            return {"error": f"Path escapes work directory: {prep_res['path']}"}

        try:
            compiled = re.compile(pattern)
        except re.error as e:
            return {"error": f"Invalid regex pattern: {e}"}

        results: list[dict[str, Any]] = []
        import fnmatch

        for file_path in target.rglob("*"):
            if not file_path.is_file():
                continue
            if not fnmatch.fnmatch(file_path.name, prep_res["glob"]):
                continue
            try:
                for line_num, line in enumerate(
                    file_path.read_text(errors="replace").splitlines(), 1
                ):
                    if compiled.search(line):
                        rel = file_path.relative_to(self._work_dir)
                        results.append(
                            {
                                "file": str(rel),
                                "line": line_num,
                                "content": line.rstrip(),
                            }
                        )
                        if len(results) >= self._max_results:
                            return {"results": results, "truncated": True}
            except (PermissionError, OSError):
                continue

        return {"results": results, "truncated": False}

    async def post(
        self, shared: dict[str, Any], prep_res: dict[str, Any], exec_res: dict[str, Any]
    ) -> str:
        tc = shared.get("current_tool_call", {})
        tool_name = tc.name if hasattr(tc, "name") else tc.get("name", "grep_search")
        content = exec_res.get("error") or str(exec_res.get("results", []))

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
