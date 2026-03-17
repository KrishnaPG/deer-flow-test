# Strategy to Prevent Architectural Drift

AI code generators tend to take shortcuts (e.g., writing directly to the database) when their context window shifts or when asked broad architectural questions. To successfully implement the "Storage-Native, Mediated-Read" architecture, we must enforce strict physical and logical boundaries *before* generating code.

## 1. Context Fences (`CONSTRAINTS.md` / `.cursorrules`)

Every AI session must have the following absolute rules injected into its system prompt or workspace context:

1.  **Thin Clients:** Agents and Workers are THIN CLIENTS. They NEVER import database drivers (no Postgres, ClickHouse, Milvus, or S3 SDKs). They ONLY communicate via HTTP/gRPC to the Storage Service or Temporal.
2.  **Absolute Immutability:** NEVER generate SQL `UPDATE` or `DELETE` statements. Data is append-only.
3.  **Read Path:** All UI reads MUST go through the State Server. The UI NEVER connects to a database directly.
4.  **No Direct DB Writes:** No component ever writes raw data or artifacts directly to a database. All truth goes to Object Storage (S3/lakeFS) via the Storage Service.

## 2. Enforce Boundaries with Static Analysis (Fitness Functions)

Do not rely on the AI remembering the rules. Enforce them in CI/CD or pre-commit hooks to physically block drift.

*   Use static analysis tools (e.g., `eslint-plugin-import` for Node, `ArchUnit` for Java, `cargo-deny` for Rust) to ban dependency imports in specific domains.
*   **Example:** Statically forbid the `src/agents/` directory from importing `@clickhouse/client` or `@aws-sdk/client-s3`. If an AI attempts to write code importing these, the linter will fail instantly, forcing a correction.

## 3. Contract-First (Schema-Driven) Development

Before asking the AI to write logic, manually or collaboratively define strict data contracts.

*   Define NATS Event payloads using **JSON Schema** or **Protobuf**.
*   Define the Storage Service API using **OpenAPI (Swagger)**.
*   **Prompting Strategy:** When assigning a task, constrain the AI to the contract: *"Implement this exact OpenAPI spec. Do not deviate."* This drastically reduces cognitive load and prevents hallucinated architectures.

## 4. Context-Isolated Tasking

Never ask the AI to "Build the State Server architecture." Break it down into siloed tasks where the AI doesn't *need* the whole context window.

*   **Good Task:** "Write a Rust function that takes a byte array, computes its SHA-256 hash, and writes it to this specific S3 path."
*   **Good Task:** "Write a Temporal Activity that takes an S3 URI, downloads the file, chunks it by paragraph, and POSTs the chunks to `http://storage-service/ingest`."
*   **Bad Task:** "Build the ingestion pipeline." (Too broad; invites drift).

---

# Implementation Roadmap

Build the system from the "Truth" outwards.

## Phase 0: Infrastructure & Contracts (The Foundation)
1.  Create a `docker-compose.yml` with MinIO (local S3), NATS JetStream, ClickHouse, Postgres, and Redis.
2.  Draft the exact JSON Schemas for the NATS `EventBus` (e.g., `WriteProgress`, `ArtifactCreated`, `StreamDelta`).
3.  Set up the repository structure and static analysis rules (import bans).

## Phase 1: The Storage Service & Event Bus (The Ingress)
1.  Implement the `POST /write` endpoint.
2.  Implement the logic to hash the file (SHA-256), write it to S3, and parallel-publish the `ArtifactCreated` event to NATS JetStream (Dual-Dispatch).
3.  **Test:** Send a file, verify it exists in MinIO with the hash as the filename, and verify the event is in the NATS queue.

## Phase 2: The State Server (The Gatekeeper & Cache)
1.  Implement the NATS subscriber that listens to the Event Bus and updates an in-memory "Hot Window".
2.  Implement a WebSocket endpoint that streams this Hot Window to a dummy UI.
3.  Implement basic Casbin/Postgres ABAC validation for incoming Intents.

## Phase 3: The Cold Storage Views (ClickHouse & Sync Workers)
1.  Configure ClickHouse `S3-engine` tables to mount the MinIO `as_is` and `chunks` directories directly.
2.  Create a Temporal Worker that listens to NATS for `ArtifactCreated` events, chunks the text, and calls the Storage Service to save the L1-L4 chunks.

## Phase 4: Agents, Streaming, and LiveKit (The Edges)
1.  Implement the "Stream-Tee" in the Storage Service (accepting token streams, pushing to NATS fast, batching to S3 slow).
2.  Spin up a basic thin-client Agent that generates text and sends it to the Stream-Tee.
3.  Integrate LiveKit Egress to dump WebRTC audio into the MinIO landing zone.

## Phase 0: Infrastructure, Configuration, & Contracts (The Foundation)

We use a high-performance **Rust (Backend) + Bun/React (Frontend)** stack managed as a **Turborepo** monorepo. We avoid fragile `.env` files and local Docker clusters, relying on a WireGuard VPN to access shared remote services.

### 1. Centralized Configuration (NATS KV)
To easily relocate code across machines without `.env` files:
1.  **Configuration as Code:** Store environment configurations (e.g., `dev.config.json`) centrally in the **NATS JetStream Key-Value (KV) store** on the remote cluster.
2.  **Bootstrap:** Services (Rust/Bun) only require a single environment variable: `NATS_URL`.
3.  **Loading:** On startup, the service connects to NATS, fetches its configuration payload from the KV store, and initializes its connections (S3, ClickHouse, Redis) in memory. This allows instant relocation and real-time remote config updates.

### 2. Monorepo Structure (Turborepo)
```text
deer-flow/
├── turbo.json                  # Turborepo orchestration (build, lint, test)
├── package.json                # Bun workspace root
├── Cargo.toml                  # Rust workspace root
├── docs/                       # Architecture and specs
├── packages/
│   ├── contracts/              # Shared Protobuf schemas (.proto)
│   └── config-loader/          # Rust/Bun libs: Fetches config from NATS KV
├── apps/
│   ├── storage-service/        # (Rust) The NATS/S3 Dual-Dispatch Ingress
│   ├── state-server/           # (Rust) Hot Cache, ABAC, WebSockets
│   └── control-center/         # (Bun + React + Vite) Operator UI
└── workers/
    └── chunking-worker/        # (Rust) Temporal worker for L0 -> L3
```

### 3. Strict Data Contracts (Protobuf)
Because performance is critical and we bridge Rust and Bun:
*   Use **Protobuf (`.proto`)** for all NATS Event Bus payloads (`StreamDelta`, `ArtifactCreated`, `IntentReceived`).
*   Protobuf provides strong typing, blazing-fast serialization in Rust, and generates lean TypeScript interfaces for Bun.

### 4. Enforce Static Fences (Linting)
*   **Rust (`cargo-deny`):** Configure workspace rules to strictly forbid `apps/storage-service` or `apps/state-server` from importing direct database drivers like `clickhouse` or `sqlx` (forcing them to use the Storage-Native abstractions).
*   **Bun (`eslint-plugin-import`):** Forbid `apps/control-center` from importing any backend logic or database libraries, ensuring all reads go through the State Server API.
