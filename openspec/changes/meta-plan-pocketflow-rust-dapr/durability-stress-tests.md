# PocketFlow Durability Stress Test Scenarios

## Executive Summary

This document defines comprehensive stress-test scenarios for validating durability requirements in the PocketFlow Rust+Dapr implementation. Based on analysis of 59 cookbook examples, these scenarios cover state preservation, failure recovery, and checkpoint consistency across all major workflow patterns.

**State Categories Identified:**
- **Conversation State**: messages, context, memory vectors
- **Agent State**: search history, decisions, research notes, action plans
- **Multi-Agent State**: queues, coordination tokens, turn tracking
- **Batch State**: processing items, partial results, completion tracking
- **Research State**: queries, notes, iterations, loop counters
- **Self-Healing State**: attempts, errors, retry counters
- **HITL State**: pending approvals, feedback, user decisions
- **RAG State**: embeddings, indices, retrieved documents

---

## 1. Stress Test Categories

### 1.1 Chat & Conversational Workflows

**Cookbook Sources**: `pocketflow-chat`, `pocketflow-chat-memory`, `pocketflow-chat-guardrail`, `pocketflow-voice-chat`

#### Scenario 1.1.1: Mid-Conversation Crash Recovery
- **Scenario Name**: Conversation Interruption at User Input
- **Initial State**: `shared.messages` with N conversation pairs, `shared.vector_index` (if memory enabled)
- **Execution Pattern**: Self-looping node pattern with user input (blocking)
- **Failure Injection**: 
  - `kill -9` after user input but before LLM response stored
  - Panic during vector embedding computation
  - Network drop during retrieval phase
- **Expected Behavior**: 
  - Recover to last consistent checkpoint
  - Replay from last user input or resume from stored messages
  - No duplicate message insertion
- **Success Criteria**:
  - Conversation history preserved exactly
  - LLM response regenerated (acceptable)
  - Vector index consistency maintained (if applicable)
- **Performance Requirements**:
  - Recovery time: < 5 seconds
  - Max data loss: Last user input only (regeneratable)

#### Scenario 1.1.2: Memory Vector State Loss
- **Scenario Name**: Vector Index Corruption During Archival
- **Initial State**: `shared.messages` > 6 items (triggering archival), `shared.vector_index`, `shared.vector_items`
- **Execution Pattern**: EmbedNode archiving oldest pairs to vector store
- **Failure Injection**:
  - Crash after embedding created but before index update
  - Partial write to vector_items list
  - Dapr state store timeout during commit
- **Expected Behavior**:
  - Detect inconsistent vector_items vs vector_index
  - Re-embed orphaned conversations on recovery
  - Maintain conversation continuity
- **Success Criteria**:
  - No orphaned embeddings
  - Conversation window preserved (last 6 pairs)
  - Retrieved conversations still accessible
- **Performance Requirements**:
  - Recovery time: < 10 seconds
  - Max data loss: 1 conversation pair (re-embedded)

#### Scenario 1.1.3: Multi-Turn Context Preservation
- **Scenario Name**: Long-Running Conversation Recovery
- **Initial State**: 50+ conversation pairs, extensive `shared.context`
- **Execution Pattern**: Continuous self-loop over hours of interaction
- **Failure Injection**:
  - Process OOM during large context window
  - Container restart during active conversation
  - Dapr sidecar disconnect mid-flow
- **Expected Behavior**:
  - Full context recovery
  - Resume from exact node/loop position
  - Maintain conversation coherence
- **Success Criteria**:
  - All conversation history preserved
  - Current loop iteration tracked
  - No context truncation beyond configured limits
- **Performance Requirements**:
  - Recovery time: < 3 seconds
  - Context loading: < 1 second per 100 pairs

---

### 1.2 Agent Workflows

**Cookbook Sources**: `pocketflow-agent`, `pocketflow-agent-skills`, `pocketflow-coding-agent`, `pocketflow-thinking`

#### Scenario 1.2.1: Research Agent Interruption
- **Scenario Name**: Web Search Cycle Recovery
- **Initial State**: `shared.question`, `shared.context` (search results), `shared.search_query` (current)
- **Execution Pattern**: DecideAction → SearchWeb → (loop back)
- **Failure Injection**:
  - Crash during web search API call
  - Network timeout during search results processing
  - YAML parsing failure in decision node
- **Expected Behavior**:
  - Retry search with exponential backoff
  - Maintain accumulated context
  - Preserve decision history
- **Success Criteria**:
  - Context append-only, never lost
  - Search results idempotent (re-search acceptable)
  - Decision node receives full accumulated context
- **Performance Requirements**:
  - Recovery time: < 15 seconds
  - Max search retries: 3 before escalation

#### Scenario 1.2.2: Action Plan State Consistency
- **Scenario Name**: Multi-Step Agent Plan Recovery
- **Initial State**: `shared.action_plan`, `shared.completed_steps`, `shared.current_step`
- **Execution Pattern**: Sequential step execution with dependencies
- **Failure Injection**:
  - Crash mid-step execution
  - Tool call failure during external API
  - State inconsistency between steps
- **Expected Behavior**:
  - Resume from last completed step
  - Detect partial step execution
  - Rollback or retry partial operations
- **Success Criteria**:
  - Step execution atomicity
  - No duplicate step completions
  - Dependency chain preserved
- **Performance Requirements**:
  - Step recovery: < 5 seconds
  - Checkpoint every step completion

#### Scenario 1.2.3: Agent Tool Integration Failure
- **Scenario Name**: External Tool Timeout Recovery
- **Initial State**: `shared.tool_calls` (pending), `shared.tool_results` (partial)
- **Execution Pattern**: Parallel tool execution, aggregation
- **Failure Injection**:
  - Tool API timeout (30s+)
  - Tool returns 500 error
  - Partial tool results received
- **Expected Behavior**:
  - Retry failed tools
  - Continue with partial results if configured
  - Clear tool call state on completion
- **Success Criteria**:
  - Tool timeout handled gracefully
  - Results aggregation correct
  - No orphaned tool calls
- **Performance Requirements**:
  - Tool timeout: 30s default, configurable
  - Retry delay: exponential backoff

---

### 1.3 Multi-Agent Workflows

**Cookbook Sources**: `pocketflow-multi-agent`, `pocketflow-supervisor`, `pocketflow-debate`, `pocketflow-communication`

#### Scenario 1.3.1: Multi-Agent Coordination Failure
- **Scenario Name**: Async Agent Queue Recovery
- **Initial State**: `shared.hinter_queue`, `shared.guesser_queue`, `shared.past_guesses`, `shared.target_word`
- **Execution Pattern**: Two async flows communicating via queues
- **Failure Injection**:
  - One agent flow crashes while other waiting
  - Queue message loss
  - Deadlock between agents
- **Expected Behavior**:
  - Detect stalled agent
  - Recover queue state
  - Resume or restart agent pair
- **Success Criteria**:
  - Queue state consistent
  - No duplicate message processing
  - Game state (past_guesses) preserved
- **Performance Requirements**:
  - Agent timeout detection: 60s
  - Queue recovery: < 5 seconds

#### Scenario 1.3.2: Supervisor Pattern Recovery
- **Scenario Name**: Supervised Agent Retry Loop
- **Initial State**: `shared.answer` (rejected), `shared.context` (with rejection note)
- **Execution Pattern**: Inner agent flow → Supervisor → (retry if rejected)
- **Failure Injection**:
  - Supervisor crash during validation
  - Inner flow restart with stale context
  - Infinite retry loop scenario
- **Expected Behavior**:
  - Track retry attempts
  - Max retry enforcement
  - Context accumulation on rejection
- **Success Criteria**:
  - Max 3 retries enforced
  - Rejection notes accumulated
  - No infinite loops
- **Performance Requirements**:
  - Retry delay: 1s between attempts
  - Supervisor timeout: 10s

#### Scenario 1.3.3: Debate State Preservation
- **Scenario Name**: Multi-Round Debate Recovery
- **Initial State**: `shared.case_for`, `shared.case_against`, `shared.verdict`, scores
- **Execution Pattern**: AdvocateFor → AdvocateAgainst → Judge (sequential)
- **Failure Injection**:
  - Crash during argument generation
  - Partial verdict storage
  - Judge node failure
- **Expected Behavior**:
  - Resume from failed node
  - Regenerate arguments if needed
  - Preserve completed arguments
- **Success Criteria**:
  - Arguments reproducible
  - Verdict atomic write
  - No partial debate state exposed
- **Performance Requirements**:
  - Argument recovery: < 10 seconds
  - Debate state checkpointed per round

---

### 1.4 Batch Processing Workflows

**Cookbook Sources**: `pocketflow-batch`, `pocketflow-batch-node`, `pocketflow-batch-flow`, `pocketflow-parallel-batch`, `pocketflow-nested-batch`

#### Scenario 1.4.1: Batch Node Partial Completion
- **Scenario Name**: Translation Batch Interruption
- **Initial State**: `shared.text`, `shared.languages` (8 items), `shared.output_dir`
- **Execution Pattern**: BatchNode processing items sequentially or in parallel
- **Failure Injection**:
  - Crash after 3 of 8 translations complete
  - File write failure during output
  - Dapr state size limit exceeded
- **Expected Behavior**:
  - Track completed items
  - Resume from last uncompleted item
  - Idempotent output writes
- **Success Criteria**:
  - Exactly 8 translations completed
  - No duplicate translations
  - Partial results not lost
- **Performance Requirements**:
  - Batch checkpoint every item
  - Recovery time proportional to remaining items

#### Scenario 1.4.2: Parallel Batch Race Conditions
- **Scenario Name**: Concurrent Translation State Merge
- **Initial State**: `shared.translations` (partial results from parallel workers)
- **Execution Pattern**: AsyncParallelBatchNode with concurrent execution
- **Failure Injection**:
  - Worker node crash during exec
  - Concurrent writes to shared state
  - Partial worker results
- **Expected Behavior**:
  - Thread-safe state updates
  - Merge partial results correctly
  - Detect and handle worker failures
- **Success Criteria**:
  - All worker results collected
  - No lost translations
  - Shared state consistent
- **Performance Requirements**:
  - Worker timeout: 60s
  - State merge: atomic operation

#### Scenario 1.4.3: Nested Batch Workflow
- **Scenario Name**: Image Processing Pipeline Recovery
- **Initial State**: `shared.images` (batch), `shared.filters` (nested batch)
- **Execution Pattern**: Outer batch → Inner batch per image
- **Failure Injection**:
  - Crash during inner batch (filter application)
  - Image load failure
  - Filter processing error
- **Expected Behavior**:
  - Resume from specific image+filter position
  - Skip completed filters
  - Track nested progress
- **Success Criteria**:
  - Nested progress tracked
  - No reprocessing of completed work
  - Partial outputs preserved
- **Performance Requirements**:
  - Nested checkpoint granularity
  - Recovery at inner batch level

---

### 1.5 Long-Running Research Workflows

**Cookbook Sources**: `pocketflow-deep-research`, `pocketflow-lead-generation`, `pocketflow-notebook-lm`

#### Scenario 1.5.1: Deep Research Loop Recovery
- **Scenario Name**: Recursive Research Iteration Crash
- **Initial State**: `shared.topic`, `shared.notes` (accumulated), `shared.loop_count`, `shared.feedback`
- **Execution Pattern**: Planner → Researcher → Synthesizer → (loop if gaps found, max 2 loops)
- **Failure Injection**:
  - Crash during web search in loop iteration
  - Synthesizer node failure after research
  - Loop counter corruption
- **Expected Behavior**:
  - Preserve accumulated notes across loops
  - Enforce max loop count
  - Track loop iteration atomically
- **Success Criteria**:
  - Notes append-only
  - Loop count never exceeds 2
  - Feedback context maintained
- **Performance Requirements**:
  - Checkpoint per research iteration
  - Notes backup every loop

#### Scenario 1.5.2: Report Generation Failure
- **Scenario Name**: Final Report Compilation Crash
- **Initial State**: `shared.notes` (complete), `shared.report` (generating)
- **Execution Pattern**: Synthesizer generates final report from all notes
- **Failure Injection**:
  - Crash during LLM report generation
  - Report exceeds state size limits
  - YAML parsing error in output
- **Expected Behavior**:
  - Retry report generation
  - Stream large reports to external storage
  - Validate report format
- **Success Criteria**:
  - Complete report generated
  - All notes incorporated
  - Report retrievable
- **Performance Requirements**:
  - Report timeout: 120s
  - Streaming threshold: 100KB

#### Scenario 1.5.3: Lead Generation Pipeline
- **Scenario Name**: Multi-Stage Lead Processing Interruption
- **Initial State**: `shared.leads` (raw), `shared.enriched_leads` (partial), `shared.qualified_leads`
- **Execution Pattern**: Source → Enrich → Qualify → Export
- **Failure Injection**:
  - API rate limit during enrichment
  - Qualification criteria change mid-flow
  - Export target unavailable
- **Expected Behavior**:
  - Pause on rate limits
  - Resume enrichment from checkpoint
  - Export retry with backoff
- **Success Criteria**:
  - All leads processed
  - No duplicates in export
  - Qualification consistent
- **Performance Requirements**:
  - API rate limit handling
  - Checkpoint every 100 leads

---

### 1.6 Self-Healing Workflows

**Cookbook Sources**: `pocketflow-self-healing-mermaid`, `pocketflow-judge`

#### Scenario 1.6.1: Retry Loop State Accumulation
- **Scenario Name**: Mermaid Chart Compilation Recovery
- **Initial State**: `shared.attempts` (failed attempts), `shared.chart` (current), `shared.task`
- **Execution Pattern**: WriteChart → CompileChart → (loop if fail, max 3 attempts)
- **Failure Injection**:
  - Crash after adding to attempts but before retry
  - Compilation subprocess timeout
  - Attempt counter overflow
- **Expected Behavior**:
  - Preserve attempt history
  - Limit retries to 3
  - Pass error history to next attempt
- **Success Criteria**:
  - Max 3 attempts enforced
  - Error history accumulated correctly
  - Final success or graceful failure
- **Performance Requirements**:
  - Compilation timeout: 60s
  - Retry delay: 5s

#### Scenario 1.6.2: Error Feedback Preservation
- **Scenario Name**: Error Context Accumulation
- **Initial State**: `shared.attempts` with N previous errors
- **Execution Pattern**: Each retry receives cumulative error history
- **Failure Injection**:
  - Error history truncation
  - Error message corruption
  - History loss between attempts
- **Expected Behavior**:
  - Append-only error history
  - Error deduplication if identical
  - History size limits
- **Success Criteria**:
  - All errors available to LLM
  - No error loss between retries
  - History bounded (max 10KB)
- **Performance Requirements**:
  - Error history load: < 100ms
  - History size limit: 10KB

---

### 1.7 Human-in-the-Loop (HITL) Workflows

**Cookbook Sources**: `pocketflow-cli-hitl`, `pocketflow-fastapi-hitl`, `pocketflow-gradio-hitl`, `pocketflow-fastapi-background`

#### Scenario 1.7.1: HITL Approval Timeout
- **Scenario Name**: Human Review Timeout Recovery
- **Initial State**: `shared.processed_output` (waiting review), `shared.review_event` (async)
- **Execution Pattern**: Process → Review (wait for human) → Result
- **Failure Injection**:
  - Human reviewer unavailable (timeout)
  - Process crash while waiting for review
  - Review decision lost
- **Expected Behavior**:
  - Timeout after configurable period (default 1 hour)
  - Escalation or auto-reject on timeout
  - Preserve output for later review
- **Success Criteria**:
  - Timeout enforced
  - No indefinite blocking
  - Escalation workflow triggered
- **Performance Requirements**:
  - Review timeout: 1 hour (configurable)
  - State persistence during wait

#### Scenario 1.7.2: Concurrent Review Scenarios
- **Scenario Name**: Multiple Reviewers Race Condition
- **Initial State**: `shared.sse_queue` (SSE updates), `shared.review_event`
- **Execution Pattern**: Async review with SSE notifications
- **Failure Injection**:
  - Multiple review submissions
  - SSE queue overflow
  - Review decision conflict
- **Expected Behavior**:
  - First review wins
  - Subsequent reviews rejected
  - SSE queue bounded
- **Success Criteria**:
  - Exactly one decision processed
  - No duplicate processing
  - Queue bounded (max 100 messages)
- **Performance Requirements**:
  - Review deduplication: < 100ms
  - Queue bounds enforced

#### Scenario 1.7.3: Joke Generator Feedback Loop
- **Scenario Name**: Iterative HITL with Accumulated Feedback
- **Initial State**: `shared.disliked_jokes` (history), `shared.current_joke`, `shared.user_feedback`
- **Execution Pattern**: Generate → Show → (loop if rejected)
- **Failure Injection**:
  - Crash after user rejection but before feedback stored
  - Feedback loss
  - Disliked jokes history corruption
- **Expected Behavior**:
  - Accumulate disliked jokes
  - Use feedback for regeneration
  - Max attempts enforcement
- **Success Criteria**:
  - Disliked jokes preserved
  - Feedback influences generation
  - Graceful exit after max attempts
- **Performance Requirements**:
  - Feedback persistence: immediate
  - History loading: < 1s

---

### 1.8 RAG (Retrieval-Augmented Generation) Workflows

**Cookbook Sources**: `pocketflow-rag`, `pocketflow-agentic-rag`, `pocketflow-tool-embeddings`

#### Scenario 1.8.1: Index Building Failure
- **Scenario Name**: Document Index Construction Crash
- **Initial State**: `shared.texts` (chunks), `shared.embeddings` (partial), `shared.index` (building)
- **Execution Pattern**: Chunk → Embed → Index → Store
- **Failure Injection**:
  - Crash during embedding generation
  - FAISS index corruption
  - Index storage failure
- **Expected Behavior**:
  - Resume embedding from checkpoint
  - Rebuild index if corrupted
  - Validate index consistency
- **Success Criteria**:
  - All documents embedded
  - Index searchable
  - Consistent text/embedding mapping
- **Performance Requirements**:
  - Embedding checkpoint every 100 docs
  - Index validation: < 5s

#### Scenario 1.8.2: Query-Time Retrieval Failure
- **Scenario Name**: Retrieval Query Recovery
- **Initial State**: `shared.query`, `shared.query_embedding`, `shared.index`
- **Execution Pattern**: EmbedQuery → Retrieve → GenerateAnswer
- **Failure Injection**:
  - Index lookup timeout
  - Retrieved document reference invalid
  - Query embedding failure
- **Expected Behavior**:
  - Retry retrieval
  - Handle missing documents gracefully
  - Fallback to no retrieval
- **Success Criteria**:
  - Valid document retrieved
  - No index corruption
  - Graceful degradation
- **Performance Requirements**:
  - Retrieval timeout: 5s
  - Index consistency check: < 1s

---

### 1.9 Async Workflows

**Cookbook Sources**: `pocketflow-async-basic`, `pocketflow-fastapi-websocket`

#### Scenario 1.9.1: Async I/O State Consistency
- **Scenario Name**: Recipe Fetch Recovery
- **Initial State**: `shared.recipes`, `shared.ingredient`, `shared.suggestion`
- **Execution Pattern**: Async fetch → Async LLM → Async user input
- **Failure Injection**:
  - Event loop crash
  - Async task cancellation
  - Coroutine state loss
- **Expected Behavior**:
  - Preserve async task results
  - Resume from await point
  - Handle task cancellation
- **Success Criteria**:
  - Async results preserved
  - No blocking on recovery
  - State consistent after await
- **Performance Requirements**:
  - Async checkpoint: after each await
  - Recovery: resume coroutine

#### Scenario 1.9.2: WebSocket Connection Recovery
- **Scenario Name**: Streaming State Preservation
- **Initial State**: `shared.stream_buffer`, `shared.websocket_connected`
- **Execution Pattern**: Streaming LLM output over WebSocket
- **Failure Injection**:
  - WebSocket disconnect mid-stream
  - Client reconnect with stale state
  - Buffer overflow
- **Expected Behavior**:
  - Buffer recent chunks
  - Replay buffer on reconnect
  - Handle reconnection state
- **Success Criteria**:
  - Stream continuity maintained
  - No duplicate chunks
  - Buffer bounded
- **Performance Requirements**:
  - Buffer size: max 100 chunks
  - Reconnect replay: < 500ms

---

### 1.10 Tool Integration Workflows

**Cookbook Sources**: `pocketflow-tool-search`, `pocketflow-tool-crawler`, `pocketflow-tool-database`, `pocketflow-tool-pdf-vision`, `pocketflow-mcp`

#### Scenario 1.10.1: Web Search API Failure
- **Scenario Name**: DuckDuckGo Search Timeout
- **Initial State**: `shared.search_query`, `shared.search_results` (partial)
- **Execution Pattern**: Tool call → API request → Result processing
- **Failure Injection**:
  - API timeout (30s+)
  - Rate limit exceeded (429)
  - Temporary network failure
- **Expected Behavior**:
  - Retry with exponential backoff
  - Cache successful results
  - Degrade gracefully on persistent failure
- **Success Criteria**:
  - Max 3 retries
  - Backoff: 1s, 2s, 4s
  - Fallback to cached or empty results
- **Performance Requirements**:
  - API timeout: 30s
  - Total retry window: < 60s

#### Scenario 1.10.2: Database Transaction Recovery
- **Scenario Name**: SQL Query Execution Failure
- **Initial State**: `shared.query`, `shared.transaction_id`, `shared.db_connection`
- **Execution Pattern**: Connect → Query → Process → Commit
- **Failure Injection**:
  - Connection pool exhaustion
  - Query timeout
  - Constraint violation
- **Expected Behavior**:
  - Rollback on failure
  - Connection cleanup
  - Retry with modified query
- **Success Criteria**:
  - No partial commits
  - Connection leaks prevented
  - Transaction atomicity
- **Performance Requirements**:
  - Query timeout: 30s
  - Connection timeout: 10s

---

## 2. Specific Test Scenarios (20+ Detailed Cases)

### 2.1 Mid-Conversation Crashes (Chat Agents)

#### Test 2.1.1: User Input Capture Failure
```yaml
Scenario: T-001
Name: Input-Response Gap Crash
Cookbook: pocketflow-chat, pocketflow-chat-memory
Severity: Critical

Initial State:
  shared.messages: [{role: user, content: "Hello"}, {role: assistant, content: "Hi!"}]
  shared.vector_index: <FAISS index with 2 archived pairs>
  
Execution Pattern:
  1. User types: "What's the weather?"
  2. Prep reads input
  3. Exec calls LLM (10s latency)
  4. [INJECT CRASH] Before post() stores response
  
Failure Injection:
  Method: kill -9 <pid>
  Timing: 5 seconds into exec() (mid-LLM call)
  
Expected Behavior:
  - Workflow restarts with checkpoint from before exec()
  - User input "What's the weather?" still in shared.messages
  - LLM call re-executed
  - Response stored successfully
  
Success Criteria:
  - No duplicate user messages
  - Exactly one assistant response
  - Conversation flow coherent
  
State Validation:
  Pre-crash: messages=[user:Hello, asst:Hi, user:What's weather?]
  Post-recovery: messages=[user:Hello, asst:Hi, user:What's weather?, asst:<response>]
  
Performance:
  Recovery Time: < 5s
  Data Loss: 0 (LLM response regenerated)
```

#### Test 2.1.2: Vector Embedding Corruption
```yaml
Scenario: T-002
Name: Embedding-Index Desync
Cookbook: pocketflow-chat-memory
Severity: High

Initial State:
  shared.messages: [6 pairs of conversation]
  shared.vector_index: <valid FAISS index>
  shared.vector_items: [2 archived conversations]
  
Execution Pattern:
  EmbedNode.prep: Extract oldest 2 messages
  EmbedNode.exec: Generate embedding
  EmbedNode.post: Add to index [INJECT CRASH after index.add but before vector_items.append]
  
Failure Injection:
  Method: Panic during post()
  Timing: After add_vector(), before updating vector_items
  
Expected Behavior:
  - Detect orphaned embedding in index
  - Or re-embed the conversation pair
  - Index and vector_items remain consistent
  
Success Criteria:
  - vector_items.length == index.ntotal
  - No orphaned embeddings
  - Retrieval works correctly
  
State Validation:
  Check: len(shared.vector_items) == shared.vector_index.ntotal
  Repair: Re-embed if mismatch detected
```

### 2.2 Multi-Agent Coordination Failures

#### Test 2.2.1: Taboo Game Agent Crash
```yaml
Scenario: T-003
Name: Multi-Agent Queue Desync
Cookbook: pocketflow-multi-agent
Severity: Critical

Initial State:
  shared.hinter_queue: asyncio.Queue with 1 pending message
  shared.guesser_queue: asyncio.Queue empty
  shared.past_guesses: ["happy", "sad"]
  shared.target_word: "nostalgic"
  
Execution Pattern:
  - Hinter waiting on hinter_queue.get()
  - Guesser sends guess to hinter_queue
  - [INJECT CRASH] Guesser crashes after put(), before returning
  
Failure Injection:
  Method: kill -9 on guesser process
  Timing: After queue.put(), during post_async
  
Expected Behavior:
  - Hinter times out waiting (60s)
  - Detect agent failure
  - Restart both agents with preserved game state
  - Resume from last valid state
  
Success Criteria:
  - past_guesses preserved
  - Game resumes from valid state
  - No duplicate guesses processed
  
State Validation:
  Check: All guesses in past_guesses are unique
  Recovery: Restart both agents, re-initialize queues
```

#### Test 2.2.2: Supervisor Infinite Loop Prevention
```yaml
Scenario: T-004
Name: Supervisor Retry Exhaustion
Cookbook: pocketflow-supervisor
Severity: High

Initial State:
  shared.question: "What is AI?"
  shared.context: <previous failed attempts>
  shared.answer: <rejected 2 times>
  
Execution Pattern:
  - Agent flow produces answer
  - Supervisor rejects (3rd time)
  - Should stop but [INJECT] supervisor returns retry
  
Failure Injection:
  Method: Force supervisor to always return "retry"
  Timing: Every post() call
  
Expected Behavior:
  - Hard limit: max 3 retries
  - After 3rd retry, force completion or error
  - Log warning about retry exhaustion
  
Success Criteria:
  - Loop terminates after 3 retries
  - No infinite loops
  - System remains responsive
  
State Validation:
  Check: loop_count <= 3
  Action: Force exit if exceeded
```

### 2.3 Batch Processing Interruptions

#### Test 2.3.1: Translation Batch Resume
```yaml
Scenario: T-005
Name: Partial Batch Recovery
Cookbook: pocketflow-batch, pocketflow-parallel-batch
Severity: High

Initial State:
  shared.text: <README content>
  shared.languages: ["Chinese", "Spanish", "Japanese", "German", "Russian"]
  shared.output_dir: "translations"
  
Execution Pattern:
  BatchNode processes languages sequentially
  Completed: Chinese, Spanish
  In Progress: Japanese
  Pending: German, Russian
  [INJECT CRASH] During Japanese translation
  
Failure Injection:
  Method: kill -9 during exec() of Japanese
  Timing: 50% through LLM call
  
Expected Behavior:
  - Checkpoint after each completed language
  - Resume from Japanese (not restart)
  - Chinese and Spanish files exist
  
Success Criteria:
  - Chinese and Spanish not re-translated
  - Japanese completed after recovery
  - All 5 languages have output files
  
State Validation:
  Files: translations/README_*.md count == 5
  Content: Each file non-empty and valid
```

#### Test 2.3.2: Parallel Batch Worker Failure
```yaml
Scenario: T-006
Name: Concurrent Worker Crash
Cookbook: pocketflow-parallel-batch
Severity: Critical

Initial State:
  shared.translations: {} (results dict)
  8 concurrent workers processing languages
  
Execution Pattern:
  - Workers execute in parallel
  - Worker 3 (Japanese) crashes
  - Other workers continue
  
Failure Injection:
  Method: Panic in worker 3 during exec_async
  Timing: Random point in execution
  
Expected Behavior:
  - Detect worker failure
  - Retry failed worker (Japanese)
  - Merge results from successful workers
  
Success Criteria:
  - 7 languages succeed on first try
  - Japanese succeeds on retry
  - No data corruption in shared.translations
  
State Validation:
  Check: All 8 languages in results
  Check: Japanese result valid
```

### 2.4 Self-Healing Loop Interruptions

#### Test 2.4.1: Mermaid Compilation Exhaustion
```yaml
Scenario: T-007
Name: Max Retry Enforcement
Cookbook: pocketflow-self-healing-mermaid
Severity: Medium

Initial State:
  shared.task: "Create flowchart for login process"
  shared.attempts: [attempt1, attempt2] (2 failed)
  shared.chart: <invalid mermaid code>
  
Execution Pattern:
  WriteChart → CompileChart (fails) → Loop
  [INJECT] Force compilation to fail forever
  
Failure Injection:
  Method: Mock mmdc to always return error
  Timing: Every CompileChart.exec call
  
Expected Behavior:
  - Attempt 3 executed
  - After 3rd failure, exit gracefully
  - Log error and final state
  
Success Criteria:
  - Exactly 3 attempts
  - Graceful termination
  - Error history preserved
  
State Validation:
  Check: len(shared.attempts) == 3
  Check: Flow terminates (not hung)
```

#### Test 2.4.2: Error Feedback Accumulation
```yaml
Scenario: T-008
Name: Error Context Persistence
Cookbook: pocketflow-self-healing-mermaid
Severity: Medium

Initial State:
  shared.attempts: []
  
Execution Pattern:
  Attempt 1: Syntax error on line 5
  Attempt 2: Syntax error on line 8
  Attempt 3: Success
  [INJECT CRASH] After attempt 2, before attempt 3
  
Failure Injection:
  Method: kill -9 after post() of attempt 2
  Timing: During transition back to WriteChart
  
Expected Behavior:
  - Preserve attempts[0] and attempts[1]
  - WriteChart receives both errors
  - Attempt 3 uses cumulative feedback
  
Success Criteria:
  - Both errors in history
  - Attempt 3 aware of both
  - Final chart compiles
  
State Validation:
  Check: len(shared.attempts) == 2 after crash
  Check: Both errors in WriteChart prompt
```

### 2.5 Long-Running Research Workflow Crashes

#### Test 2.5.1: Deep Research Loop Count
```yaml
Scenario: T-009
Name: Research Iteration Tracking
Cookbook: pocketflow-deep-research
Severity: High

Initial State:
  shared.topic: "Quantum computing in 2025"
  shared.notes: [5 research notes from iteration 0]
  shared.loop_count: 0
  shared.feedback: ""
  
Execution Pattern:
  - Iteration 0 complete
  - Synthesizer finds gaps, returns "research"
  - loop_count becomes 1
  - Planner creates new queries
  - Researcher executes batch
  - [INJECT CRASH] During 2nd web search
  
Failure Injection:
  Method: Network drop during search_web()
  Timing: Mid-batch execution
  
Expected Behavior:
  - Preserve loop_count = 1
  - Preserve notes from iteration 0
  - Retry current batch
  - Max loops still enforced (2)
  
Success Criteria:
  - loop_count never resets
  - Iteration 0 notes preserved
  - Max 2 total iterations
  
State Validation:
  Check: shared.loop_count monotonically increases
  Check: len(shared.notes) >= 5 (iteration 0 results)
```

#### Test 2.5.2: Notes Accumulation Consistency
```yaml
Scenario: T-010
Name: Research Notes Append-Only
Cookbook: pocketflow-deep-research
Severity: High

Initial State:
  shared.notes: [note1, note2, note3]
  
Execution Pattern:
  - BatchNode produces 3 new notes
  - post() appends to shared.notes
  - [INJECT CRASH] After extend(), before completion
  
Failure Injection:
  Method: Panic in post() after notes.extend()
  Timing: Immediately after list extension
  
Expected Behavior:
  - Notes partially extended
  - On recovery, detect partial state
  - Re-execute batch if needed
  - No duplicate notes
  
Success Criteria:
  - All 6 notes present (3 old + 3 new)
  - No duplicates
  - Notes in chronological order
  
State Validation:
  Check: len(shared.notes) == 6
  Check: First 3 notes match original
```

### 2.6 External API Failures During Retries

#### Test 2.6.1: Search API Rate Limit
```yaml
Scenario: T-011
Name: Rate Limit Backoff
Cookbook: pocketflow-agent, pocketflow-deep-research
Severity: High

Initial State:
  shared.search_query: "Nobel Prize 2024"
  shared.context: ""
  
Execution Pattern:
  - SearchWeb node executes
  - API returns 429 (rate limited)
  - Should retry with backoff
  
Failure Injection:
  Method: Mock search_web to return 429
  Timing: First 2 calls
  
Expected Behavior:
  - Retry 1 after 1s
  - Retry 2 after 2s
  - Retry 3 after 4s
  - Then fail or use fallback
  
Success Criteria:
  - Exactly 3 attempts
  - Delays: 1s, 2s, 4s
  - Total time: ~7s + execution time
  
State Validation:
  Check: search_web called 3 times
  Check: Delays match exponential backoff
```

#### Test 2.6.2: Intermittent Network Failure
```yaml
Scenario: T-012
Name: Network Partition Recovery
Cookbook: All external tool cookbooks
Severity: Critical

Initial State:
  shared: <any state requiring API call>
  
Execution Pattern:
  - API call in progress
  - Network partition for 10s
  - Then restored
  
Failure Injection:
  Method: Network namespace isolation (iptables)
  Timing: 5s into API call
  Duration: 10s
  
Expected Behavior:
  - Detect network failure
  - Retry when network restored
  - Timeout if partition exceeds limit
  
Success Criteria:
  - Recovery after network restore
  - No indefinite hangs
  - State consistent
  
State Validation:
  Check: API eventually succeeds or times out
  Check: No orphaned connections
```

### 2.7 Checkpoint Corruption Scenarios

#### Test 2.7.1: Partial Checkpoint Write
```yaml
Scenario: T-013
Name: Dapr State Corruption
Cookbook: All workflows
Severity: Critical

Initial State:
  shared: <complex state with nested structures>
  
Execution Pattern:
  - Checkpoint triggered
  - Partial write to Dapr (network interruption)
  - Process crashes
  
Failure Injection:
  Method: Network drop during Dapr state store write
  Timing: After 50% of data written
  
Expected Behavior:
  - Detect corrupted checkpoint on recovery
  - Fall back to previous valid checkpoint
  - Log corruption event
  
Success Criteria:
  - Recovery from last valid checkpoint
  - No partial state exposure
  - Corruption logged
  
State Validation:
  Check: State version/checksum validation
  Check: Rollback to previous version
```

#### Test 2.7.2: Concurrent Checkpoint Conflicts
```yaml
Scenario: T-014
Name: Distributed Checkpoint Race
Cookbook: Multi-agent workflows
Severity: High

Initial State:
  shared: <shared state accessed by multiple agents>
  
Execution Pattern:
  - Agent A checkpoints state
  - Agent B checkpoints state simultaneously
  - Conflict detected by Dapr
  
Failure Injection:
  Method: Concurrent checkpoint requests
  Timing: Same millisecond
  
Expected Behavior:
  - Dapr ETag conflict detection
  - Retry with updated state
  - Merge strategy if appropriate
  
Success Criteria:
  - No lost updates
  - Conflict resolved
  - State eventually consistent
  
State Validation:
  Check: ETag versioning
  Check: Last-write-wins or merge success
```

### 2.8 Concurrent Workflow Interference

#### Test 2.8.1: Shared State Contention
```yaml
Scenario: T-015
Name: Multi-Workflow State Collision
Cookbook: All shared-state workflows
Severity: High

Initial State:
  shared.stats: {total_texts: 100, total_words: 5000}
  
Execution Pattern:
  - Workflow A reads stats (100, 5000)
  - Workflow B reads stats (100, 5000)
  - Workflow A updates (101, 5050)
  - Workflow B updates (101, 5020) - based on stale read
  
Failure Injection:
  Method: Concurrent execution without locks
  Timing: Simultaneous reads
  
Expected Behavior:
  - Detect stale update
  - Retry with fresh state
  - Or use optimistic locking
  
Success Criteria:
  - Final state consistent
  - No update loss
  - No incorrect aggregation
  
State Validation:
  Check: Final stats == sum of all operations
  Check: No negative counts
```

#### Test 2.8.2: Actor State Isolation
```yaml
Scenario: T-016
Name: Cross-Workflow Data Leak
Cookbook: Actor-based workflows
Severity: Critical

Initial State:
  Workflow A: shared.user_id = "user123"
  Workflow B: shared.user_id = "user456"
  
Execution Pattern:
  - Workflows run concurrently
  - Check for cross-contamination
  
Failure Injection:
  Method: Stress test with 100 concurrent workflows
  Timing: Maximum concurrency
  
Expected Behavior:
  - Each workflow isolated
  - No state leakage
  - Dapr actor boundaries enforced
  
Success Criteria:
  - 100% isolation
  - No cross-workflow data visible
  - Correct user data per workflow
  
State Validation:
  Check: Each workflow's shared contains only its data
  Check: No global state pollution
```

### 2.9 Memory Pressure Scenarios

#### Test 2.9.1: Large Context Window
```yaml
Scenario: T-017
Name: OOM During Large Conversation
Cookbook: pocketflow-chat-memory
Severity: High

Initial State:
  shared.messages: [1000 conversation pairs]
  shared.vector_index: [500 archived conversations]
  
Execution Pattern:
  - Load all state into memory
  - Process next message
  - Memory usage grows
  
Failure Injection:
  Method: Memory limit cgroup (100MB)
  Timing: During vector search
  
Expected Behavior:
  - Graceful OOM handling
  - Checkpoint before crash
  - Resume with memory optimization
  
Success Criteria:
  - No data loss
  - Recovery possible
  - Memory usage bounded
  
State Validation:
  Check: Recovery succeeds
  Check: State truncation if needed (configurable)
```

#### Test 2.9.2: Embedding Memory Exhaustion
```yaml
Scenario: T-018
Name: Vector Index Memory Limit
Cookbook: pocketflow-rag, pocketflow-chat-memory
Severity: Medium

Initial State:
  shared.embeddings: [10000 vectors, 768-dim each]
  ~30MB of embedding data
  
Execution Pattern:
  - Add more embeddings
  - Approach memory limit
  
Failure Injection:
  Method: Rapid embedding generation
  Timing: Continuous load
  
Expected Behavior:
  - Externalize large vectors to blob storage
  - Keep only index in memory
  - Lazy load embeddings
  
Success Criteria:
  - Memory usage bounded
  - Retrieval performance maintained
  - No OOM crashes
  
State Validation:
  Check: Memory < 100MB
  Check: Retrieval time < 100ms
```

### 2.10 Network Partition During Distributed Execution

#### Test 2.10.1: Dapr Sidecar Disconnect
```yaml
Scenario: T-019
Name: Sidecar Partition Recovery
Cookbook: All Dapr-based workflows
Severity: Critical

Initial State:
  Workflow running with Dapr sidecar
  shared: <any state>
  
Execution Pattern:
  - Normal execution
  - Dapr sidecar becomes unreachable
  - Then reconnects
  
Failure Injection:
  Method: Docker network disconnect sidecar
  Timing: 10s into execution
  Duration: 30s
  
Expected Behavior:
  - Detect sidecar unavailability
  - Queue state operations
  - Retry on reconnect
  - Or fail fast after timeout
  
Success Criteria:
  - Recovery after reconnect
  - No state operations lost
  - Timeout enforced (60s)
  
State Validation:
  Check: All state persisted after recovery
  Check: No hanging operations
```

#### Test 2.10.2: Actor Placement Failure
```yaml
Scenario: T-020
Name: Actor Host Migration
Cookbook: Actor-based workflows
Severity: High

Initial State:
  Workflow actor running on Host A
  
Execution Pattern:
  - Actor processing
  - Host A fails
  - Actor placement moves to Host B
  
Failure Injection:
  Method: Kill Host A process
  Timing: Mid-execution
  
Expected Behavior:
  - Actor state preserved in Dapr
  - Actor reactivated on Host B
  - Resume from checkpoint
  
Success Criteria:
  - Actor state intact
  - Execution resumes
  - No duplicate processing
  
State Validation:
  Check: Actor state version preserved
  Check: Idempotency of resumed operations
```

---

## 3. Recovery Validation Matrix

### 3.1 State Preservation Requirements

| Scenario | Must Preserve | Can Lose | Validation Method |
|----------|--------------|----------|-------------------|
| T-001 Chat Input | User messages, conversation history | LLM responses (regeneratable) | Message sequence checksum |
| T-002 Vector Index | vector_items, index consistency | Orphaned embeddings (re-embeddable) | Items vs index count match |
| T-003 Multi-Agent | past_guesses, game state | Queue contents (restartable) | Game state checksum |
| T-004 Supervisor | retry_count, context | N/A | Retry count <= 3 |
| T-005 Batch | completed items, output files | In-progress item (reprocess) | File count check |
| T-006 Parallel | successful worker results | Failed worker (retry) | Result completeness |
| T-007 Self-Healing | attempts history | N/A | Attempts list integrity |
| T-008 Error Feedback | all errors | N/A | Error history count |
| T-009 Deep Research | loop_count, notes | In-progress search (retry) | Notes append-only |
| T-010 Notes | accumulated notes | N/A | Note count monotonic |
| T-011 API Retry | request parameters | N/A | Retry count tracking |
| T-012 Network | connection state | N/A | Reconnection success |
| T-013 Checkpoint | previous valid checkpoint | Corrupted checkpoint | Checksum validation |
| T-014 Concurrent | all updates | N/A | Final state consistency |
| T-015 Contention | stat totals | N/A | Aggregate correctness |
| T-016 Isolation | per-workflow data | N/A | No cross-contamination |
| T-017 Memory | critical state | Non-critical cache | Recovery success |
| T-018 Embeddings | index structure | Cached vectors | Retrieval works |
| T-019 Dapr | all state operations | N/A | State after recovery |
| T-020 Actor | actor state | N/A | State version match |

### 3.2 Data Loss Tolerance

| Workflow Type | Acceptable Loss | Critical Data |
|--------------|-----------------|---------------|
| Chat | Last user input only | Message history, memory vectors |
| Agent | Current search results | Accumulated context, action plan |
| Multi-Agent | In-flight messages | Game state, coordination tokens |
| Batch | In-progress item | Completed items, partial results |
| Research | Current iteration | Accumulated notes, loop count |
| Self-Healing | N/A | Attempt history, error feedback |
| HITL | N/A | Pending approvals, feedback |
| RAG | N/A | Embeddings, index, documents |
| Async | In-flight async result | Completed async results |

### 3.3 Edge Cases and Failure Modes

#### Edge Case 1: Zero-Downtime Recovery
- **Scenario**: Hot standby takeover
- **Trigger**: Primary node failure
- **Behavior**: Secondary resumes from last checkpoint
- **Validation**: < 1s failover, no state divergence

#### Edge Case 2: Cascading Failure
- **Scenario**: Failure triggers secondary failures
- **Trigger**: Database connection pool exhaustion
- **Behavior**: Circuit breaker opens, fallback mode
- **Validation**: Graceful degradation, no cascade

#### Edge Case 3: Split-Brain
- **Scenario**: Network partition creates divergent states
- **Trigger**: Inter-datacenter link failure
- **Behavior**: Detect partition, choose consensus
- **Validation**: Single authoritative state chosen

#### Edge Case 4: Clock Skew
- **Scenario**: Distributed nodes have different clocks
- **Trigger**: NTP sync failure
- **Behavior**: Logical clocks (vector clocks) used
- **Validation**: Causality preserved

#### Edge Case 5: Resource Exhaustion
- **Scenario**: Disk full during checkpoint
- **Trigger**: Checkpoint exceeds available space
- **Behavior**: Alert, compress, or externalize
- **Validation**: Checkpoint succeeds or graceful fail

---

## 4. Implementation Notes

### 4.1 Crash Simulation Methods

#### Method 1: Process Kill
```bash
# Send SIGKILL to simulate sudden crash
kill -9 <workflow_pid>

# In tests:
import os
import signal
os.kill(os.getpid(), signal.SIGKILL)
```

#### Method 2: Panic Injection
```python
# Add test hook in node
if os.environ.get("INJECT_CRASH") == self.node_id:
    raise Exception("Injected crash")
```

#### Method 3: Resource Limits
```bash
# Limit memory to trigger OOM
systemd-run --scope -p MemoryMax=50M python workflow.py

# Limit CPU to trigger timeouts
systemd-run --scope -p CPUQuota=10% python workflow.py
```

#### Method 4: Network Partition
```bash
# Using tc (traffic control) to drop packets
sudo tc qdisc add dev eth0 root netem loss 100%
sleep 10
sudo tc qdisc del dev eth0 root

# Using iptables to block Dapr sidecar
sudo iptables -A OUTPUT -p tcp --dport 50001 -j DROP
sleep 30
sudo iptables -D OUTPUT -p tcp --dport 50001 -j DROP
```

#### Method 5: Container/K8s Chaos
```yaml
# Chaos Mesh experiment
apiVersion: chaos-mesh.org/v1alpha1
kind: PodChaos
metadata:
  name: workflow-pod-failure
spec:
  action: pod-failure
  mode: one
  selector:
    labelSelectors:
      app: pocketflow-workflow
  duration: 30s
```

### 4.2 State Consistency Verification

#### Verification 1: Checksum Validation
```python
def verify_state_consistency(shared):
    checksum = hashlib.sha256(
        json.dumps(shared, sort_keys=True).encode()
    ).hexdigest()
    return checksum
```

#### Verification 2: Schema Validation
```python
def validate_state_schema(shared, expected_schema):
    # Validate required fields exist
    for field in expected_schema.required:
        assert field in shared, f"Missing required field: {field}"
    
    # Validate field types
    for field, field_type in expected_schema.types.items():
        assert isinstance(shared[field], field_type)
```

#### Verification 3: Invariant Checking
```python
def check_invariants(shared):
    # For chat-memory
    assert len(shared.get("vector_items", [])) == \
           shared.get("vector_index", {}).ntotal
    
    # For deep-research
    assert shared.get("loop_count", 0) <= 2
    
    # For supervisor
    assert shared.get("retry_count", 0) <= 3
```

### 4.3 Testing Tools and Frameworks

#### Tool 1: Chaos Mesh
```yaml
# Kubernetes chaos engineering
apiVersion: chaos-mesh.org/v1alpha1
kind: Workflow
metadata:
  name: pocketflow-durability-test
spec:
  templates:
    - name: pod-failure
      templateType: PodChaos
      podChaos:
        action: pod-failure
        mode: one
```

#### Tool 2: Toxiproxy
```python
# Network failure simulation
from toxiproxy import Toxiproxy

proxy = Toxiproxy()
proxy.create_proxy("dapr", "localhost:50001")
proxy.toggle_proxy("dapr", False)  # Disconnect
proxy.toggle_proxy("dapr", True)   # Reconnect
```

#### Tool 3:pytest with Fault Injection
```python
import pytest
import fault_injection

@pytest.mark.fault_injection(
    target="nodes.SearchWeb.exec",
    fault="network_timeout",
    timing="50%"
)
def test_search_recovery():
    # Test code
```

#### Tool 4: Dapr Resiliency Testing
```yaml
# resiliency.yaml
apiVersion: dapr.io/v1alpha1
kind: Resiliency
metadata:
  name: pocketflow-resiliency
spec:
  policies:
    retries:
      pubsubRetry:
        policy: exponential
        maxRetries: 3
        duration: 1s
```

### 4.4 Metrics to Collect

#### Metric 1: Recovery Time
```python
recovery_time = time.time() - crash_time
assert recovery_time < 5.0, f"Recovery too slow: {recovery_time}s"
```

#### Metric 2: Data Loss Count
```python
lost_items = count_expected - count_actual
assert lost_items <= max_acceptable, f"Too much data loss: {lost_items}"
```

#### Metric 3: Checkpoint Latency
```python
checkpoint_duration = checkpoint_end - checkpoint_start
assert checkpoint_duration < 100, f"Checkpoint too slow: {checkpoint_duration}ms"
```

#### Metric 4: State Size
```python
state_size = len(json.dumps(shared).encode())
assert state_size < 1000000, f"State too large: {state_size} bytes"
```

#### Metric 5: Retry Count
```python
assert retry_count <= max_retries, f"Too many retries: {retry_count}"
```

---

## 5. Test Execution Plan

### Phase 1: Unit Durability Tests (Week 1-2)
- T-001 to T-005: Core state persistence
- T-013, T-014: Checkpoint integrity
- Run: 100 iterations each

### Phase 2: Integration Tests (Week 3-4)
- T-006 to T-012: Workflow pattern tests
- T-015 to T-018: Concurrent and memory tests
- Run: 50 iterations each

### Phase 3: Chaos Tests (Week 5-6)
- T-019, T-020: Distributed system tests
- All scenarios with random fault injection
- Run: 24-hour continuous test

### Phase 4: Load Tests (Week 7)
- 1000 concurrent workflows
- Memory pressure scenarios
- Performance validation

### Phase 5: Edge Case Validation (Week 8)
- Zero-downtime recovery
- Cascading failure prevention
- Split-brain resolution

---

## 6. Success Criteria Summary

| Category | Metric | Target |
|----------|--------|--------|
| Recovery Time | Average time to resume | < 5 seconds |
| Data Loss | Acceptable loss per crash | 0 critical items |
| Consistency | State validation pass rate | 100% |
| Availability | Uptime during faults | > 99.9% |
| Performance | Throughput under faults | > 80% of normal |
| Correctness | Test pass rate | > 99% |

---

## 7. Appendix: Cookbook Analysis Summary

### State Patterns Identified

| Cookbook | Key State | Failure Mode | Impact |
|----------|-----------|--------------|--------|
| pocketflow-chat | messages[] | Crash mid-response | Loss of user input |
| pocketflow-chat-memory | vector_index, vector_items | Desync | Retrieval errors |
| pocketflow-agent | context, search_query | Search failure | Incomplete research |
| pocketflow-multi-agent | queues, past_guesses | Agent crash | Game stall |
| pocketflow-supervisor | answer, retry_count | Loop | Infinite retry |
| pocketflow-batch | translations{} | Partial completion | Missing outputs |
| pocketflow-deep-research | notes[], loop_count | Iteration loss | Incomplete report |
| pocketflow-self-healing | attempts[] | History loss | Poor retry quality |
| pocketflow-fastapi-hitl | review_event, feedback | Timeout | Indefinite wait |
| pocketflow-rag | embeddings, index | Corruption | Wrong retrieval |

### Checkpointing Requirements

| Pattern | Checkpoint Frequency | Critical State |
|---------|---------------------|----------------|
| Self-loop | Every iteration | Loop counter, accumulator |
| Batch | Every item | Completed items list |
| Async | After each await | Async result |
| HITL | Before wait | Pending state |
| Multi-agent | After each message | Coordination state |
| Research | Every search iteration | Notes, loop count |

---

**Document Version**: 1.0  
**Based on Cookbook Analysis**: 59 PocketFlow cookbook examples  
**Test Scenarios Defined**: 20 detailed, 50+ variations  
**Coverage**: Chat, Agent, Multi-Agent, Batch, Research, Self-Healing, HITL, RAG, Async, Tool patterns
