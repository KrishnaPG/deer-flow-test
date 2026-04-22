## ADDED Requirements

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

### Requirement: patch_file tool modifies files

The `patch_file` tool SHALL replace text in files.

#### Scenario: Patch file successfully
- **WHEN** `patch_file` is invoked with `path="src/main.py"`, `old_str="foo"`, `new_str="bar"`
- **THEN** the first occurrence of `old_str` is replaced with `new_str`

#### Scenario: Old string not found
- **WHEN** `old_str` is not found in the file
- **THEN** an error message with closest match suggestion is returned

### Requirement: run_command tool executes shell commands

The `run_command` tool SHALL execute shell commands and return output.

#### Scenario: Execute command
- **WHEN** `run_command` is invoked with `cmd="ls -la"`
- **THEN** command output (stdout + stderr) is returned

#### Scenario: Command timeout
- **WHEN** command exceeds 30 second timeout
- **THEN** an error message is returned indicating timeout