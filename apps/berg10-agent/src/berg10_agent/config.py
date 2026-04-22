"""Configuration management for berg10-agent."""

from __future__ import annotations

import os
from dataclasses import dataclass
from pathlib import Path
from typing import Optional

from .constants import ConfigKey, Defaults


@dataclass
class AgentConfig:
    """Agent configuration loaded from env/CLI with defaults."""

    model: str = Defaults.MODEL
    api_key: str = ""
    base_url: str = ""
    host: str = Defaults.HOST
    port: int = Defaults.PORT
    log_level: str = Defaults.LOG_LEVEL
    work_dir: str = Defaults.WORK_DIR
    max_turns: int = Defaults.MAX_TURNS
    max_history_tokens: int = Defaults.MAX_HISTORY_TOKENS
    compact_threshold: int = Defaults.COMPACT_THRESHOLD
    tool_timeout: int = Defaults.TOOL_TIMEOUT
    enable_validators: bool = Defaults.ENABLE_VALIDATORS
    enable_memory: bool = Defaults.ENABLE_MEMORY
    enable_skills: bool = Defaults.ENABLE_SKILLS

    @classmethod
    def from_env(cls, **overrides: object) -> AgentConfig:
        """Build config from environment variables with CLI overrides."""
        env_vals = {
            "model": os.environ.get(ConfigKey.MODEL.value),
            "api_key": os.environ.get(ConfigKey.API_KEY.value, ""),
            "base_url": os.environ.get(ConfigKey.BASE_URL.value, ""),
            "host": os.environ.get(ConfigKey.HOST.value),
            "port": _int_env(ConfigKey.PORT.value),
            "log_level": os.environ.get(ConfigKey.LOG_LEVEL.value),
            "work_dir": os.environ.get(ConfigKey.WORK_DIR.value),
            "max_turns": _int_env(ConfigKey.MAX_TURNS.value),
            "max_history_tokens": _int_env(ConfigKey.MAX_HISTORY_TOKENS.value),
            "compact_threshold": _int_env(ConfigKey.COMPACT_THRESHOLD.value),
            "tool_timeout": _int_env(ConfigKey.TOOL_TIMEOUT.value),
            "enable_validators": _bool_env(ConfigKey.ENABLE_VALIDATORS.value),
            "enable_memory": _bool_env(ConfigKey.ENABLE_MEMORY.value),
            "enable_skills": _bool_env(ConfigKey.ENABLE_SKILLS.value),
        }

        # CLI overrides take precedence
        for key, val in overrides.items():
            if val is not None:
                env_vals[key] = val

        # Apply defaults for None values
        defaults = {
            "model": Defaults.MODEL,
            "host": Defaults.HOST,
            "port": Defaults.PORT,
            "log_level": Defaults.LOG_LEVEL,
            "work_dir": Defaults.WORK_DIR,
            "max_turns": Defaults.MAX_TURNS,
            "max_history_tokens": Defaults.MAX_HISTORY_TOKENS,
            "compact_threshold": Defaults.COMPACT_THRESHOLD,
            "tool_timeout": Defaults.TOOL_TIMEOUT,
            "enable_validators": Defaults.ENABLE_VALIDATORS,
            "enable_memory": Defaults.ENABLE_MEMORY,
            "enable_skills": Defaults.ENABLE_SKILLS,
            "api_key": "",
            "base_url": "",
        }

        merged = {k: env_vals[k] if env_vals[k] is not None else defaults[k] for k in defaults}
        return cls(**merged)

    @property
    def resolved_work_dir(self) -> Path:
        """Return absolute path to work directory."""
        return Path(self.work_dir).resolve()


def _int_env(key: str) -> Optional[int]:
    val = os.environ.get(key)
    if val is not None:
        try:
            return int(val)
        except ValueError:
            return None
    return None


def _bool_env(key: str) -> Optional[bool]:
    val = os.environ.get(key)
    if val is not None:
        return val.lower() in ("true", "1", "yes")
    return None
