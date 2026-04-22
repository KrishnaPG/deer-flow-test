"""Schema-based validators using JSON Schema."""

from __future__ import annotations

from typing import Any

from jsonschema import Draft202012Validator, ValidationError

from .base import BaseValidator, ValidatorResult


class SchemaValidator(BaseValidator):
    """JSON Schema validator for tool input parameters."""

    def __init__(self) -> None:
        self._schemas: dict[str, dict[str, Any]] = {}

    def register_schema(self, tool_name: str, schema: dict[str, Any]) -> None:
        self._schemas[tool_name] = schema

    def validate(self, tool_name: str, params: dict) -> ValidatorResult:
        schema = self._schemas.get(tool_name)
        if schema is None:
            return ValidatorResult(passed=True)

        validator = Draft202012Validator(schema)
        errors = list(validator.iter_errors(params))
        if errors:
            msgs = [f"  - {e.message}" for e in errors[:5]]
            return ValidatorResult(
                passed=False,
                reason=f"Schema validation failed for '{tool_name}':\n" + "\n".join(msgs),
            )
        return ValidatorResult(passed=True)
