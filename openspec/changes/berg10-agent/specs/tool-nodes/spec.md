## ADDED Requirements

### Requirement: Security validation is extensible

The agent SHALL support configurable/extensible security validators for tool inputs.

#### Scenario: Pattern-based validation
- **WHEN** a regex pattern validator is registered for a tool parameter
- **THEN** the input is matched against the pattern before execution; rejection returns error

#### Scenario: Operation-based validation
- **WHEN** an operation-type validator is registered (e.g., path_traversal_check)
- **THEN** the input is checked against the operation rules before execution

#### Scenario: Schema-based validation
- **WHEN** a JSON schema validator is registered for a tool
- **THEN** the input params are validated against the schema before execution

#### Scenario: Custom validator can be added
- **WHEN** a new validator function is registered in the validator registry
- **THEN** it is applied to matching tool calls without modifying tool implementation

### Requirement: list_files tool lists directory contents

The `list_files` tool SHALL enumerate files in a directory.

#### Scenario: List files in directory
- **WHEN** `list_files` is invoked with `directory="src"`
- **THEN** all non-hidden files in `src` are returned as newline-separated paths

### Requirement: grep_search tool searches for patterns

The `grep_search` tool SHALL search for regex patterns in files.

#### Scenario: Grep finds matches
- **WHEN** `grep_search` is invoked with `pattern="def main"` and `path="src"`
- **THEN** matching lines are returned in `<file>:<line>:<content>` format

### Requirement: read_file tool reads file contents

The `read_file` tool SHALL read files with optional line range.

#### Scenario: Read full file
- **WHEN** `read_file` is invoked with `path="src/main.py"`
- **THEN** the entire file contents are returned with line numbers

#### Scenario: Read with line range
- **WHEN** `read_file` is invoked with `path="src/main.py"`, `start=1`, `end=10`
- **THEN** only lines 1-10 are returned with line numbers

#### Scenario: Read outside workdir is blocked
- **WHEN** `read_file` is invoked with `path="/etc/passwd"` outside configured workdir
- **THEN** the operation is rejected by the path_traversal validator with an appropriate error

### Requirement: patch_file tool modifies files

The `patch_file` tool SHALL replace text in files.

#### Scenario: Patch file successfully
- **WHEN** `patch_file` is invoked with `path="src/main.py"`, `old_str="foo"`, `new_str="bar"`
- **THEN** the first occurrence of `old_str` is replaced with `new_str`

#### Scenario: Old string not found
- **WHEN** `old_str` is not found in the file
- **THEN** an error message with closest match suggestion is returned

#### Scenario: Patch outside workdir is blocked
- **WHEN** `patch_file` is invoked with `path="/etc/passwd"` outside configured workdir
- **THEN** the operation is rejected by the path_traversal validator with an appropriate error

### Requirement: run_command tool executes shell commands

The `run_command` tool SHALL execute shell commands and return output.

#### Scenario: Execute command
- **WHEN** `run_command` is invoked with `cmd="ls -la"`
- **THEN** command output (stdout + stderr) is returned

#### Scenario: Command timeout
- **WHEN** command exceeds 30 second timeout
- **THEN** an error message is returned indicating timeout

#### Scenario: Dangerous command pattern is blocked
- **WHEN** `run_command` is invoked with `cmd="rm -rf /"` matching a dangerous_pattern validator
- **THEN** the command is rejected before execution with an appropriate error

#### Scenario: Command runs outside workdir
- **WHEN** `run_command` is invoked with `cmd="cd / && ls"` (command attempts to escape workdir)
- **THEN** the command is rejected by the path_traversal validator with an appropriate error