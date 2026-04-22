"""Apply unified diff patches to files."""

from __future__ import annotations

from pathlib import Path
from typing import Any

from pocketflow import AsyncNode


class PatchFileNode(AsyncNode):
    """Apply unified diff patches to files."""

    def __init__(self, work_dir: str = ".") -> None:
        super().__init__()
        self._work_dir = Path(work_dir).resolve()

    async def prep_async(self, shared: dict[str, Any]) -> dict[str, Any]:
        tc = shared.get("current_tool_call", {})
        args = tc.arguments if hasattr(tc, "arguments") else tc.get("arguments", {})
        return {
            "path": args.get("path", ""),
            "diff": args.get("diff", ""),
        }

    async def exec_async(self, prep_res: dict[str, Any]) -> dict[str, Any]:
        file_path = prep_res["path"]
        diff_content = prep_res["diff"]

        if not file_path:
            return {"error": "File path is required"}
        if not diff_content:
            return {"error": "Diff content is required"}

        resolved = (self._work_dir / file_path).resolve()
        if not str(resolved).startswith(str(self._work_dir)):
            return {"error": f"Path escapes work directory: {file_path}"}

        if not resolved.exists():
            return {"error": f"File does not exist: {file_path}"}

        try:
            original = resolved.read_text(errors="replace")
            patched = _apply_diff(original, diff_content)
            resolved.write_text(patched)
            return {
                "success": True,
                "path": file_path,
                "lines_changed": _count_changes(diff_content),
            }
        except ValueError as e:
            return {"error": f"Patch failed: {e}"}
        except (PermissionError, OSError) as e:
            return {"error": f"Cannot write file: {e}"}

    async def post_async(
        self, shared: dict[str, Any], prep_res: dict[str, Any], exec_res: dict[str, Any]
    ) -> str:
        tc = shared.get("current_tool_call", {})
        tool_name = tc.name if hasattr(tc, "name") else tc.get("name", "patch_file")
        content = exec_res.get("error") or str(exec_res)

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


def _apply_diff(original: str, diff: str) -> str:
    """Apply a simple unified diff to file content."""
    lines = original.splitlines(keepends=True)
    result: list[str] = []
    hunks = _parse_hunks(diff)
    line_idx = 0

    for hunk in hunks:
        # Copy lines before hunk
        while line_idx < hunk.src_start - 1 and line_idx < len(lines):
            result.append(lines[line_idx])
            line_idx += 1

        for op, text in hunk.changes:
            if op == " ":
                if line_idx < len(lines):
                    result.append(lines[line_idx])
                    line_idx += 1
            elif op == "-":
                line_idx += 1
            elif op == "+":
                result.append(text + "\n")

    # Copy remaining lines
    while line_idx < len(lines):
        result.append(lines[line_idx])
        line_idx += 1

    return "".join(result)


def _parse_hunks(diff: str) -> list:
    """Parse unified diff hunks."""
    from dataclasses import dataclass

    @dataclass
    class Hunk:
        src_start: int
        changes: list[tuple[str, str]]

    hunks: list[Hunk] = []
    current: Hunk | None = None

    for line in diff.splitlines():
        if line.startswith("@@"):
            import re

            m = re.match(r"@@ -(\d+)", line)
            if m:
                current = Hunk(src_start=int(m.group(1)), changes=[])
                hunks.append(current)
        elif current is not None:
            if line.startswith("+"):
                current.changes.append(("+", line[1:]))
            elif line.startswith("-"):
                current.changes.append(("-", line[1:]))
            elif line.startswith(" "):
                current.changes.append((" ", line[1:]))

    return hunks


def _count_changes(diff: str) -> int:
    return sum(
        1
        for line in diff.splitlines()
        if line.startswith(("+", "-")) and not line.startswith(("+++", "---"))
    )
