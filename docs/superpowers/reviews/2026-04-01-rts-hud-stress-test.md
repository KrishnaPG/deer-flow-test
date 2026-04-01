# Sci-Fi RTS HUD Stress Test Review

**Date**: 2026-04-01
**Target Spec**: `docs/superpowers/specs/scifi-rts-hud/2026-03-30-scifi-rts-hud-design.md`
**Against**: `docs/architecture/stess-tests.md` & `docs/architecture/state-server.md`

## Executive Summary

The aesthetic direction and component inventory of the Sci-Fi RTS HUD are highly polished. However, Section 8 (Data Model) currently assumes a standard, local, synchronous ECS architecture. When evaluated against the Storage-Native State Server mandates and the architectural stress scenarios, the HUD design exposes critical gaps—particularly concerning the network boundary, event sourcing, and failover behavior. 

The GUI currently violates the fundamental "Mediated Read / Intent Write" rules of the State Server by appearing to own and mutate its local ECS state directly.

## State Server Rule Violations

1. **Write Path (Intents) Violation**: The `CommandConsole` and `BuildQueue` components imply local state mutation upon user interaction (e.g., clicking a build button immediately modifies the `ProductionQueue`). 
   * **Mandate**: "External human commands/intents go through the State Server for immediate ABAC validation, then handed to the Storage Service."
   * **Fix**: The HUD must implement an Intent-dispatch mechanism (e.g., an `EventWriter<PlayerIntent>`). The UI is strictly forbidden from mutating core game state (`ResMut<PlayerResources>`, `ProductionQueue`) directly.
2. **Read Path (Projections) Violation**: The spec lists `Res<PlayerResources>` and `Res<TerrainMap>` as canonical sources. 
   * **Mandate**: The Object Storage is the Source of Truth. The State Server provides a Hot Cache via NATS JetStream.
   * **Fix**: The Bevy ECS state must be defined explicitly as a *read-only projection* driven entirely by NATS JetStream subscriptions.

---

## Scenario 5: Live Stream (HITL) Analysis

* **Context**: Sub-50ms latency is required for Human-In-The-Loop interactions. The State Server "Stream-Tee" pushes tokens to NATS for real-time UI display.
* **Gap**: In an RTS, command dispatch (unit movement, build orders) and event feeds require immediate feedback to feel responsive. If the HUD waits for a full round-trip to the Storage Service (S3 commit) before visually registering a command, the game will feel sluggish and unplayable.
* **Risk**: The current HUD design does not specify how it handles optimistic updates vs. authoritative State Server events. 
* **Recommendation**: 
  * The `CommandConsole` should dispatch an Intent and immediately render a "Pending/Unconfirmed" visual state (e.g., a dashed border on the build queue icon or a ghosted move waypoint).
  * The Bevy client must implement a fast NATS JetStream subscriber that maps `CommandAcknowledged` or `StreamDelta` events back to the ECS to confirm the action, snapping the UI to the authoritative state.

---

## Scenario 7: Replica Crash (Rehydration) Analysis

* **Context**: If the active State Server crashes during a session, a standby replica takes over and rapidly rehydrates its Hot Cache by replaying the NATS JetStream from the last acknowledged sequence.
* **Gap**: The HUD design (specifically Section 7: Animation Spec) is extremely vulnerable to **Rehydration Storms**. 
  * When the new State Server replica replays the last several seconds of events, the Bevy ECS will receive a massive burst of duplicate `LogEvent`, `ResourceCountChanged`, and `UnitUnderAttack` events.
  * Because animations (e.g., `ThreatPing`, `BuildCompleteAnim`, `DataPulse`) are triggered by ECS events, the rehydration burst will cause visual chaos—dozens of threat pings and particle bursts firing simultaneously.
* **Risk**: The `VecDeque<LogEntry>` in the `EventFeed` may also duplicate entries if it lacks idempotency checks against JetStream sequence IDs.
* **Recommendation**: 
  1. **Idempotency**: All incoming State Server events must carry a `sequence_id`. The Bevy app must track the highest processed `sequence_id` to discard duplicate events during a replay.
  2. **Rehydration State**: Introduce a `Res<ConnectionState>` (Connected, Disconnected, Rehydrating). When `Rehydrating` is true, data events silently update the ECS (resources, health, build queues) but **do not trigger transient visual animations**. Animations must be suppressed until the client catches up to the NATS tail.

---

## Scenario 8: Concurrent Conflicting Intents

* **Context**: Multiple operators (e.g., co-op commanders) issue commands simultaneously. The State Server uses deterministic NATS JetStream sequencing to resolve conflicts (Last Write Wins).
* **Gap**: If the GUI assumes it owns the state or resolves conflicts locally, it will desync from the Storage-Native truth.
* **Recommendation**: The UI must rely entirely on the deterministically sequenced events from the State Server. If Operator A and Operator B click a unique, single-use resource simultaneously, Operator A's UI might optimistically claim it. When the State Server processes Operator B first (due to network routing), Operator A's UI must smoothly rollback the optimistic update and render an "Action Rejected" alert, relying purely on the NATS event order.

## Next Actions for the Spec

1. Update **Section 8 (Data Model)** to map Bevy ECS resources to NATS Hot Cache subscriptions.
2. Add a new section defining the **Intent Dispatch Boundary** for external writes.
3. Update **Section 7 (Animation Spec)** with a rule to suppress visual triggers during State Server Rehydration.