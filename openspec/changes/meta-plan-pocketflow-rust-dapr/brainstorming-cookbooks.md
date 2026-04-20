# Brainstorming: Cookbooks Porting Plan

## Current State (Python PocketFlow)
- **59 cookbook examples** covering various patterns and utilities
- Examples include: hello-world, agent, RAG, batch, chat, debate, deep-research, etc.
- Each cookbook is a self-contained Python script with dependencies

## Goals for Rust Porting
1. Port all cookbooks to Rust with Dapr integration
2. Ensure functional equivalence with Python versions
3. Create comprehensive test harness for validation
4. Provide clear documentation and migration guides

## Prioritization Strategy

### Tier 1: Core Examples (Port First)
- **hello-world**: Basic Node and Flow usage
- **flow**: Simple workflow orchestration
- **batch-node**: Batch processing
- **batch-flow**: Batch workflow
- **chat**: Basic chatbot with memory

### Tier 2: Pattern Examples
- **agent**: Agent pattern with tool usage
- **agentic-rag**: RAG with agent reasoning
- **coding-agent**: Code generation agent
- **debate**: Multi-agent debate
- **deep-research**: Research agent

### Tier 3: Integration Examples
- **fastapi-websocket**: WebSocket API integration
- **fastapi-hitl**: Human-in-the-loop API
- **google-calendar**: External API integration
- **gradio-hitl**: Gradio UI integration

### Tier 4: Advanced Examples
- **communication**: Agent communication patterns
- **judge**: Quality control loops
- **lead-generation**: Business workflow
- **invoice**: Document processing

## Test Harness Design

### Architecture
```
┌─────────────────────────────────────────────────────────┐
│                  Test Harness                           │
├─────────────────────────────────────────────────────────┤
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐ │
│  │  Python     │    │  Rust       │    │  Comparator │ │
│  │  Runner     │    │  Runner     │    │  (Output)   │ │
│  └─────────────┘    └─────────────┘    └─────────────┘ │
│         │                   │                  │        │
│         ▼                   ▼                  ▼        │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐ │
│  │  Input      │    │  Input      │    │  Results    │ │
│  │  Generator  │    │  Generator  │    │  Database   │ │
│  └─────────────┘    └─────────────┘    └─────────────┘ │
└─────────────────────────────────────────────────────────┘
```

### Components
1. **Input Generator**: Creates test inputs for each cookbook
2. **Python Runner**: Executes Python cookbooks in isolated environment
3. **Rust Runner**: Executes Rust examples with Dapr sidecar (mock)
4. **Comparator**: Compares outputs for functional equivalence
5. **Results Database**: Stores test results and metrics

### Integration Testing Framework
- **Unit tests**: For each Rust module
- **Integration tests**: For each cookbook with Dapr components
- **Compatibility tests**: Compare Python vs Rust outputs
- **Performance tests**: Benchmark Rust vs Python execution

## Dapr Integration for Cookbooks

### Required Dapr Components
1. **State Store**: For shared state and caching
2. **Pub/Sub**: For agent communication patterns
3. **Bindings**: For external APIs (web search, vector DB, etc.)
4. **Secrets**: For API keys and configuration
5. **Workflows**: For orchestration patterns

### Mocking Strategy
- Use Dapr CLI in standalone mode for testing
- Mock Dapr components for unit tests
- Use testcontainers for integration tests

## Open Questions

1. **Scope**: Should we port all 59 cookbooks or only essential ones?
2. **Testing**: How to handle external API dependencies in tests?
3. **Documentation**: How to document differences between Python and Rust versions?
4. **Maintenance**: How to keep Rust cookbooks updated as Python versions evolve?
5. **Distribution**: How to package and distribute Rust cookbooks?

## Next Steps

1. Create OpenSpec change for cookbooks porting plan
2. Define detailed specs for each cookbook category
3. Design test harness architecture
4. Plan implementation phases