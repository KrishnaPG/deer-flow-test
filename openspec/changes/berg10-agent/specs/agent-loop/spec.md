## ADDED Requirements

### Requirement: Agent flow executes in a decision loop

The agent SHALL repeatedly decide on actions and execute them until completion.

#### Scenario: Single action completes task
- **WHEN** user provides a task that completes in one action
- **THEN** the agent executes the action and returns the result

#### Scenario: Multiple actions required
- **WHEN** user provides a task requiring multiple steps
- **THEN** the agent loops through actions until the task is complete or max steps reached

### Requirement: Agent tracks conversation history

The agent SHALL maintain conversation history across turns.

#### Scenario: History accumulates
- **WHEN** multiple messages are sent in a session
- **THEN** conversation history grows with each exchange

### Requirement: Agent compacts history after threshold

The agent SHALL summarize old history once it exceeds the threshold.

#### Scenario: History compaction
- **WHEN** history exceeds 30 turns
- **THEN** the oldest turns are summarized and replaced with a summary

### Requirement: Agent loads project skills

The agent SHALL load project-specific rules from `AGENTS.md`.

#### Scenario: Skills loaded
- **WHEN** `AGENTS.md` exists in the working directory
- **THEN** its contents are injected into the prompt for each decision

### Requirement: Agent loads session memory

The agent SHALL load cross-session memory from `.memory.md`.

#### Scenario: Memory loaded
- **WHEN** `.memory.md` exists in the working directory
- **THEN** its contents are injected into the prompt as context