"""Pattern-based validators using regex matching."""

from __future__ import annotations

import re
from dataclasses import dataclass, field

from .base import BaseValidator, ValidatorResult


@dataclass
class PatternRule:
    """A single pattern validation rule."""

    pattern: str
    field_name: str
    flags: int = 0
    description: str = ""
    _compiled: re.Pattern | None = field(default=None, repr=False)

    def __post_init__(self) -> None:
        self._compiled = re.compile(self.pattern, self.flags)


class PatternValidator(BaseValidator):
    """Regex-based validator for tool input fields."""

    def __init__(self, rules: list[PatternRule] | None = None) -> None:
        self._rules: list[PatternRule] = list(rules) if rules else []

    def add_rule(self, rule: PatternRule) -> None:
        self._rules.append(rule)

    def validate(self, tool_name: str, params: dict) -> ValidatorResult:
        for rule in self._rules:
            value = params.get(rule.field_name, "")
            if not isinstance(value, str):
                value = str(value)
            if rule._compiled and rule._compiled.search(value):
                return ValidatorResult(
                    passed=False,
                    reason=f"Pattern match blocked: {rule.description or rule.pattern} "
                    f"on field '{rule.field_name}' for tool '{tool_name}'",
                )
        return ValidatorResult(passed=True)
