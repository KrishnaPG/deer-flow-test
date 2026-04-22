"""Operation-based validators (path traversal, dangerous commands)."""

from __future__ import annotations

from pathlib import Path
from typing import Sequence

from .base import BaseValidator, ValidatorResult


DANGEROUS_PATTERNS: tuple[str, ...] = (
    "../",
    "..\\",
    "~",
)

DANGEROUS_COMMANDS: tuple[str, ...] = (
    "rm -rf /",
    "rm -rf *",
    "mkfs",
    "dd if=",
    ":(){:|:&};:",
    "chmod 777",
    "curl | sh",
    "wget | sh",
)


class PathTraversalValidator(BaseValidator):
    """Blocks path traversal attempts in tool inputs."""

    def __init__(
        self,
        work_dir: str = ".",
        field_names: Sequence[str] = ("path", "file", "file_path", "target"),
    ) -> None:
        self._work_dir = Path(work_dir).resolve()
        self._field_names = set(field_names)

    def validate(self, tool_name: str, params: dict) -> ValidatorResult:
        for field_name in self._field_names:
            raw = params.get(field_name, "")
            if not raw or not isinstance(raw, str):
                continue

            for pattern in DANGEROUS_PATTERNS:
                if pattern in raw:
                    return ValidatorResult(
                        passed=False,
                        reason=f"Path traversal blocked: '{raw}' contains '{pattern}' "
                        f"on field '{field_name}' for tool '{tool_name}'",
                    )

            resolved = (self._work_dir / raw).resolve()
            if not str(resolved).startswith(str(self._work_dir)):
                return ValidatorResult(
                    passed=False,
                    reason=f"Path escapes work directory: '{raw}' resolves to '{resolved}' "
                    f"(work_dir: {self._work_dir})",
                )

        return ValidatorResult(passed=True)


class DangerousCommandValidator(BaseValidator):
    """Blocks known dangerous shell commands."""

    def __init__(
        self,
        extra_patterns: Sequence[str] | None = None,
        field_name: str = "command",
    ) -> None:
        self._blocked = list(DANGEROUS_COMMANDS)
        if extra_patterns:
            self._blocked.extend(extra_patterns)
        self._field_name = field_name

    def validate(self, tool_name: str, params: dict) -> ValidatorResult:
        command = str(params.get(self._field_name, ""))
        for blocked in self._blocked:
            if blocked in command:
                return ValidatorResult(
                    passed=False,
                    reason=f"Dangerous command blocked: contains '{blocked}'",
                )
        return ValidatorResult(passed=True)
