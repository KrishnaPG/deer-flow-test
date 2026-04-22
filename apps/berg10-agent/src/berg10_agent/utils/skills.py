"""Load AGENTS.md skills files for agent capabilities."""

from __future__ import annotations

from pathlib import Path

from ..constants import FilePath


def load_skills(work_dir: str) -> str:
    """Load AGENTS.md from work directory if it exists."""
    path = Path(work_dir) / FilePath.SKILLS_FILE
    if path.exists() and path.is_file():
        return path.read_text(errors="replace")
    return ""


def discover_skills(work_dir: str) -> list[dict[str, str]]:
    """Discover all .md skill files in work directory."""
    base = Path(work_dir)
    skills: list[dict[str, str]] = []

    # Check main AGENTS.md
    main_path = base / FilePath.SKILLS_FILE
    if main_path.exists():
        skills.append(
            {
                "name": "main",
                "path": str(main_path),
                "content": main_path.read_text(errors="replace"),
            }
        )

    # Check for .agents/ directory
    agents_dir = base / ".agents"
    if agents_dir.is_dir():
        for md_file in sorted(agents_dir.glob("*.md")):
            skills.append(
                {
                    "name": md_file.stem,
                    "path": str(md_file),
                    "content": md_file.read_text(errors="replace"),
                }
            )

    return skills
