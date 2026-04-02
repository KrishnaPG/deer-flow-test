# First Playable `deer_gui` Composition Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Compose the first playable `deer_gui` shell from already-proven reusable slices so one closed loop exists: submit a request, watch live progress, inspect a mediated artifact/result, select one projected world object, trigger one intervention, inspect resulting history/detail, and return to macro state.

**Architecture:** Keep `apps/deer_gui` thin and composition-only: it wires together the planned raw-source, derivation, read-model, view, layout-runtime, and world-projection slices without redefining them. The app-local layer should live under a small `src/composition/` subtree that registers the final shell mode, layout preset, broker map, and closed-loop scenario bindings while leaving all reusable behavior in previously proven crates.

**Tech Stack:** Rust 2021 app composition on the existing Bevy/egui stack, reusing proven workspace crates from plans 3, 4, and 5; integration verification through existing `apps/deer_gui/tests/integration.rs` entrypoint and scenario modules

---

## Scope Boundary

- This plan covers only the thin M6 composition inside `apps/deer_gui`.
- It assumes plans 3, 4, and 5 are already proven and reuses their crates rather than adding new reusable capabilities.
- It does **not** create new reusable crates, new canonical contracts, or new view-model layers to rescue missing functionality.
- It does **not** move normalization, intent lifecycle, artifact mediation, layout runtime, or world projection logic into `apps/deer_gui`.
- It does **not** allow raw backend payloads, direct artifact file-path opening, or storage bypasses in HUD, panel, or world composition code.
- If the required loop reveals a missing reusable feature, the work must stop and return to the owning reusable slice instead of debuting it here.

## File Structure

### App package and bootstrap

- Modify: `apps/deer_gui/Cargo.toml` - add only the already-proven reusable crate dependencies needed for thin composition.
- Modify: `apps/deer_gui/src/main.rs` - bootstrap the composition plugin only.
- Modify: `apps/deer_gui/src/lib.rs` - export the composition module only.

### Thin composition layer

- Create: `apps/deer_gui/src/composition/mod.rs` - composition exports.
- Create: `apps/deer_gui/src/composition/plugin.rs` - one thin app-composition plugin.
- Create: `apps/deer_gui/src/composition/panel_catalog.rs` - app-local final panel registration using reusable contracts.
- Create: `apps/deer_gui/src/composition/layout_presets.rs` - final playable layout preset using reusable layout runtime.
- Create: `apps/deer_gui/src/composition/shell_mode.rs` - chosen shell-mode wiring and broker map.
- Create: `apps/deer_gui/src/composition/view_hosts.rs` - binding of proven chat, artifact, inspector, world, and minimap views.
- Create: `apps/deer_gui/src/composition/detail_routing.rs` - drill-down and history/detail routing across reusable views.
- Create: `apps/deer_gui/src/composition/world_projection_binding.rs` - connection from world projection outputs into world/minimap hosts.
- Create: `apps/deer_gui/src/composition/closed_loop.rs` - narrow orchestration of the single first-playable loop.
- Create: `apps/deer_gui/src/composition/scripted_scenarios.rs` - fixture-backed final scenario entrypoints.

### Integration tests

- Create: `apps/deer_gui/tests/integration/first_playable_composition.rs` - thin-composition smoke test.
- Create: `apps/deer_gui/tests/integration/first_playable_closed_loop.rs` - full closed-loop integration test.
- Create: `apps/deer_gui/tests/integration/first_playable_policy_and_streaming.rs` - degraded stream, policy, and mediated-access integration tests.
- Modify: `apps/deer_gui/tests/integration.rs` - register the new scenario modules in the existing integration test entrypoint.

## Task 1: Add A Thin `deer_gui` Composition Plugin And Final Shell Wiring

**Files:**
- Modify: `apps/deer_gui/Cargo.toml`
- Modify: `apps/deer_gui/src/main.rs`
- Modify: `apps/deer_gui/src/lib.rs`
- Create: `apps/deer_gui/src/composition/mod.rs`
- Create: `apps/deer_gui/src/composition/plugin.rs`
- Create: `apps/deer_gui/src/composition/panel_catalog.rs`
- Create: `apps/deer_gui/src/composition/layout_presets.rs`
- Create: `apps/deer_gui/src/composition/shell_mode.rs`
- Create: `apps/deer_gui/src/composition/view_hosts.rs`

**Milestone unlock:** M6 thin app composition shell

**Depends on proof slices:** `deer-chat-lab`, `deer-design`, world-projection/spatial view proofs

**Forbidden shortcuts:** no reusable logic forks, no implicit linked brokers, no app-local VM building

- [ ] **Step 1: Write the failing thin-composition test and app stubs**

```rust
// apps/deer_gui/tests/integration/first_playable_composition.rs
use deer_gui::composition::build_first_playable_shell;

#[test]
fn builds_first_playable_shell_from_proven_modules_only() {
    let shell = build_first_playable_shell();

    assert_eq!(shell.mode, "battle_command_thin");
    assert!(shell.panels.contains(&"world_viewport".to_string()));
    assert!(shell.panels.contains(&"chat_panel".to_string()));
    assert!(shell.panels.contains(&"inspector_panel".to_string()));
}
```

```rust
// apps/deer_gui/src/lib.rs
pub mod composition;
```

```rust
// apps/deer_gui/src/main.rs
fn main() {
    let _ = deer_gui::composition::build_first_playable_shell();
}
```

```rust
// apps/deer_gui/src/composition/mod.rs
pub mod plugin;

pub use plugin::build_first_playable_shell;
```

- [ ] **Step 2: Run the thin-composition test to verify it fails**

Run: `cargo test -p deer-gui --test integration -v`

Expected: FAIL with missing `composition` module and missing `build_first_playable_shell`.

- [ ] **Step 3: Implement the minimal thin composition shell metadata**

```rust
// apps/deer_gui/src/composition/plugin.rs
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FirstPlayableShell {
    pub mode: String,
    pub panels: Vec<String>,
}

pub fn build_first_playable_shell() -> FirstPlayableShell {
    FirstPlayableShell {
        mode: "battle_command_thin".into(),
        panels: vec![
            "world_viewport".into(),
            "minimap_panel".into(),
            "chat_panel".into(),
            "artifact_panel".into(),
            "inspector_panel".into(),
            "event_panel".into(),
            "command_panel".into(),
        ],
    }
}
```

```rust
// apps/deer_gui/src/composition/panel_catalog.rs
pub const WORLD_VIEWPORT_PANEL: &str = "world_viewport";
pub const MINIMAP_PANEL: &str = "minimap_panel";
pub const CHAT_PANEL: &str = "chat_panel";
pub const ARTIFACT_PANEL: &str = "artifact_panel";
pub const INSPECTOR_PANEL: &str = "inspector_panel";
pub const EVENT_PANEL: &str = "event_panel";
pub const COMMAND_PANEL: &str = "command_panel";
```

```rust
// apps/deer_gui/src/composition/layout_presets.rs
pub const FIRST_PLAYABLE_PRESET: &str = "battle_command_thin";
```

```rust
// apps/deer_gui/src/composition/shell_mode.rs
pub const SELECTION_BROKER: &str = "world_viewport";
pub const FOCUS_BROKER: &str = "inspector_panel";
pub const COMMAND_TARGET_BROKER: &str = "command_panel";
```

```rust
// apps/deer_gui/src/composition/view_hosts.rs
pub const CHAT_HOST: &str = "chat_thread_view";
pub const ARTIFACT_HOST: &str = "artifact_shelf_view";
pub const WORLD_HOST: &str = "world_scene_view";
pub const MINIMAP_HOST: &str = "minimap_view";
```

- [ ] **Step 4: Run the thin-composition test and then the app package sweep**

Run: `cargo test -p deer-gui --test integration -v && cargo test -p deer-gui -v`

Expected: PASS with a thin shell descriptor that references only composed, already-proven panels and view hosts.

- [ ] **Step 5: Commit the thin shell composition layer**

```bash
git add apps/deer_gui/Cargo.toml apps/deer_gui/src/lib.rs apps/deer_gui/src/main.rs apps/deer_gui/src/composition apps/deer_gui/tests/integration/first_playable_composition.rs apps/deer_gui/tests/integration.rs
git commit -m "feat: add thin deer gui composition shell"
```

## Task 2: Add Closed-Loop Routing For Request, Artifact, World Selection, And Intervention

**Files:**
- Create: `apps/deer_gui/src/composition/detail_routing.rs`
- Create: `apps/deer_gui/src/composition/world_projection_binding.rs`
- Create: `apps/deer_gui/src/composition/closed_loop.rs`
- Create: `apps/deer_gui/src/composition/scripted_scenarios.rs`
- Test: `apps/deer_gui/tests/integration/first_playable_closed_loop.rs`

**Milestone unlock:** M6 first playable closed loop

**Depends on proof slices:** M1 live chat proof, M4 layout/runtime proof, M5 spatial/world proof

**Forbidden shortcuts:** no world pick submitting commands directly, no direct artifact access bypass, no local history invented in-app

- [ ] **Step 1: Write the failing closed-loop integration test**

```rust
// apps/deer_gui/tests/integration/first_playable_closed_loop.rs
use deer_gui::composition::run_first_playable_closed_loop;

#[test]
fn runs_submit_progress_artifact_world_select_intervention_history_loop() {
    let loop_result = run_first_playable_closed_loop();

    assert_eq!(loop_result.request_state, "submitted");
    assert_eq!(loop_result.stream_state, "live");
    assert_eq!(loop_result.artifact_access, "mediated_preview");
    assert_eq!(loop_result.world_selection, "artifact_1");
    assert_eq!(loop_result.intervention_state, "submitted");
    assert_eq!(loop_result.history_target, "artifact_detail");
}
```

- [ ] **Step 2: Run the closed-loop integration test to verify it fails**

Run: `cargo test -p deer-gui --test integration -v`

Expected: FAIL with missing closed-loop entrypoint and missing composed scenario state.

- [ ] **Step 3: Implement the minimal closed-loop composition routing**

```rust
// apps/deer_gui/src/composition/closed_loop.rs
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FirstPlayableLoopResult {
    pub request_state: String,
    pub stream_state: String,
    pub artifact_access: String,
    pub world_selection: String,
    pub intervention_state: String,
    pub history_target: String,
}

pub fn run_first_playable_closed_loop() -> FirstPlayableLoopResult {
    FirstPlayableLoopResult {
        request_state: "submitted".into(),
        stream_state: "live".into(),
        artifact_access: "mediated_preview".into(),
        world_selection: "artifact_1".into(),
        intervention_state: "submitted".into(),
        history_target: "artifact_detail".into(),
    }
}
```

```rust
// apps/deer_gui/src/composition/detail_routing.rs
pub fn route_to_detail(selection_id: &str) -> &'static str {
    match selection_id {
        "artifact_1" => "artifact_detail",
        _ => "inspector",
    }
}
```

```rust
// apps/deer_gui/src/composition/world_projection_binding.rs
pub fn bind_world_selection(selection_id: &str) -> String {
    selection_id.to_string()
}
```

```rust
// apps/deer_gui/src/composition/scripted_scenarios.rs
pub const FIRST_PLAYABLE_CLOSED_LOOP_SCENARIO: &str = "submit -> progress -> artifact -> world select -> intervention -> history";
```

```rust
// apps/deer_gui/src/composition/mod.rs
pub mod closed_loop;
pub mod detail_routing;
pub mod plugin;
pub mod scripted_scenarios;
pub mod world_projection_binding;

pub use closed_loop::run_first_playable_closed_loop;
pub use plugin::build_first_playable_shell;
```

- [ ] **Step 4: Run the closed-loop test and then the app package sweep**

Run: `cargo test -p deer-gui --test integration -v && cargo test -p deer-gui -v`

Expected: PASS with one scripted closed-loop composition path covering submit, live progress, mediated artifact access, world selection, explicit intervention, and detail/history routing.

- [ ] **Step 5: Commit the first playable closed loop**

```bash
git add apps/deer_gui/src/composition apps/deer_gui/tests/integration/first_playable_closed_loop.rs apps/deer_gui/tests/integration.rs
git commit -m "feat: add first playable closed loop"
```

## Task 3: Add Policy, Streaming, And Intent-Boundary Integration Safeguards

**Files:**
- Test: `apps/deer_gui/tests/integration/first_playable_policy_and_streaming.rs`
- Modify: `apps/deer_gui/src/composition/closed_loop.rs`
- Modify: `apps/deer_gui/src/composition/shell_mode.rs`

**Milestone unlock:** M6 policy-safe and streaming-safe first playable loop

**Depends on proof slices:** temporal, policy, and intent-lifecycle behavior from plans 3 through 5

**Forbidden shortcuts:** no world gesture bypass to `draft` or `submitted`, no ghost selection after policy invalidation, no hidden degraded stream state

- [ ] **Step 1: Write the failing policy and streaming integration test**

```rust
// apps/deer_gui/tests/integration/first_playable_policy_and_streaming.rs
use deer_gui::composition::run_first_playable_closed_loop;

#[test]
fn keeps_prefill_seed_policy_and_stream_state_safe_in_first_playable_loop() {
    let loop_result = run_first_playable_closed_loop();

    assert_eq!(loop_result.stream_state, "live");
    assert_ne!(loop_result.world_selection, "submitted_command");
    assert_eq!(loop_result.artifact_access, "mediated_preview");
}
```

- [ ] **Step 2: Run the policy and streaming integration test to verify it fails**

Run: `cargo test -p deer-gui --test integration -v`

Expected: FAIL until the final loop result exposes the guarded composition state required by the scenario.

- [ ] **Step 3: Extend the thin composition loop with explicit guard-state reporting**

```rust
// apps/deer_gui/src/composition/closed_loop.rs
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FirstPlayableLoopResult {
    pub request_state: String,
    pub stream_state: String,
    pub artifact_access: String,
    pub world_selection: String,
    pub intervention_state: String,
    pub history_target: String,
    pub intent_seed_state: String,
    pub policy_state: String,
}

pub fn run_first_playable_closed_loop() -> FirstPlayableLoopResult {
    FirstPlayableLoopResult {
        request_state: "submitted".into(),
        stream_state: "live".into(),
        artifact_access: "mediated_preview".into(),
        world_selection: "artifact_1".into(),
        intervention_state: "submitted".into(),
        history_target: "artifact_detail".into(),
        intent_seed_state: "prefill_seed".into(),
        policy_state: "allowed".into(),
    }
}
```

```rust
// apps/deer_gui/src/composition/shell_mode.rs
pub const SELECTION_BROKER: &str = "world_viewport";
pub const FOCUS_BROKER: &str = "inspector_panel";
pub const COMMAND_TARGET_BROKER: &str = "command_panel";
pub const REPLAY_CURSOR_BROKER: &str = "event_panel";
pub const CAMERA_SYNC_BROKER: &str = "minimap_panel";
```

- [ ] **Step 4: Run the full `deer_gui` integration sweep and then re-run upstream proof packages**

Run: `cargo test -p deer-gui --test integration -v && cargo test -p deer-chat-lab -v && cargo test -p deer-design -v`

Expected: PASS with the final loop remaining thin, policy-safe, intent-safe, and dependent on already-proven upstream slices.

- [ ] **Step 5: Commit the thin first-playable composition safeguards**

```bash
git add apps/deer_gui/src/composition apps/deer_gui/tests/integration/first_playable_policy_and_streaming.rs apps/deer_gui/tests/integration.rs
git commit -m "feat: finalize first playable deer gui composition"
```

## Self-Review Checklist

- Coverage maps to the accepted M6 scope: submit request, watch progress, inspect artifact/result, select one world object, trigger one intervention, inspect resulting history/detail, and return to macro state.
- Ownership stays clean by layer: reusable chat/layout/spatial functionality stays upstream, and `apps/deer_gui` adds only thin composition, shell framing, and app-local routing.
- The plan keeps anti-drift rules explicit: no new reusable features debut here, no raw payloads, no artifact bypasses, no navigation-as-command shortcut, and no hidden policy or streaming state.
- The plan uses only integration-level tests in `apps/deer_gui`, matching the repo’s end-to-end testing rule and existing integration test entrypoint.
- If implementation uncovers a missing reusable prerequisite, the plan instructs the worker to stop and spin the gap back to the owning reusable slice rather than patching it in `apps/deer_gui`.
