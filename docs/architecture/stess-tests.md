# State Server Stress Tests

This document defines the architectural stress tests and extreme edge cases the Storage-Native State Server is designed to survive. 

## Scenario 1: Deep Scientific Research Lifecycle (10,000+ Parallel Agents)
*A massive burst of worker processes generating hypotheses, code, and datasets.*

*   **Action:** 10,000 agents simultaneously chunk L3 data.
*   **Data Path:** Agents fire-and-forget chunks to the Storage Service. The Storage Service uses aggressive in-memory Bloom filters (Hash Collapsing) to deduplicate exact matches (e.g., 1,000 identical "Error: timeout" logs) into a **single S3 write**, while parallel-emitting 1,000 NATS events. Background workers micro-batch the rest into Parquet files.
*   **Stress Point Survived:** Storage costs and DB query times are annihilated by deduplication and micro-batching. State Server ignores the write burst.

## Scenario 2: YouTube Translate + Dub + Lip-Sync (HOTL)
*ASR -> Translate -> TTS -> Align -> Render pipeline.*

*   **Action:** A complex, multi-stage pipeline transcribes an original L0 video, translates it, synthesizes new audio, and renders a lip-synced composite video tailored for a target culture.
*   **Data Path (Storage):** Workers fire-and-forget large L1-L4 artifacts (video/audio blobs) to the Storage Service.
*   **State Server Role:** Listens to NATS `WriteProgress` events to update the UI's progress bars in real-time. The operator can visually observe the pipeline's exact state without blocking worker execution (HOTL pattern). Accepts operator overrides (intents) to edit the glossary.
*   **Stress Point Survived:** The UI remains perfectly responsive and informative while massive background I/O occurs asynchronously. Glossary edits spawn new branches via immutability, preserving older versions.

## Scenario 3: Right to be Forgotten (Compliance & Immutability)
*A user requests the deletion of a massive, heavily processed L0 document.*

*   **Action:** User clicks "Delete Document" in the UI.
*   **Data Path:** The UI sends an intent to the State Server. The State Server validates ABAC and hands the intent to the Storage Service.
*   **The Immutable Ledger:** The Storage Service writes a new, immutable record to the `s3://.../exclusions/` prefix: `{ target_hash: "as_is_hash_xyz", reason: "user_request" }`.
*   **Database View (Overlay):** ClickHouse and Milvus dynamically construct their "Search Views" using an anti-join or filter against the Exclusions ledger.
*   **Stress Point Survived:** Immediate UI disappearance satisfying compliance without violating append-only S3 guarantees.

## Scenario 4: Massive Video Proxying (LakeFS / Streaming)
*A user requests to view an original 4GB L0 raw MP4 file.*

*   **Action:** User clicks "View Original Source" in the UI.
*   **Data Path:** UI requests the file from the State Server.
*   **State Server Role:** Validates ABAC permissions, generates a Pre-Signed URL pointing directly to the specific version commit in lakeFS/S3, and returns the URL.
*   **Stress Point Survived:** Prevents State Server OOM crashes and bandwidth saturation by never proxying large blob traffic.

## Scenario 5: Real-Time Audio/Text Streaming Chat (HITL)
*User speaking to an AI agent with millisecond latency requirements.*

*   **Action:** User initiates a live voice conversation with an AI Agent.
*   **Data Path (Media):** Direct WebRTC connection via LiveKit (sub-50ms latency). LiveKit Egress independently records the raw session to an S3 "landing zone" for later Hash & Promote to L0 truth by the Storage Service.
*   **Data Path (Text/Data):** AI Agent streams text to the Storage Service. The "Stream-Tee" instantly pushes tokens to NATS for real-time UI display, while micro-batching sentences to S3 in the background.
*   **Stress Point Survived:** Absolute low-latency HITL interaction while preserving strict "Storage-Native" truth, completely offloading the transport layer, and keeping AI Agents extremely "thin".

## Scenario 6: S3 / Object Storage Rate Limiting (503 Slow Down)
*The underlying cloud object storage begins aggressively rate-limiting PUT requests during a spike.*

*   **Action:** Storage Service receives `503 Slow Down` from S3.
*   **Data Path:** Storage Service pauses pushing from its internal NATS JetStream ingress queue to S3. Agents remain unaffected and continue writing to the persistent queue instantly. 
*   **State Server Role:** The UI continues updating based on NATS events (which are independent of S3).
*   **Stress Point Survived:** No data is dropped. Agents do not block or crash. Temporal workflows implement exponential backoff seamlessly until S3 recovers.

## Scenario 7: State Server Replica Crash / Redis Leadership Loss during an active WebRTC Session
*The primary active State Server handling an ongoing HITL agent session crashes.*

*   **Action:** Kubernetes/Nomad kills the State Server pod.
*   **Data Path:** A standby State Server assumes the Redis leadership lease. LiveKit maintains the media connection directly with the user and the agent (WebRTC peer-to-peer or SFU proxy bypasses State Server). 
*   **State Server Role:** The new State Server replica rapidly rehydrates its Hot Cache by replaying the NATS JetStream from the last acknowledged sequence for that `mission_id`.
*   **Stress Point Survived:** The audio/video chat never drops. UI updates resume seamlessly within milliseconds after the new replica catches up to the NATS tail.

## Scenario 8: Concurrent Conflicting Intents
*Two operators editing the same knowledge base glossary simultaneously.*

*   **Action:** Operator A adds "Apples = Red", Operator B adds "Apples = Green" at the exact same millisecond.
*   **Data Path:** Both intents hit the State Server, validate ABAC, and are forwarded to the Storage Service Event Bus.
*   **The Immutable Ledger:** The Storage Service appends *both* events to the immutable log. They are assigned a strict, deterministic sequence ID by NATS JetStream.
*   **Database View:** ClickHouse reads the log. Using an `argMax` or standard "Last Write Wins" materialized view on the sequence ID, the conflict is deterministically resolved.
*   **Stress Point Survived:** No distributed database row locks required. Complete audit trail of *both* operators' actions is preserved immutably.
