# Plan: Rewrite REPL to use DeerFlowClient properly

## Context

The web UI uses `langgraph_sdk` to talk to a LangGraph Server (HTTP), but `DeerFlowClient` was built as an **embedded alternative** that runs the same agent in-process. After thorough analysis, both paths use the identical `create_agent()` call with the same middleware chain, tools, state schema (`ThreadState`), and checkpointer. The key difference is execution (sync in-process vs async HTTP), not behavior.

**Conclusion:** We do NOT need `langgraph_sdk` or a running server. `DeerFlowClient.stream()` with proper `thread_id` and checkpointer management gives us the same behavior.

## Root Cause of Previous Issues

1. **Monkey-patching `ClarificationMiddleware`** — This made `ask_clarification` execute as a regular tool (with no tool function registered), causing the agent to skip clarification and jump straight to `present_files` for non-existent files.
2. **Double-dedup** — Both `DeerFlowClient.stream()` and our REPL had `seen_ids` sets, causing messages to be silently dropped.

## Approach

Rewrite `test/test_embedded_client.py` as a clean REPL that:

1. Creates `DeerFlowClient` with no monkey-patches (leave `ClarificationMiddleware` intact)
2. Uses a single `thread_id` for the conversation (checkpointer preserves multi-turn state)
3. Handles `ClarificationMiddleware` interrupts by:
   - Detecting `ask_clarification` in `tool_calls` events
   - Showing the question and options to the user
   - Getting user input
   - Calling `client.stream()` again with the response on the same `thread_id`
   - The agent sees the full history (tool call + ToolMessage + user's response) and proceeds
4. Moves the `seen_ids` deduplication to the REPL level only (rely on the REPL's `seen_msg_ids`, ignore `DeerFlowClient.stream()`'s internal dedup for our purposes — or better, just process all events and dedup at display time)
5. Always verbose (show all tool calls and results)
6. Auto-open generated images via `subprocess.run(["open", ...])`

## Implementation Details

### File: `test/test_embedded_client.py`

**Imports & Setup** (no monkey-patching):
- `DeerFlowClient` creation with `ModelConfig` / `SandboxConfig` (same as current)
- Default to `InMemorySaver` checkpointer (auto-created by `DeerFlowClient._ensure_agent`)
- Global `thread_id = str(uuid.uuid4())`

**Event Processing — `process_stream()`**:
- Iterate `client.stream(message, thread_id=thread_id)`
- Dedup by event type + ID at the REPL level only
- For `messages-tuple` events with `tool_calls`:
  - Print each tool call with formatted args (bash command, write_file path, etc.)
  - If `ask_clarification` is in tool_calls, extract `question` and `options`, set a flag
- For `messages-tuple` events with `type="tool"`:
  - Always show sandbox tool results (bash, write_file, read_file, ls, str_replace) with `←`/`✗` markers
  - Skip showing the `ask_clarification` ToolMessage (it's just the formatted question — we already showed it)
- For `values` events:
  - Resolve artifact virtual paths to host paths
  - Auto-open images via `subprocess.run(["open", ...])`
- For `end` events:
  - Show token usage
- Return whether `ask_clarification` was called

**Main Loop**:
```
while True:
    user_input = input("You: ")
    # handle /quit, /new, /verbose commands
    while True:
        asked_clarification = process_stream(user_input)
        if not asked_clarification:
            break
        # Show clarification question, get user response
        user_input = input("You: ")
        # Continue the loop — next stream() call on same thread_id
        # sees the ask_clarification tool message + user's response
```

The inner `while True` handles multiple consecutive clarification rounds (if the agent asks 2+ questions).

### Dedup Strategy

- `DeerFlowClient.stream()` has its own `seen_ids` scoped per `stream()` call — this dedups messages WITHIN a single call (state snapshots re-emit old messages).
- Our REPL has a GLOBAL `seen_msg_ids` across all `stream()` calls — this dedups messages ACROSS turns (messages from previous turns get re-emitted when `stream_mode="values"` re-yields the full state).
- Both are needed and complementary. No conflict.

### Edge Cases

- **Clarification loop**: If the agent keeps asking clarification forever, the user can type `/new` to start fresh or Ctrl+C to exit
- **Agent asks clarification on first message**: System prompt says to clarify — this is expected behavior, not a bug
- **File not found**: When `present_files` references non-existent files, the agent's system prompt instruction to clarify first should prevent this. If it still happens, show the error.
- **Multiple tool calls**: The agent can emit multiple tool calls in one AIMessage. We show all of them. If one is `ask_clarification`, it will be the last one (midware returns `Command(goto=END)`).

## Files to Modify

- `test/test_embedded_client.py` — Complete rewrite

## Files NOT to Modify

- All files in `3rdParty/deer-flow/` — upstream, read-only
- `test/test_debug_run.py` — leave as-is (non-interactive test)

## Verification

1. Run `source .venv/bin/activate && python test/test_embedded_client.py`
2. Test basic chat: "Hello, how are you?"
3. Test image generation: "Create an image of a sunset"
   - Should ask clarification → respond → agent runs image generation skill → auto-opens the result
4. Test sandbox tools: "Write a hello world Python script"
   - Should show `bash` command, `write_file` path/content, result markers
5. Test `/new` command to reset conversation
