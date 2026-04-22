"""Load and manage .memory.md files for persistent context."""

from __future__ import annotations

from pathlib import Path

from ..constants import FilePath


def load_memory(work_dir: str) -> str:
    """Load .memory.md from work directory if it exists."""
    path = Path(work_dir) / FilePath.MEMORY_FILE
    if path.exists() and path.is_file():
        return path.read_text(errors="replace")
    return ""


def save_memory(work_dir: str, content: str) -> None:
    """Save content to .memory.md in work directory."""
    path = Path(work_dir) / FilePath.MEMORY_FILE
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(content)


def append_memory(work_dir: str, entry: str) -> None:
    """Append an entry to .memory.md."""
    existing = load_memory(work_dir)
    updated = f"{existing}\n\n{entry}".strip() if existing else entry
    save_memory(work_dir, updated)
