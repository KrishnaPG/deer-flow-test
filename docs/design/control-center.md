`apps/deer_gui` is an `egui` desktop app, not a web shell, and the current UI in `apps/deer_gui/src/app.rs` is still a classic `threads | chat | artifacts` workspace. Given the state-server model in `docs/architecture/state-server.md` and the stress cases in `docs/architecture/stess-tests.md`, I'd redesign it as a calm scientific operations cockpit with progressive depth instead of a denser sidebar app.

**Recommended Direction**
- Replace the thread-first mental model with a `mission / workflow / agent swarm` model; chat becomes one tool inside the mission, not the whole product.
- Treat the UI as 5 layers: `Command`, `Observe`, `Intervene`, `Inspect`, `Recover`.
- Default to abstraction first: show health, progress, blockers, approvals, and important artifacts; reveal logs, lineage, and raw events only on demand.
- Separate `HITL` and `HOTL` behavior in the UX: live co-pilot sessions need focus and low-latency surfaces; background swarms need summarized supervision and exception routing.
- Keep the visual tone calm and research-grade, not neon-devops; a light "lab console" aesthetic fits the current app better than a dark terminal clone.

**Core UX Model**
- `Mission`: the top-level unit the user cares about.
- `Workflow`: a durable run or branch under a mission.
- `Agent Swarm`: many agents contributing to a workflow.
- `Artifact Graph`: outputs, references, datasets, notes, images, reports, logs, embeddings lineage.
- `Attention Queue`: approvals, failures, retries, ambiguity, ready artifacts, completion milestones.
- `Focus Lens`: the current zoom level, from executive summary down to raw logs.

**New Shell**
- Left rail: `Missions` and saved `Views`, not old chat threads.
- Main canvas: adaptive center surface that switches between `Command Deck`, `Experts Meeting`, `Workflow Timeline`, and `Artifact Graph`.
- Right dock: contextual inspector for selected mission/workflow/agent/artifact.
- Bottom event rail: compact live stream of important events, collapsible and filterable.
- Top strip: global status, active watches, pending HITL approvals, and system health.

**Key Screens / Modes**
- `Command Deck`: prompt/composer + mission brief + current watches + recent outputs; this is the default landing view.
- `Experts Meeting`: real-time multi-agent conversation with pinned context chips for images, files, graphs, and artifacts.
- `Swarm Monitor`: clustered view of 100s of agents by stage, health, retry state, and throughput; never render every agent equally.
- `Workflow Timeline`: stage-by-stage pipeline with retries, waits, approvals, and branch points.
- `Artifact Graph`: lineage-first browser using the correlation contract from `docs/architecture/state-server.md`; let users trace `prompt -> agent -> artifact -> memory`.
- `Forensics`: raw logs, spans, event tail, and filters by `mission_id`, `run_id`, `agent_id`, `artifact_id`, `trace_id`.

**How To Avoid Overwhelming Users**
- Use progressive disclosure: summary cards first, then drill-down drawers, then raw logs.
- Aggregate by default: "23 agents indexing / 4 retrying / 2 blocked for approval" rather than 29 rows.
- Promote only actionable alerts; auto-silence repetitive success noise and dedup repeated failures.
- Let users pin a workflow, agent, or artifact to a `Watchlist`; all else stays backgrounded.
- Add focus filters: `Only blocked`, `Only my watched`, `Only artifact-ready`, `Only errors`, `Only live`.
- Keep one persistent "Why should I care now?" panel for top-priority events.

**HITL / HOTL UX**
- `HITL`: interruptive but bounded cards with clear action, deadline, and consequence; approval should be 1-click with a drill-in option.
- `HOTL`: passive monitoring cards with confidence/progress/state; user watches without being dragged into noise.
- Both should feed one unified attention queue, but visually distinct: approvals in amber, watches in blue/teal, failures in rust/red.
- For live agent brainstorming, use an "active conversation stage" with side-by-side context pins instead of burying files inside the message list.

**egui-Friendly Design Constraints**
- Prefer cards, strips, compact timelines, and inspectors over giant node canvases as the primary view.
- For swarm-scale data, render grouped summaries first and virtualize long tables/logs.
- Maintain ring buffers for live events/logs; only tail active selections.
- Downsample observer updates under load, matching the architecture's load-shedding model in `docs/architecture/state-server.md`.
- Avoid many constantly animating widgets; use subtle pulse/highlight only for state transitions.

**Data / View-Model Gaps**
- `apps/deer_gui/src/models.rs` currently exposes threads, messages, todos, suggestions, and artifacts, but not mission/workflow/agent/approval/event models.
- Add view models for `MissionSummary`, `WorkflowRun`, `AgentStatus`, `ApprovalRequest`, `AttentionEvent`, `ArtifactNode`, `LineageEdge`, and `LogTail`.
- Normalize event severity and type early so the UI can collapse noisy streams into digestible summaries.
- Carry the correlation identifiers from the architecture doc through every inspectable item.

**Visual Direction**
- Keep it light and readable, but shift from cozy chat to scientific control-room.
- Palette: parchment/stone base, slate text, oxidized teal for live, amber for approvals, vermilion for failures, moss for complete.
- Typography in `egui` terms: stronger hierarchy, tighter labels, tabular numerics for counters/timestamps.
- Use shape language to signal meaning: rounded mission cards, sharper diagnostic panels, outlined trace/log surfaces.

**Phased Plan**
- Phase 1: reframe current thread UI into `Mission Command Deck` without changing backend behavior much.
- Phase 2: add `Attention Queue`, `Watchlist`, grouped `Swarm Monitor`, and contextual right inspector.
- Phase 3: add `Experts Meeting` mode and richer context pinning for files/images/artifacts.
- Phase 4: add lineage/timeline/forensics surfaces using the event correlation model.
- Phase 5: optimize for scale with virtualization, sampling, grouped event aggregation, and log tail controls.

**Concrete First Implementation Pass**
- Replace left `Threads` panel in `apps/deer_gui/src/app.rs` with `Missions + Views`.
- Split center panel into tabbed modes: `Deck`, `Meeting`, `Timeline`, `Artifacts`, `Forensics`.
- Convert the current task board into an `Attention Queue` with grouped states and actions.
- Turn the right artifact panel into a general `Inspector` that can show agent, artifact, approval, or log details.
- Add a bottom `Live Events` strip with severity filters and auto-silencing.
- Keep the composer, but elevate it into a mission command bar with mode presets like `Research`, `Orchestrate`, `Debate`, `Investigate`.