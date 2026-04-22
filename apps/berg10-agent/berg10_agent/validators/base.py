"""Base validator interface."""

from __future__ import annotations

from abc import ABC, abstractmethod
from dataclasses import dataclass


@dataclass(frozen=True)
class ValidatorResult:
    """Result of a validation check."""

    passed: bool
    reason: str = ""

    def __bool__(self) -> bool:
        return self.passed


class BaseValidator(ABC):
    """Abstract base for tool input validators."""

    @abstractmethod
    def validate(self, tool_name: str, params: dict) -> ValidatorResult:
        """Validate tool parameters. Return result with pass/fail and reason."""
        ...
