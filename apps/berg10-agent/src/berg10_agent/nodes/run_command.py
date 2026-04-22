"""Execute shell commands with configurable timeout."""

from __future__ import annotations

import asyncio
from typing import Any

from pocketflow import AsyncNode


class RunCommandNode(AsyncNode):
    """Execute shell commands with timeout and output capture."""

    def __init__(self, work_dir: str = ".", default_timeout: int = 60) -> None:
        super().__init__()
        self._work_dir = work_dir
        self._default_timeout = default_timeout

    async def prep(self, shared: dict[str, Any]) -> dict[str, Any]:
        tc = shared.get("current_tool_call", {})
        args = tc.arguments if hasattr(tc, "arguments") else tc.get("arguments", {})
        return {
            "command": args.get("command", ""),
            "timeout": args.get("timeout", self._default_timeout),
        }

    async def exec(self, prep_res: dict[str, Any]) -> dict[str, Any]:
        command = prep_res["command"]
        if not command:
            return {"error": "Command is required"}

        timeout = min(prep_res["timeout"], self._default_timeout)

        try:
            proc = await asyncio.create_subprocess_shell(
                command,
                stdout=asyncio.subprocess.PIPE,
                stderr=asyncio.subprocess.PIPE,
                cwd=self._work_dir,
            )
            stdout, stderr = await asyncio.wait_for(proc.communicate(), timeout=timeout)

            return {
                "stdout": stdout.decode(errors="replace"),
                "stderr": stderr.decode(errors="replace"),
                "exit_code": proc.returncode,
                "success": proc.returncode == 0,
            }
        except asyncio.TimeoutError:
            proc.kill()
            return {"error": f"Command timed out after {timeout}s", "exit_code": -1}
        except Exception as e:
            return {"error": f"Command failed: {e}", "exit_code": -1}

    async def post(
        self, shared: dict[str, Any], prep_res: dict[str, Any], exec_res: dict[str, Any]
    ) -> str:
        tc = shared.get("current_tool_call", {})
        tool_name = tc.name if hasattr(tc, "name") else tc.get("name", "run_command")

        if exec_res.get("error"):
            content = exec_res["error"]
        else:
            parts = []
            if exec_res.get("stdout"):
                parts.append(f"stdout:\n{exec_res['stdout']}")
            if exec_res.get("stderr"):
                parts.append(f"stderr:\n{exec_res['stderr']}")
            parts.append(f"exit_code: {exec_res.get('exit_code', 'unknown')}")
            content = "\n".join(parts)

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
