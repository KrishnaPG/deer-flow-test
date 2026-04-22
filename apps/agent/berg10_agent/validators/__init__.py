"""Validator registry for tool input security."""

from __future__ import annotations

from .base import BaseValidator, ValidatorResult
from .config_loader import load_validators_from_config
from .operation import PathTraversalValidator
from .pattern import PatternValidator
from .schema import SchemaValidator

__all__ = [
    "BaseValidator",
    "ValidatorResult",
    "PatternValidator",
    "PathTraversalValidator",
    "SchemaValidator",
    "ValidatorRegistry",
    "load_validators_from_config",
]


class ValidatorRegistry:
    """Extensible registry for tool input validators."""

    def __init__(self) -> None:
        self._validators: dict[str, BaseValidator] = {}

    def register(self, name: str, validator: BaseValidator) -> None:
        self._validators[name] = validator

    def unregister(self, name: str) -> None:
        self._validators.pop(name, None)

    def get(self, name: str) -> BaseValidator | None:
        return self._validators.get(name)

    def list_names(self) -> list[str]:
        return list(self._validators.keys())

    def validate_all(self, tool_name: str, params: dict) -> ValidatorResult:
        """Run all registered validators against tool params."""
        for validator in self._validators.values():
            result = validator.validate(tool_name, params)
            if not result.passed:
                return result
        return ValidatorResult(passed=True)
