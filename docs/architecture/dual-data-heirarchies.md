# The Dual Hierarchy

When it comes to AI Agents producing some outcome based on user inputs, *there are three interleaved data hierarchies*, not one.

Hierarchy A: Agent Orchestration/Execution
- What agents *did* — their lifecycle, metrics, decisions, state changes
- This is the "meta" layer: prompts issued, tools called, tasks completed, errors encountered
- L0: Raw agent events (tool calls, status pings, heartbeat ticks)
- L1: Sanitized agent activity records (normalized formats, deduped)
- L2: Agent-facing views (task queues, history rows, telemetry summaries, unit actors)
- L3+: Performance insights, fatigue models, veteran tier calculations

Hierarchy B: Generated Artifact Content
- What agents *produced* — the actual knowledge/artifacts
- This is the "content" layer: user prompts, gathered sources, summaries, final outputs
- L0: User prompts, raw source materials, raw tool outputs
- L1: System-transformed prompts, cleaned inputs, normalized payloads
- L2: Agent-gathered sources, retrieved context, intermediate views
- L3: Summarized insights, aggregated findings, knowledge extraction
- L4: Final artifacts (images, reports, code, datasets, models)
- L5+: Prescriptions, intervention plans based on artifacts
- L6: Outcome evaluations — did the artifact achieve its goal?

Hierarchy C: Game Presentation — how the data is *shown*
- This is the "view model" layer needed by Game UI — not raw data, not content, but *presentation*
- Consumes L2/L3/L4+ from A and B, projects into game-facing VMs
- Multiple presentation projections (views):
  - "Commander" mode: emphasizes Hierarchy A (what agents did)
  - "Researcher" mode: emphasizes Hierarchy B (what agents produced)
  - "Thread" mode: merges both (chat with agent metrics decorating each prompt/artifact)


The Interleaving:
- Hierarchy A *drives* Hierarchy B (agent actions produce artifacts)
- Hierarchy B *feeds back into* Hierarchy A (artifacts become inputs to subsequent agent actions)
- A single AI agent "thread" contains both hierarchies simultaneously
- The presentation consumes both Hierarchy A and B as L0, and depending on the viewer, projects either or both as L2; 
  - Thus the L2 data projections of Hierarchy C power the [Canonical Domain model](../superpowers/specs/modular-toolkit/02-workspace-and-layering.md) for Game UI;
  - also refer: [Canonical Record Layer](../superpowers/specs/modular-toolkit/07-chat-and-artifact-pipeline.md) and [Canonical Model Rules](../superpowers/specs/modular-toolkit/18-generator-mapping-matrix.md)


## The Key Insight About Levels
	
	L2 is not absolute — it's consumer-relative.

E.g. "Generator L4" (final artifact) becomes "Game UX L2" (presentation-ready) when it's ready to be shown. The "level" describes readiness for a particular consumer, not an intrinsic property of the data. This means:
  - ArtifactRecord at L4 in Hierarchy B = the final generated image/report
  - That same artifact, when projected for Game UI = L2 presentation data
  - The normalizer doesn't "promote" it from L4 to L2 — it reinterprets L4 content as L2 presentation