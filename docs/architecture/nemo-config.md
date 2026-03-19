# Nemo-Config: Infrastructure Control Plane & DX

This document outlines the architecture, lifecycle, and developer/operator workflow for `nemo-config` tool (Consul Environment Management & Orchestration). 

`nemo-config` is a standalone, reusable infrastructure control plane designed to eliminate `.env` file management, dynamic container provisioning, and fragile application startup sequences across both development and production environments.

## Core Philosophy: The Decoupled Boot Sequence

The architecture strictly separates the **Application Data Plane** (the actual business logic, e.g., the State Server) from the **Infrastructure Control Plane** (`nemo-config`).

1.  **Applications NEVER crash on startup due to missing infrastructure.** They boot instantly into a "Degraded Mode" and wait patiently for configuration.
2.  **Configuration is Stateful and Centralized.** All connection strings (Postgres, ClickHouse, MinIO, etc.) live securely in the Consul Key-Value (KV) store.
3.  **Infrastructure Provisioning is Dynamic.** `nemo-config` can deploy missing services to remote WireGuard nodes via SSH, rather than relying on monolithic, static `docker-compose.yml` files.

---

## 1. The `nemo-config` Tool (The Control Plane)

`nemo-config` is a standalone CLI/UI tool run strictly for provisioning, discovery, and monitoring. It is **NOT** bundled into the production binary of any application. Once the cluster is green, it can be shut down entirely.

### Capabilities

*   **The Catalog:** Ships with pre-configured templates for popular open-source infrastructure (Postgres, Redis, ClickHouse, MinIO, SigNoz, LiveKit). It knows *how* to deploy and health-check them, but is agnostic to whether a specific app *needs* them.
*   **The Actor (UI/CLI):** Allows the user to:
    *   Toggle services ON/OFF from the catalog.
    *   Provide an existing URL for a toggled service (reusing existing infra on the VPN).
    *   Provide SSH credentials to dynamically deploy a toggled service to a remote WireGuard node (e.g., executing `docker compose up -d` over SSH).
*   **The Publisher:** As services become healthy (either existing or newly deployed), `nemo-config` publishes their connection strings to the central Consul KV store (e.g., prefix: `nemo/postgres`, key: `url`).
*   **The Monitor (Optional):** If left running, it acts as a lightweight cluster dashboard, showing real-time health (GREEN/RED) by polling service endpoints.

---

## 2. The Application Lifecycle (e.g., State Server)

The main application (API, Edge worker, etc., which is different from the current `nemo-config` tool) is completely agnostic to *how* its infrastructure was provisioned. 

### Startup Requirements
The application only knows two things on startup:
1.  The `CONSUL_URL` (the singular "Bootstrap Node").
2.  Its own struct of prerequisites (e.g., `requires: { db: true, storage: true, event_bus: true }`).

### The "Degraded Mode" State Machine

*   **The Watcher:** On boot, the application connects to Consul and watches the `nemo/` KV prefix.
*   **State: Degraded:** If the KV store is missing required keys (e.g., `nemo/postgres/url`), the app stays in degraded mode. It performs no business logic. Its health endpoint returns `503 Service Unavailable` with a JSON payload indicating missing dependencies (`{"status": "degraded", "missing": ["postgres"]}`).
*   **State: Functional (The Magic Transition):** The moment the app's Consul watcher detects that all required keys are present in the KV store, it dynamically initializes its connection pools. If successful, it transitions to "Functional", logs `[INFO] All prerequisites met`, and begins serving traffic instantly—zero restarts required.

---

## 3. Developer & Operator Workflows

Because `nemo-config` and the application are decoupled, the workflow is identical for both a new developer cloning the repo and an operator deploying to a production cluster.

### Scenario: Developer Onboarding (or Production Deployment)

1.  **Clone & Boot the App:**
    *   The developer clones the application repository.
    *   They run `cargo run` (or deploy the production binary). 
    *   The app boots instantly but enters **Degraded Mode** because the Consul KV store lacks the required URLs.
2.  **Start the Config Tool:**
    *   The developer runs the standalone `nemo-config` tool (e.g., `nemo start`).
    *   A local UI opens (`localhost:8080`).
3.  **Configure / Deploy Infrastructure:**
    *   The UI displays the catalog. The developer selects Postgres and MinIO.
    *   They point Postgres to an *existing* IP on the WireGuard VPN (reusing infrastructure).
    *   They use the *SSH deploy feature* to spin up MinIO on a spare Raspberry Pi (or cloud VM) on the network.
4.  **The Magic Transition:**
    *   As soon as `nemo-config` verifies MinIO is up, it writes the URL to Consul KV (e.g., `nemo/minio/url`). 
    *   The main application (sitting in Degraded Mode watching Consul) instantly detects the change, builds its connection pools, and is ready for use.
5.  **Cleanup:**
    *   The developer closes the `nemo-config` UI. The main application continues running perfectly. The configuration is safely persisted in Consul.

### Scenario: Hot-Swapping a Database
If a database needs to be migrated to a new server:
1.  Open the `nemo-config` UI.
2.  Update the Postgres IP address.
3.  The main application detects the KV change via Consul, drops the old Postgres pool, creates a new one, and continues serving requests—zero downtime, no `.env` edits, no container restarts.
