# Developer Troubleshooting & Diagnostics Guide

This document is designed for developers, operators, and AI support agents to diagnose and troubleshoot issues within the Storage-Native distributed system. 

It provides concrete query strategies and identifies which observability tools to use to trace data across the multi-plane storage architecture.

## Tooling Quick Reference

*   **OpenTelemetry / SigNoz / Tempo:** Use for distributed tracing, finding where a request failed (UI -> State Server -> Temporal -> Dapr -> Worker).
*   **Langfuse:** Use for LLM-specific telemetry (latency, token count, cost, exact prompts/responses).
*   **NATS JetStream CLI / Dashboard:** Use to inspect queue lag, dead-letter queues (DLQ), and verify if an event was actually emitted.
*   **ClickHouse:** Use to query the canonical source of truth for events, intents, and chunks.
*   **Temporal Web UI:** Use to inspect long-running asynchronous HOTL workflows, retries, and blocked tasks.

---

## Troubleshooting Q&A

### Q: Why did a data point fail to process from L0 (As-Is) to L3 (Chunks)?
**Symptoms:** The original file exists, but chunks/embeddings are not appearing in search or ClickHouse.
**Diagnostic Steps:**
1.  **Check OpenTelemetry Trace:** Search by the `artifact_id` or `trace_id`. Identify if the `ChunkingWorker` ever received the task.
2.  **Check Temporal Workflows:** Look for failed or continuously retrying workflows related to this `mission_id`. Did the worker run out of memory (OOM)?
3.  **Check NATS DLQ:** Did the `ArtifactCreated` event for the L0 file end up in a Dead Letter Queue due to a malformed payload, preventing the chunking pipeline from triggering?
4.  **Validate S3 Permissions:** Ensure the `ChunkingWorker` IAM role has `s3:GetObject` on the specific `s3://.../as_is/L0/` path.

### Q: What exact response did an agent generate for a specific prompt?
**Symptoms:** Need to audit what the AI said, or reproduce a hallucination.
**Diagnostic Steps:**
1.  **Query ClickHouse (Canonical Truth):** Query the `events` or `as_is` tables filtering by `mission_id` and `agent_id`. This provides the finalized, hashed artifact payload.
    ```sql
    SELECT payload FROM as_is_plane WHERE mission_id = 'm-123' AND agent_id = 'agent-456' ORDER BY timestamp DESC LIMIT 1;
    ```
2.  **Check Langfuse (LLM Context):** If you need to see the exact prompt that *caused* the response, use Langfuse. Search by the `trace_id` attached to the ClickHouse event to see the full LLM inputs, system prompts, and token usage.

### Q: Why are text tokens generating slowly in the UI during a live chat (HITL)?
**Symptoms:** The user is speaking to the agent, but the text is lagging or appearing in large chunks instead of streaming smoothly.
**Diagnostic Steps:**
1.  **Isolate the LLM:** Check Langfuse for the specific agent's "Time to First Token" (TTFT) and "Tokens Per Second" (TPS). If the LLM provider is lagging, the issue is upstream.
2.  **Check the "Stream-Tee":** If the LLM is fast, check the Storage Service metrics. Are `StreamDelta` events being published to NATS JetStream with low latency (<5ms)? 
3.  **Check NATS Consumer Lag:** Ensure the State Server replica subscribed to the `mission_id` topic does not have consumer lag.
4.  **Check State Server Websockets:** Monitor the State Server drop rates. Are websockets to the UI saturated or dropping frames due to network conditions?

### Q: Why is semantic search not finding a newly uploaded document?
**Symptoms:** The document was ingested, but Vector/RAG searches return 0 results.
**Diagnostic Steps:**
1.  **Verify L1-L4 Chunks in ClickHouse:** First, confirm the text chunks exist. If not, refer to the "L0 to L3" troubleshooting step above.
2.  **Check NATS "Embeddings" Queue:** Look at the queue connecting the Storage Service to the Milvus Sync Workers. Is there a massive backlog?
3.  **Check Sync Worker Logs:** Are the Sync Workers being rate-limited (`429 Too Many Requests`) by the Embedding API provider (e.g., OpenAI, Cohere)?
4.  **Check Milvus Index Status:** Ensure Milvus has completed building the index for the newly inserted vectors. Query Milvus directly using the `chunk_hash` to see if the vector exists physically but isn't searchable yet.

### Q: Why is the State Server rejecting all UI reads and writes with 403 Forbidden or timeouts?
**Coupling Point:** State Server <-> ABAC (Casbin/Postgres)
**Symptoms:** All UI components load blank. Operator commands fail instantly.
**Diagnostic Steps:**
1.  **Check Postgres Health:** The State Server relies on Postgres to evaluate Casbin ABAC policies. If Postgres is down, connection-starved, or responding slowly, the State Server fails closed to prevent unauthorized access.
2.  **Trace the Intent:** Use OpenTelemetry to trace a specific failed request from the UI. Does the trace end abruptly at the `validate_abac` span in the State Server?
3.  **Verify JWT/SPIFFE IDs:** Ensure the UI's JWT hasn't expired, and the Dapr sidecar injecting SPIFFE IDs into service-to-service calls is healthy.

### Q: Why are AI agents failing to invoke other services, with "connection refused" or generic network timeouts?
**Coupling Point:** Temporal <-> Worker <-> Dapr Sidecar
**Symptoms:** Temporal workflows show "Activity Timeout" or "Activity Failed". Agent logs show connection errors when calling another internal service API.
**Diagnostic Steps:**
1.  **Check Dapr Sidecar Status:** Dapr handles service invocation and mTLS. If the Dapr sidecar for a given pod failed to start, the agent's outbound calls via `localhost:3500` will fail.
2.  **Check mTLS Certificate Expiry:** If mTLS is strictly enforced, expired certificates in the service mesh will cause Dapr to reject connections between agents and the State Server.
3.  **Inspect Temporal Activity Retries:** Use the Temporal Web UI. Are the activities stuck in an exponential backoff retry loop due to `DaprServiceInvocationError`?

### Q: Why is a completed WebRTC audio call missing from the L0 "As-Is" historical view?
**Coupling Point:** LiveKit Egress <-> S3 <-> Storage Service
**Symptoms:** An interactive HITL session finished, but the final composite recording is missing from ClickHouse and cannot be replayed.
**Diagnostic Steps:**
1.  **Check LiveKit Egress Status:** Did the Egress pod crash during the call? It should have written the raw media to an ephemeral S3 landing zone.
2.  **Check S3 Landing Zone:** Directly inspect the S3 bucket (`s3://.../landing/`). Are there orphaned `.webm` or `.ogg` files that were never processed?
3.  **Check Storage Service Promotion Logs:** The Storage Service listens for the Egress completion event to hash the file and move it to the canonical L0 path. Check NATS for the LiveKit completion event, and ensure the Storage Service didn't crash while copying the file.

### Q: Why is the UI suddenly lagging severely for all users, or failing to reconnect to the websocket?
**Coupling Point:** State Server <-> Redis (Hot Lane / Leadership Lease)
**Symptoms:** The "Hot Window" is empty or stale. Active sessions drop unexpectedly and cannot be re-established.
**Diagnostic Steps:**
1.  **Check Redis Connectivity:** The State Server relies on Redis to hold leadership leases for active missions and store the Hot Window. If Redis is partitioned or OOM, State Servers will lose leadership and drop connections.
2.  **Check Replica Failover Logs:** Look at the State Server pod logs. Are they constantly fighting for leadership (flapping) because Redis is slow to respond?
3.  **Verify NATS Subscription Limits:** When a State Server reconnects after losing leadership, it rehydrates from NATS. If NATS is rate-limiting the State Server's bulk read requests, the Hot Cache rehydration will stall.