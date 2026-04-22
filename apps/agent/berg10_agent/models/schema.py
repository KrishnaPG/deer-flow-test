"""Model configuration schema and agent mode definitions."""

from __future__ import annotations

from dataclasses import dataclass, field
from enum import Enum
from typing import Any


class AgentMode(str, Enum):
    """Supported agent execution modes."""

    BUILD = "build"  # Default coding mode
    PLAN = "plan"  # Planning/analysis mode
    DEBUG = "debug"  # Debugging mode
    REVIEW = "review"  # Code review mode


@dataclass
class ModelConfig:
    """Configuration for an LLM provider/model."""

    id: str  # Unique identifier (e.g., "gemma-free")
    name: str  # Display name (e.g., "Gemma 3 12B (Free)")
    model: str  # Provider model string (e.g., "openrouter/google/gemma-3-12b-it")
    api_key: str = ""  # Provider-specific API key
    base_url: str = ""  # Optional custom endpoint
    provider: str = ""  # Provider name (e.g., "openrouter", "openai")
    is_free: bool = False  # Flag for free tier models
    metadata: dict[str, Any] = field(default_factory=dict)

    def to_dict(self) -> dict[str, Any]:
        """Convert to dictionary for API response."""
        return {
            "id": self.id,
            "name": self.name,
            "model": self.model,
            "api_key": self.api_key,
            "base_url": self.base_url,
            "provider": self.provider,
            "is_free": self.is_free,
            "metadata": self.metadata,
        }

    @classmethod
    def from_dict(cls, data: dict[str, Any]) -> ModelConfig:
        """Create from dictionary."""
        return cls(
            id=data["id"],
            name=data["name"],
            model=data["model"],
            api_key=data.get("api_key", ""),
            base_url=data.get("base_url", ""),
            provider=data.get("provider", ""),
            is_free=bool(data.get("is_free", False)),
            metadata=data.get("metadata", {}),
        )
