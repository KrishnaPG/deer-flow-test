## ADDED Requirements

### Requirement: Validator registry exists

The agent SHALL maintain a registry of pluggable validators for tool inputs.

#### Scenario: Registry is accessible
- **WHEN** a tool is invoked
- **THEN** the registered validators are applied to the input parameters before execution

### Requirement: Pattern-based validator

The registry SHALL support regex pattern matching validators.

#### Scenario: Pattern matches - blocked
- **WHEN** a pattern validator is configured with pattern `rm -rf .*` for `run_command.cmd`
- **THEN** input matching the regex is rejected with an appropriate error message

#### Scenario: Pattern does not match - allowed
- **WHEN** a pattern validator is configured but input does not match
- **THEN** execution proceeds normally

### Requirement: Operation-based validator

The registry SHALL support operation-type validators (e.g., path traversal checks).

#### Scenario: Path traversal detected
- **WHEN** a file operation targets a path outside the configured workdir
- **THEN** the operation is rejected with an appropriate error message

#### Scenario: Path is within workdir - allowed
- **WHEN** a file operation targets a path within the configured workdir
- **THEN** execution proceeds normally

### Requirement: Schema-based validator

The registry SHALL support JSON Schema validation for tool parameters.

#### Scenario: Schema validation passes
- **WHEN** a schema validator is configured and input matches the schema
- **THEN** execution proceeds normally

#### Scenario: Schema validation fails
- **WHEN** a schema validator is configured and input does not match the schema
- **THEN** the operation is rejected with schema validation error details

### Requirement: Custom validators can be registered

The registry SHALL support adding custom validator functions at runtime.

#### Scenario: Custom validator is added
- **WHEN** a new validator function is registered with a name and applicable tools
- **THEN** the validator is applied to all matching tool calls without modifying the tool implementation

### Requirement: Validators are configurable

The validators SHALL be configurable via configuration files or environment variables.

#### Scenario: Validators loaded from config
- **WHEN** a configuration file specifies validator rules
- **THEN** those rules are applied to the corresponding tool calls