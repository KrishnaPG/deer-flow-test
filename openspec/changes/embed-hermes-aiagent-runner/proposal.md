# Embed Hermes In Our ACP Client

## Summary
Provide a Rust-managed ACP integration layer, referred to as `our ACP Client`, that launches Hermes ACP as a subprocess over JSON-RPC stdio, captures every observable ACP frame and client-issued command envelope, exposes a live response stream, and durably publishes the raw protocol traffic to Redb topic `hermes.l0_drop` before any ingestion or Berg10 interpretation occurs.

## Why
- Keeps the first write path extremely fast and durable: Redb is the primary L0 landing zone, not an ETL layer.
- Preserves enough raw fidelity to reconstruct full Hermes sessions for replay, audit, and future real-time agentic chat UI.
- Avoids modifying `3rdParty/hermes-agent`; integration happens entirely through the ACP boundary.
- Makes sequencing and durability our responsibility instead of relying on internal Hermes storage semantics.
- Positions ACP as the primary control and session boundary not only for Hermes, but for future non-Hermes agents as well.
- Cleanly separates ACP Client concerns (capture, order, durability, replay metadata, live stream fanout) from ingestion concerns (classification, Berg10 mapping, normalization, projection).

## Scope
In-scope:
- `our ACP Client` that launches and manages Hermes ACP over JSON-RPC stdio.
- Use of the Rust `agent-client-protocol` crate where appropriate for ACP protocol typing and request/response/session-update handling.
- Context-sensitive, strongly typed runtime naming in Rust, with examples such as:
  - `ChatSessionId`
  - `ChatRunId`
  - `ChatThreadId`
  - `AcpCapturedProtocolEvent`
  - `AcpResponseStreamEvent`
- Redb-first raw capture to topic `hermes.l0_drop`, behind a broker-agnostic durable event publication boundary so another broker can replace Redb later.
- Preservation of every observable boundary event `our ACP Client` can capture, including:
  - client-issued commands such as prompt, cancel, session create, session load, and resume
  - ACP requests, responses, and notifications observed on the subprocess boundary
  - subprocess lifecycle events that materially affect reconstruction (spawned, exited, crashed, restarted, timeout)
- Stable replay metadata on each published envelope:
  - `session_id`
  - `run_id`
  - `seq`
  - `timestamp`
  - optional `thread_id`, `message_id`, `tool_call_id`, and similar fields when observable
- A sequencing model owned by `our ACP Client`, scoped monotonically within `session_id`.
- A process model that supports multiple sessions per ACP subprocess, while defaulting to one subprocess per session for initial isolation.
- A live response stream exposed now from ACP-observed events:
  - live subprocess/session/run status
  - live tool activity
  - live coarse thought/status events currently available from Hermes ACP
  - final/coarse assistant text at the granularity Hermes ACP currently exposes
- Redb-backed replay as the initial durable recovery source, with optional in-memory replay tail as an optimization and a broker abstraction that keeps future broker replacement possible.
- Use of `BERG10_BASE_DIR` for runner-owned paths under `<base_dir>/runners/hermes/` when local state or diagnostics are needed.

Out-of-scope (this change):
- Berg10 field mapping, `data_hierarchy` classification, normalization, and any other domain interpretation.
- Ingestion from `hermes.l0_drop` into `crates/berg10/storage-vfs/` and `crates/berg10/storage-catalog/`.
- Stable public HTTP/WebSocket client APIs.
- True incremental assistant-text deltas when Hermes ACP does not expose them.
- Hermes-specific hybrid streaming integration such as a future TUI-gateway-backed stream adapter.
- File fallback as a primary path; this change is explicitly Redb-first.
- Any source changes under `3rdParty/`.

## Acceptance Criteria
- `our ACP Client` launches Hermes ACP as a subprocess and communicates using JSON-RPC over stdio.
- `our ACP Client` uses the Rust `agent-client-protocol` crate where appropriate for ACP wire typing and session-update handling.
- `our ACP Client` can create or resume Hermes sessions and execute prompts as runs within those sessions.
- Every observable boundary event `our ACP Client` can capture is published to Redb topic `hermes.l0_drop`.
- Published envelopes preserve raw ACP payloads and client command payloads without Berg10 mapping or semantic reinterpretation.
- Each published envelope includes stable replay metadata with `session_id`, `run_id`, `seq`, and `timestamp`, plus optional identifiers when available.
- `session_id` is used as the Redb message key.
- `seq` is assigned by `our ACP Client` and is monotonic within a session.
- The implementation supports multiple sessions per subprocess at the abstraction level, while the default runtime policy uses one subprocess per session.
- `our ACP Client` exposes a live response stream immediately, even though current Hermes ACP granularity only supports live status/tool events and final/coarse assistant text.
- `our ACP Client` does not invent synthetic assistant text deltas when Hermes ACP only exposes coarse/final text.
- Redb producer settings provide high durability and ordering safety (`acks=all`, idempotence enabled, retries with backoff, ordering-safe in-flight configuration).
- Redb remains the durable source of raw truth until downstream ingestion succeeds; client-side storage interpretation is not performed.
- The design keeps `thread_id` optional so future manual session branching can attach it when a real source exists.

## Risks & Mitigations
- **Subprocess Instability:** Hermes ACP can hang, crash, or emit malformed frames.
  - *Mitigation:* Treat stdout as protocol-only, stderr as diagnostics, add heartbeat/timeouts, emit lifecycle envelopes, and allow supervised restart.
- **Current ACP Streaming Limits:** Hermes ACP currently exposes live status/tool activity and final/coarse assistant text, but not true incremental assistant text deltas.
  - *Mitigation:* Expose the live stream now with honest fidelity, and defer richer Hermes-specific text streaming to a future hybrid stream adapter.
- **Reconstruction Gaps:** Missing prompt/cancel or protocol frames would weaken replay fidelity.
  - *Mitigation:* Persist every observable client-side command and every observable ACP frame, not only assistant-visible events.
- **Ordering Drift:** Multiple runs inside one session can interleave unexpectedly.
  - *Mitigation:* Use `session_id` as the partition key and assign session-scoped monotonic `seq` in `our ACP Client`.
- **Storage Growth:** Full raw capture increases broker retention cost.
  - *Mitigation:* Allow compression, keep retention operationally long-lived, and defer deletion to successful downstream ingestion policy.
