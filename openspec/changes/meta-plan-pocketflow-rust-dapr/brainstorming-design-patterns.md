# Brainstorming: Design Patterns Dapr Mapping

## Current State (PocketFlow-Rust)
- **No design patterns implemented** (Agent, Map-Reduce, RAG, Multi-Agent, Supervisor missing)
- Only basic Flow and Node abstractions exist
- RAG example exists but minimal

## Dapr Building Blocks for Patterns

### 1. Agent Pattern (Dynamic Action Selection)
**Python**: Agent node selects actions based on LLM reasoning, can call tools, loop until task complete.
**Dapr Mapping**:
- **Agent as Dapr Actor**: Maintains conversation state, identity, turn-based concurrency
- **Actions as Dapr Activities**: Each tool/action is a separate activity
- **Orchestration via Dapr Workflow**: Workflow calls agent actor, which decides next action
- **State**: Actor state stores conversation history, workflow state stores overall progress

**Architecture**:
```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│  Agent Actor    │    │  Action         │    │  Workflow       │
│  (LLM reasoning)│───▶│  Activities     │◀───│  (Orchestration)│
└─────────────────┘    └─────────────────┘    └─────────────────┘
         │                       │                       │
         ▼                       ▼                       ▼
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│  Actor State    │    │  State Store    │    │  Workflow State │
│  (conversation) │    │  (tool results) │    │  (progress)     │
└─────────────────┘    └─────────────────┘    └─────────────────┘
```

### 2. Map-Reduce Pattern (Batch Processing)
**Python**: BatchNode processes iterable items, aggregates results.
**Dapr Mapping**:
- **Map as Dapr Workflow fan-out**: Use `for_each` with parallelism limit
- **Reduce as Dapr Activity**: Aggregation activity after fan-out
- **State**: Use Dapr State Management to collect intermediate results
- **Idempotency**: Each map item processed idempotently via deterministic keys

**Architecture**:
```
┌─────────────────────────────────────────────────────────┐
│                  Map-Reduce Workflow                    │
├─────────────────────────────────────────────────────────┤
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐ │
│  │   Input     │───▶│  Fan-out    │───▶│   Reduce    │ │
│  │   Items     │    │  (parallel) │    │  Activity   │ │
│  └─────────────┘    └─────────────┘    └─────────────┘ │
│         │                   │                  │        │
│         ▼                   ▼                  ▼        │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐ │
│  │  State      │    │  Activity   │    │  State      │ │
│  │  (input)    │    │  (map fn)   │    │  (output)   │ │
│  └─────────────┘    └─────────────┘    └─────────────┘ │
└─────────────────────────────────────────────────────────┘
```

### 3. RAG Pattern (Retrieval-Augmented Generation)
**Python**: Retrieve documents, generate answer with context.
**Dapr Mapping**:
- **Retrieval as Dapr Binding**: Use vector DB binding for similarity search
- **Generation as Dapr Activity**: LLM call with retrieved context
- **State**: Store retrieved documents in Dapr State for caching
- **Workflow**: Sequential retrieve→generate, or iterative retrieve→generate→refine

**Architecture**:
```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│  Query          │───▶│  Vector DB      │───▶│  Retrieved      │
│  Input          │    │  Binding        │    │  Documents      │
└─────────────────┘    └─────────────────┘    └─────────────────┘
         │                       │                       │
         ▼                       ▼                       ▼
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│  LLM Activity   │◀───│  Context        │    │  State Cache    │
│  (generation)   │    │  Assembly       │    │  (documents)    │
└─────────────────┘    └─────────────────┘    └─────────────────┘
```

### 4. Multi-Agent Pattern (Agent Communication)
**Python**: Multiple agents communicate via async queues.
**Dapr Mapping**:
- **Agents as Dapr Actors**: Each agent is an actor with identity
- **Communication via Dapr Pub/Sub**: Agents publish messages to topics
- **Orchestration via Dapr Workflow**: Supervisor workflow coordinates agents
- **State**: Actor state per agent, shared state via Dapr State Management

**Architecture**:
```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│  Agent A Actor  │───▶│  Pub/Sub Topic  │◀───│  Agent B Actor  │
└─────────────────┘    └─────────────────┘    └─────────────────┘
         │                       │                       │
         ▼                       ▼                       ▼
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│  Supervisor     │───▶│  Coordination   │◀───│  Shared State   │
│  Workflow       │    │  (messages)     │    │  (Dapr State)   │
└─────────────────┘    └─────────────────┘    └─────────────────┘
```

### 5. Supervisor Pattern (Quality Control Loop)
**Python**: Supervisor evaluates agent outputs, provides feedback, loops until quality threshold.
**Dapr Mapping**:
- **Supervisor as Dapr Workflow**: Loop with evaluation activity
- **Evaluation as Dapr Activity**: LLM-based quality assessment
- **Feedback via Dapr State**: Store feedback in shared state
- **Termination**: Workflow continues until quality score meets threshold

**Architecture**:
```
┌─────────────────────────────────────────────────────────┐
│                Supervisor Workflow Loop                 │
├─────────────────────────────────────────────────────────┤
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐ │
│  │  Agent      │───▶│  Evaluation │───▶│  Feedback   │ │
│  │  Execution  │    │  Activity   │    │  Storage    │ │
│  └─────────────┘    └─────────────┘    └─────────────┘ │
│         │                   │                  │        │
│         ▼                   ▼                  ▼        │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐ │
│  │  State      │    │  Quality    │    │  Continue?  │ │
│  │  (output)   │    │  Score      │    │  (branch)   │ │
│  └─────────────┘    └─────────────┘    └─────────────┘ │
└─────────────────────────────────────────────────────────┘
```

## Cross-Cutting Concerns

### State Management
- **Agent state**: Actor state for conversation history
- **Workflow state**: Dapr Workflow checkpointing
- **Shared state**: Dapr State Management for cross-agent data

### Communication
- **Synchronous**: Direct activity calls within workflow
- **Asynchronous**: Dapr Pub/Sub for agent-to-agent messaging
- **Request-response**: Dapr Service invocation for tool calls

### Error Handling
- **Retry**: Dapr retry policies for activities
- **Compensation**: Dapr workflow compensation for rollback
- **Fallback**: Alternative workflow paths on failure

## Open Questions

1. **Actor vs Activity for agents**: Should agents be actors (stateful) or activities (stateless) called by workflow?
2. **Communication patterns**: Which agents need pub/sub vs direct workflow calls?
3. **State sharing**: How to share state between agents without tight coupling?
4. **Idempotency**: How to ensure agent actions are idempotent for retries?
5. **Observability**: How to trace multi-agent conversations across Dapr components?

## Next Steps

1. Create OpenSpec change for design patterns porting plan
2. Define detailed specs for each pattern mapping
3. Design Rust API with examples
4. Plan implementation phases