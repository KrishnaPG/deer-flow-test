"""Load validator configuration from JSON/YAML config files."""

from __future__ import annotations

import json
from pathlib import Path
from typing import Any

from .base import BaseValidator
from .operation import DangerousCommandValidator, PathTraversalValidator
from .pattern import PatternRule, PatternValidator
from .schema import SchemaValidator


def load_validators_from_config(config_path: str | Path) -> list[BaseValidator]:
    """Load validators from a JSON config file.

    Expected format:
    {
      "pattern_rules": [
        {"pattern": "<regex>", "field_name": "path", "description": "..."}
      ],
      "path_traversal": {"work_dir": ".", "field_names": ["path", "file"]},
      "dangerous_commands": {"field_name": "command", "extra_patterns": [...]},
      "schemas": {
        "read_file": {"type": "object", "properties": {"path": {"type": "string"}}}
      }
    }
    """
    path = Path(config_path)
    if not path.exists():
        return []

    with open(path) as f:
        config: dict[str, Any] = json.load(f)

    validators: list[BaseValidator] = []

    # Pattern rules
    pattern_rules_data = config.get("pattern_rules", [])
    if pattern_rules_data:
        pv = PatternValidator()
        for rule in pattern_rules_data:
            pv.add_rule(
                PatternRule(
                    pattern=rule["pattern"],
                    field_name=rule.get("field_name", "path"),
                    flags=rule.get("flags", 0),
                    description=rule.get("description", ""),
                )
            )
        validators.append(pv)

    # Path traversal
    pt_config = config.get("path_traversal")
    if pt_config is not None:
        validators.append(
            PathTraversalValidator(
                work_dir=pt_config.get("work_dir", "."),
                field_names=pt_config.get("field_names", ("path", "file", "file_path", "target")),
            )
        )

    # Dangerous commands
    dc_config = config.get("dangerous_commands")
    if dc_config is not None:
        validators.append(
            DangerousCommandValidator(
                extra_patterns=dc_config.get("extra_patterns"),
                field_name=dc_config.get("field_name", "command"),
            )
        )

    # Schema validators
    schemas = config.get("schemas", {})
    if schemas:
        sv = SchemaValidator()
        for tool_name, schema in schemas.items():
            sv.register_schema(tool_name, schema)
        validators.append(sv)

    return validators
