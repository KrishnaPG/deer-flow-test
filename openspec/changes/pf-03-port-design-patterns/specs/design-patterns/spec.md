## ADDED Requirements

### Requirement: Port Agent pattern to Rust with Dapr Actors
The system SHALL port the PocketFlow Agent pattern to Rust using Dapr Actors for stateful, single-threaded execution and Dapr Activities for tool execution.

#### Scenario: Agent decision logic
- **WHEN** implementing agent decision logic
- **THEN** the system SHALL use Dapr Actors to maintain agent state
- **AND** the system SHALL support dynamic action selection based on context
- **AND** the system SHALL preserve the agent's ability to call tools and LLMs

#### Scenario: Agent state persistence
- **WHEN** persisting agent state
- **THEN** the system SHALL use Dapr Actor state management
- **AND** the system SHALL support agent recovery after failures
- **AND** the system SHALL maintain conversation history and context

#### Scenario: Agent tool execution
- **WHEN** executing agent tools
- **THEN** the system SHALL map tools to Dapr Activities
- **AND** the system SHALL support both synchronous and asynchronous tool calls
- **AND** the system SHALL handle tool failures with retry and fallback

#### Scenario: Agent action-based routing
- **WHEN** an agent node completes its execution
- **THEN** the agent SHALL return an `Action` enum value (e.g., `Action::Custom("search".to_string())` or `Action::Custom("answer".to_string())`)
- **AND** the returned action SHALL be used by the Flow's immutable routing table to lookup the next node
- **AND** the agent's decision logic SHALL be implemented in the node's `post()` method
- **AND** the agent SHALL store any intermediate results in SharedStore for use by downstream nodes

#### Scenario: Agent decision node pattern
- **WHEN** implementing a DecideAction node (agent's decision point)
- **THEN** the node SHALL have a `post()` method that calls LLM to decide next action
- **AND** the node SHALL return `Action::Custom("search".to_string())` if LLM decides search is needed
- **AND** the node SHALL return `Action::Custom("answer".to_string())` if LLM decides to answer directly
- **AND** the node SHALL store decision metadata in SharedStore for observability

#### Scenario: Agent loop for continued research
- **WHEN** an agent needs to gather more information iteratively
- **THEN** the graph SHALL define a route back to the decision node via the Builder: `.route("search", Action::Custom("decide".to_string()), "decide")`
- **AND** this loop SHALL be defined at graph construction time via the Builder
- **AND** the loop SHALL continue until the agent returns an action not wired to the loop

### Requirement: Port Map-Reduce pattern to Rust with Dapr Workflows
The system SHALL port the PocketFlow Map-Reduce pattern to Rust using Dapr Workflow fan-out/fan-in.

#### Scenario: Map phase
- **WHEN** executing map phase
- **THEN** the system SHALL use Dapr Workflow parallel execution for fan-out
- **AND** the system SHALL process items concurrently with configurable parallelism
- **AND** the system SHALL preserve item-by-item processing semantics

#### Scenario: Reduce phase
- **WHEN** executing reduce phase
- **THEN** the system SHALL aggregate results from map phase
- **AND** the system SHALL support custom reduction logic
- **AND** the system SHALL handle partial failures in map phase

#### Scenario: Idempotent map processing
- **WHEN** processing map items
- **THEN** the system SHALL ensure each item is processed idempotently
- **AND** the system SHALL use deterministic keys for retries
- **AND** the system SHALL support exactly-once processing semantics

#### Scenario: BatchNode map semantics
- **WHEN** implementing map phase with BatchNode
- **THEN** prep() SHALL return an iterable (list of items)
- **AND** exec() SHALL be called once per item, results collected into list
- **AND** post() SHALL receive the full list of results for aggregation

#### Scenario: Linear map-reduce flow
- **WHEN** building a simple map-reduce flow
- **THEN** the system SHALL use Builder pattern chaining: `Builder::new().add(read).add(map).add(reduce)`
- **AND** the read node produces data stored in shared state
- **AND** the map node (BatchNode) processes items from shared state
- **AND** the reduce node aggregates results from shared state

### Requirement: Port RAG pattern to Rust with Dapr Vector bindings
The system SHALL port the PocketFlow RAG pattern to Rust using Dapr Vector bindings and LLM Activities.

#### Scenario: Document retrieval
- **WHEN** retrieving documents for RAG
- **THEN** the system SHALL use Dapr Vector bindings for similarity search
- **AND** the system SHALL support multiple vector stores (Pinecone, Weaviate, etc.)
- **AND** the system SHALL preserve chunking and embedding logic

#### Scenario: LLM integration
- **WHEN** generating responses with LLM
- **THEN** the system SHALL use Dapr Activities for LLM calls
- **AND** the system SHALL support prompt caching and context management
- **AND** the system SHALL handle streaming responses where applicable

#### Scenario: Iterative RAG
- **WHEN** implementing iterative RAG (retrieve→generate→refine)
- **THEN** the system SHALL use Dapr Workflow loops
- **AND** the system SHALL support configurable iteration limits
- **AND** the system SHALL track iteration state in workflow

#### Scenario: Offline and online flow separation
- **WHEN** building a RAG system
- **THEN** the system SHALL support separate offline flow (indexing) and online flow (retrieval)
- **AND** offline flow SHALL handle: chunk documents → embed → create index → store
- **AND** online flow SHALL handle: embed query → retrieve documents → generate answer
- **AND** the two flows SHALL share state via SharedStore (texts, index, etc.)
- **AND** flows SHALL be independent and separately executable

#### Scenario: Sequential RAG nodes
- **WHEN** executing a RAG flow
- **THEN** each node SHALL be chained via a Builder pattern
- **AND** data SHALL be passed between nodes via SharedStore
- **AND** each node SHALL store its output in SharedStore for next node to access

### Requirement: Port Multi-Agent pattern to Rust with Dapr Pub/Sub
The system SHALL port the PocketFlow Multi-Agent pattern to Rust using Dapr Actors for agents and Dapr Pub/Sub for communication.

#### Scenario: Agent communication
- **WHEN** agents communicate with each other
- **THEN** the system SHALL use Dapr Pub/Sub for message passing
- **AND** the system SHALL support both synchronous and asynchronous communication
- **AND** the system SHALL preserve message ordering and delivery guarantees

#### Scenario: Agent coordination
- **WHEN** coordinating multiple agents
- **THEN** the system SHALL support supervisor patterns for coordination
- **AND** the system SHALL handle agent failures and restarts
- **AND** the system SHALL provide observability into agent interactions

#### Scenario: Shared state
- **WHEN** agents need shared state
- **THEN** the system SHALL use Dapr State Management
- **AND** the system SHALL support transactional updates across agents
- **AND** the system SHALL handle concurrent access with appropriate locking

#### Scenario: Concurrent agent flows via shared queues
- **WHEN** running multiple agent flows concurrently
- **THEN** agents SHALL communicate via async message queues
- **AND** each agent flow SHALL run independently with its own start node
- **AND** agents SHALL loop by wiring themselves via Builder routing
- **AND** the system SHALL use `tokio::spawn` to run all agent flows concurrently
- **AND** agents SHALL terminate when they return a terminal action (e.g., `Action::Done`)

#### Scenario: Inter-agent signaling
- **WHEN** one agent needs to signal another
- **THEN** the system SHALL allow agents to put messages into shared queues
- **AND** agents SHALL block (await) on queue.get() until message arrives
- **AND** special messages (e.g., "GAME_OVER") SHALL signal termination

### Requirement: Port Supervisor pattern to Rust with Dapr Workflows
The system SHALL port the PocketFlow Supervisor pattern to Rust using Dapr Workflow loops and evaluation Activities.

#### Scenario: Quality validation
- **WHEN** supervising agent outputs
- **THEN** the system SHALL validate outputs against quality criteria
- **AND** the system SHALL support retry with feedback loops
- **AND** the system SHALL provide escalation mechanisms for failures

#### Scenario: Supervisor loop
- **WHEN** implementing supervisor loop
- **THEN** the system SHALL use Dapr Workflow conditional looping
- **AND** the system SHALL support configurable quality thresholds
- **AND** the system SHALL track iteration count and timeout

#### Scenario: Feedback integration
- **WHEN** providing feedback to agents
- **THEN** the system SHALL store feedback in Dapr State
- **AND** the system SHALL allow agents to retrieve feedback
- **AND** the system SHALL support feedback-based learning

#### Scenario: Supervisor back-edge for retry loops
- **WHEN** supervisor rejects the agent's output
- **THEN** the supervisor SHALL return `Action::Retry`
- **AND** the graph SHALL have a Builder-defined route back to the agent flow
- **AND** this back-edge SHALL create the retry loop at construction time
- **AND** when supervisor approves, it SHALL return `Action::Done`, terminating the loop

#### Scenario: Supervisor escalation
- **WHEN** supervisor determines output cannot be fixed after max iterations
- **THEN** the supervisor SHALL return an escalation action (e.g., `Action::Escalate`)
- **AND** the graph SHALL have an edge to an escalation handler node

### Requirement: Port Heartbeat/Monitor pattern to Rust
The system SHALL port the PocketFlow Heartbeat Monitor pattern where an outer Flow loops indefinitely over an inner Flow with polling.

#### Scenario: Outer polling loop
- **WHEN** implementing a heartbeat monitor
- **THEN** the system SHALL have an outer Flow with a wait/poll node
- **AND** the outer loop SHALL continue until max cycles or terminal signal
- **AND** the wait node SHALL return "continue" or "done" to control looping

#### Scenario: Inner flow execution per cycle
- **WHEN** the outer loop executes
- **THEN** each cycle SHALL run the inner Flow once
- **AND** after inner flow completes, control SHALL return to outer wait node
- **AND** cycle count SHALL be tracked in shared state

#### Scenario: Polling interval
- **WHEN** the wait node executes
- **THEN** it SHALL sleep/wait for configured poll_interval
- **AND** it SHALL increment cycle_count
- **AND** it SHALL check if max_cycles reached to decide continue/done

### Requirement: Port Self-Healing pattern to Rust
The system SHALL port the PocketFlow Self-Healing pattern where errors trigger retry loops with feedback.

#### Scenario: Error feedback loop
- **WHEN** a node fails (e.g., compilation error)
- **THEN** the node SHALL return an error action (e.g., `Action::Custom("fix".to_string())`)
- **AND** the error context SHALL be stored in shared state
- **AND** a subsequent fix node SHALL read the error and attempt correction
- **AND** the loop SHALL continue until success or max attempts

#### Scenario: Self-healing with max attempts
- **WHEN** implementing self-healing with max attempts
- **THEN** each failed attempt SHALL be recorded in shared state
- **AND** after max attempts, the node SHALL return terminal action
- **AND** the node SHALL store both code and error for later analysis

### Requirement: Port Guardrails pattern to Rust
The system SHALL port the PocketFlow Guardrails pattern where input validation failures loop back for correction.

#### Scenario: Input validation with retry
- **WHEN** a guardrail node validates input
- **THEN** if validation fails, the node SHALL return `Action::Retry`
- **AND** the validation message SHALL be stored in shared state
- **AND** control SHALL return to the input source for correction
- **AND** this loop SHALL continue until validation passes

#### Scenario: Guardrail message to user
- **WHEN** guardrail rejects input
- **THEN** the system SHALL present the rejection reason to the user
- **AND** the user SHALL have opportunity to provide corrected input
- **AND** validation SHALL run again on corrected input

### Requirement: Port Majority Vote pattern to Rust
The system SHALL port the PocketFlow Majority Vote pattern where a BatchNode runs ensemble voting and aggregates results.

#### Scenario: Ensemble batch processing
- **WHEN** implementing majority vote
- **THEN** prep() SHALL return a list of identical questions (num_tries)
- **AND** exec() SHALL be called once per question (LLM generates independent answer)
- **AND** exec_fallback() SHALL gracefully handle failed attempts (return None)

#### Scenario: Vote aggregation
- **WHEN** post() receives list of answers
- **THEN** the system SHALL filter out None (failed) results
- **AND** the system SHALL count occurrences of each answer
- **AND** the most common answer SHALL be selected as final result
- **AND** the result SHALL be stored in shared state

### Requirement: Port Judge pattern to Rust
The system SHALL port the PocketFlow Judge pattern where a judge node evaluates agent output with pass/fail actions.

#### Scenario: Judge evaluation
- **WHEN** a judge node evaluates output
- **THEN** the judge SHALL return {valid: true/false, reason: "..."}
- **AND** valid=true SHALL cause judge to return terminal action (`Action::Done`)
- **AND** valid=false SHALL cause judge to return `Action::Retry` action

#### Scenario: Judge with max retries
- **WHEN** implementing judge with limited retries
- **THEN** the inner agent flow SHALL be restarted on each retry
- **AND** the judge SHALL track attempt count
- **AND** after max retries, judge SHALL return escalation action

### Requirement: Port Debate pattern to Rust
The system SHALL port the PocketFlow Debate pattern where sequential advocate nodes present opposing arguments to a judge.

#### Scenario: Sequential advocates
- **WHEN** building a debate flow
- **THEN** the system SHALL wire advocates sequentially via Builder
- **AND** each advocate SHALL write its argument to shared state
- **AND** the judge SHALL evaluate both arguments

#### Scenario: Judge evaluation of debate
- **WHEN** the judge node executes
- **THEN** the judge SHALL read arguments from shared state
- **AND** the judge SHALL render a verdict
- **AND** the verdict SHALL be stored in shared state

### Requirement: Port A2A Protocol pattern to Rust
The system SHALL port the PocketFlow A2A pattern where a PocketFlow is wrapped as an A2A server/task manager.

#### Scenario: PocketFlow as A2A server
- **WHEN** wrapping PocketFlow as A2A server
- **THEN** the system SHALL implement InMemoryTaskManager interface
- **AND** on_send_task SHALL extract query from A2A request
- **AND** the system SHALL run the PocketFlow with extracted query
- **AND** the system SHALL package result as A2A artifact

### Requirement: Pattern composition
The system SHALL support composition of design patterns (e.g., Agent inside Map-Reduce, Supervisor coordinating Multi-Agent).

#### Scenario: Pattern nesting
- **WHEN** composing patterns
- **THEN** the system SHALL allow patterns to be nested within other patterns
- **AND** the system SHALL preserve pattern isolation and state boundaries
- **AND** the system SHALL support cross-pattern communication via Dapr building blocks

#### Scenario: Configuration inheritance
- **WHEN** configuring composed patterns
- **THEN** the system SHALL support configuration inheritance from parent to child patterns
- **AND** the system SHALL allow pattern-specific overrides
- **AND** the system SHALL validate configuration consistency

### Requirement: Maintain pattern compatibility with Python PocketFlow
The system SHALL ensure Rust implementations provide functional equivalence with Python PocketFlow design patterns, improving upon Python where beneficial.

#### Scenario: Pattern behavior
- **WHEN** executing the same pattern in Python and Rust
- **THEN** the system SHALL produce functionally equivalent results for identical inputs
- **AND** the system SHALL handle edge cases and error conditions equivalently
- **AND** the system SHALL preserve pattern-specific configuration options
- **AND** the system MAY produce improved behavior that achieves same outcome faster

#### Scenario: API compatibility
- **WHEN** using the Rust pattern API
- **THEN** the system SHALL provide methods matching Python PocketFlow's pattern API
- **AND** the system SHALL maintain similar method signatures where possible
- **AND** the system SHALL document differences due to Rust's type system
- **AND** the system SHALL prioritize working implementation over line-by-line copying