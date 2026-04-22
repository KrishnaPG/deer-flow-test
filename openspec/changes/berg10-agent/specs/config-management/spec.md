## ADDED Requirements

### Requirement: Configuration from environment file

The agent SHALL load configuration from a `.env` file using python-dotenv.

#### Scenario: Env file loaded
- **WHEN** `.env` file exists with `BERG10_MODEL="..."`
- **THEN** the model is set from the env file

### Requirement: CLI overrides environment

The CLI SHALL allow command-line flags to override environment settings.

#### Scenario: CLI overrides env
- **WHEN** `--model "custom/model"` is passed alongside an env file
- **THEN** the CLI value takes precedence

### Requirement: Environment variable overrides env file

An environment variable SHALL override values from the `.env` file.

#### Scenario: Env var takes precedence
- **WHEN** `BERG10_MODEL` is set in the shell environment
- **THEN** the shell environment value takes precedence over `.env` file

### Requirement: Precedence order is deterministic

The configuration precedence SHALL be: CLI > env var > `.env` file > defaults.

#### Scenario: Precedence order
- **WHEN** all three config sources set different values
- **THEN** CLI > env var > `.env` file > default model