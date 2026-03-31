# Hybrid Data Game Design

**Date**: 2026-03-31
**Project**: `apps/deer_gui`
**Goal**: Turn agentic-AI orchestration data into a quickly playable hybrid RTS/RPG experience using Bevy + egui, with immediate visual payoff and minimal asset dependency.

## 1. Product Shape

The first playable game is not two games.

It is:
- **RTS by default** for macro orchestration, monitoring, queue management, and world state
- **RPG at intervention points** for agent inspection, live meetings, critical branching choices, and artifact/knowledge review

Core rule:
- **90% RTS shell, 10% RPG puncture points**

This keeps the design coherent, keeps scope under control, and still delivers the emotional payoff of character-driven moments.

## 2. What We Already Have

The backend already provides the most important thing a game needs: meaningful state.

Available game content from backend truth:
- chat sessions
- live meetings
- meeting replays
- agent orchestrations
- execution traces
- metrics
- artifacts produced
- current live agent progress
- agent health/history/performance/work queues/track record
- knowledge graphs
- research topics
- references
- insights
- ideas
- concepts
- charts
- audio
- video
- branching and replayable history
- stress scenarios with failure modes and recovery patterns from `docs/architecture/stess-tests.md`

This means the first game does not need invented missions from scratch. The system behavior is already the mission source.

## 3. What We Do Not Have

We do not currently have:
- custom 3D ships, characters, or buildings
- authored environment art
- mature animation library for narrative scenes
- bespoke VFX and SFX packs
- handcrafted UI icon set covering the full game vocabulary
- authored story campaign content

That is acceptable for v1. The first slice should be designed to succeed without them.

## 4. Design Strategy

The first version must be built from four principles:

1. **Backend truth is the content**
   - We do not fake a world first and connect data later.
   - The world simulation is a stylized rendering of real orchestration and storage events.

2. **Procedural first**
   - Use particles, vector shapes, shader backgrounds, generated portraits/cards, and text-first diegetic UI.
   - Avoid waiting on art production.

3. **One simulation, many views**
   - RTS and RPG views are different presentations of the same backend object graph.
   - An agent is still the same agent whether shown as a fleet unit or a companion.

4. **Playable in one screen quickly**
   - The first slice should feel good before any large world, campaign, or asset pipeline exists.

## 5. Core Fantasy

The player is a **Commander-Captain** operating an AI operations vessel.

At macro scale:
- they command sectors, swarms, queues, research, and event responses

At micro scale:
- they intervene in key agents, conversations, meetings, and decisions

Macro fantasy references:
- Homeworld 2
- Sins of a Solar Empire
- StarCraft II control logic

Micro fantasy references:
- Mass Effect bridge/companion interactions
- BG3-style inspection, tooltips, and character sheets

## 6. Minimal Canonical Simulation Model

The game must begin with one canonical model shared by all views.

### World Objects

| Backend Truth | Canonical Game Object | RTS Presentation | RPG Presentation |
| --- | --- | --- | --- |
| Agent | `UnitActor` | drone, corvette, specialist ship | companion, specialist operative |
| Agent orchestration | `TaskForce` | fleet, wing, squad | party formation, coordinated skill plan |
| Chat session | `ConversationNode` | command channel | dialogue scene |
| Meeting | `CouncilEvent` | war room session | bridge/camp briefing |
| Trace | `ActionTimeline` | mission trail | quest journal |
| Artifact | `WorldItem` | module, upgrade, relic | loot, codex item, crafted gear |
| Knowledge graph | `KnowledgeNetwork` | sector graph, tech tree | codex graph, clue board |
| Queue | `ProductionFlow` | build/refinery queue | ritual/crafting/action queue |
| Metrics | `TelemetryState` | fleet telemetry | character stats / squad condition |
| Replay | `ReplayRecord` | battle replay | memory replay |

### Simulation Rule

No separate RTS-only or RPG-only data model is allowed in v1.

Every screen must derive from the canonical objects above.

## 7. First Playable Vertical Slice

The first vertical slice must be intentionally tiny.

### Slice Name

**Operation Glass Harbor**

### Slice Contents

- 1 star-map / operations scene
- 3 agent classes
- 3 active work queues
- 1 live event feed
- 1 minimap / sector graph
- 1 artifact output path
- 2 stress/failure events
- 1 meeting/intervention drill-down
- 1 agent profile / companion view
- 1 replay inspector

### Agent Classes

| Class | Backend Role | RTS Fantasy | RPG Fantasy |
| --- | --- | --- | --- |
| Researcher | analysis, synthesis, insight generation | science corvette | analyst mage / scholar |
| Operator | orchestration, branching, queue management | command frigate | tactician / officer |
| Harvester | collection, storage, artifact movement | resource drone / transport | scavenger / technician |

### Slice Loop

1. Player observes macro state in RTS command view.
2. They assign or rebalance agent groups.
3. Live queue progress produces artifacts and events.
4. A failure or ambiguity appears.
5. Player drills into one RPG-style intervention scene.
6. Their choice modifies the macro simulation.
7. A new artifact / insight / unlock is generated.
8. Player reviews replay or knowledge update.

If this loop is fun, the project is viable.

## 8. What Makes It Feel Like A Game Quickly

Immediate results come from high-feedback systems, not asset count.

We should prioritize:
- moving units
- visible progress bars
- event bursts
- failure states with consequence
- unlocks/promotions
- camera transitions between macro and micro
- satisfying sound and shader feedback

We should defer:
- cinematic cutscenes
- handcrafted 3D assets
- lore-heavy campaign writing
- large tech trees
- broad faction system

## 9. Asset Strategy

### Phase A: Procedural / Programmatic Only

Use:
- shader-driven space backgrounds
- Hanabi particles for swarms, data trails, pings, explosions, queue completion effects
- vector/egui drawn HUD shapes
- SVG-generated icons
- generated portrait cards with symbolic class imagery
- geometric ships built from primitive meshes
- minimap and graph views drawn procedurally

This phase should be enough for the first playable slice.

### Phase B: Open-Source Enhancement

After the loop works, selectively add:
- CC0 / permissive icon packs
- CC0 ambient / sci-fi SFX
- public-domain or permissive fonts if needed beyond current choices
- simple open-source mesh kits for ships, drones, props, and skyboxes

Acceptance rule:
- only bring in an open asset if it improves an already-working interaction
- never block a feature on finding art

### Phase C: Custom Assets Later

Only after the systems are proven:
- custom unit silhouettes
- custom faction look
- bespoke VFX
- authored portraits / characters
- premium sound pass

## 10. Visual Pipeline

### Runtime Presentation Layers

| Layer | Purpose | Initial Build Method |
| --- | --- | --- |
| Background scene | atmosphere and scale | WGSL shader + procedural starfield + particles |
| World actors | agents, queues, resource nodes | primitive meshes + emissive materials + icons |
| World overlays | routes, alerts, progress, selection | Bevy gizmos + custom painter overlays |
| HUD | top bar, command console, event feeds | egui |
| Drill-down screens | profile, meeting, replay, artifact | egui modal / overlay with RPG framing |

### Camera Model

- **Macro camera**: strategic RTS overview
- **Micro camera**: modal drill-down / profile / meeting framing
- **Transition rule**: camera moves or panel transitions must imply the user is zooming into the same truth, not loading a different mode

## 11. Data Pipeline

### Input Pipeline

`backend events -> canonical game state -> view adapters -> RTS/RPG presentation`

### Recommended Layers

| Layer | Role |
| --- | --- |
| Adapter layer | converts backend objects/events into canonical simulation objects |
| Simulation layer | owns unit states, queue states, event states, unlocks, consequences |
| Presentation layer | renders RTS HUD, world, and RPG drill-downs |
| Replay layer | records deterministic state deltas or structured event history |

### Rule

Backend integration should initially support both:
- **live mode** from real backend events
- **recorded mode** from fixture/replay files

Recorded mode is mandatory for rapid iteration, demos, and testing.

## 12. Gap-Filling Plan

### What The Backend Already Solves

- meaningful live events
- rich object relationships
- progress and health semantics
- artifact lineage
- branching/replay truths
- failure and recovery scenarios

### What The Game Layer Must Add

- spatial metaphor
- feedback timing
- promotion / consequence logic
- interaction verbs
- readable visual hierarchy
- narrative framing for intervention moments

### Gaps And Solutions

| Gap | Best Initial Solution |
| --- | --- |
| No ship art | primitive mesh silhouettes + emissive shader IDs |
| No character art | symbolic portrait cards + class badges + nameplates |
| No level art | abstract space sector / command room backgrounds |
| No cutscenes | panel transitions + camera zooms + sound cues |
| No authored quests | stress-test scenarios + backend events as missions |
| No economy system | derive economy from throughput, backlog, artifacts, research unlocks |
| No combat assets | represent conflict as queue pressure, corruption, contention, and sector instability |

## 13. Interaction Verbs

The first slice should support only a small, strong verb set.

### Macro Verbs

- assign
- reroute
- prioritize
- pause
- boost
- quarantine
- inspect
- replay

### Micro Verbs

- approve
- reject
- persuade
- interrupt
- promote
- equip
- branch

If a feature does not use these verbs, it is probably too early.

## 14. Agile / XP Delivery Plan

### Milestone 0: Toy Backbone

Goal:
- display one fake sector with fake agents and fake queues

Done when:
- units move
- top HUD updates
- one queue progresses
- one event appears

### Milestone 1: Real Hybrid Proof

Goal:
- connect one real backend-backed flow to the toy simulation

Done when:
- one real session appears as a playable mission
- one agent becomes a selectable actor
- one trace becomes replayable

### Milestone 2: RPG Puncture

Goal:
- one macro event opens one meaningful micro intervention

Done when:
- clicking a critical event opens an agent profile / meeting drill-down
- user makes a choice
- choice affects macro state

### Milestone 3: Artifact And Knowledge Reward

Goal:
- actions generate inspectable outputs and progression

Done when:
- an artifact is produced
- it unlocks a visible research/knowledge node
- replay and provenance are visible

### Milestone 4: Failure Drama

Goal:
- system stress becomes gameplay

Done when:
- at least two stress scenarios from `docs/architecture/stess-tests.md` produce compelling game events
- player can recover, reroute, or sacrifice something

## 15. Recommended First Stress Scenarios

Use these first because they are dramatic and visually legible.

1. **Scenario 1: 10,000 Agent Burst**
   - Great for swarm spectacle and queue pressure
2. **Scenario 2: HOTL Media Pipeline**
   - Great for intervention gameplay and artifact output
3. **Scenario 6: S3 Slow Down**
   - Great for logistics disruption and recovery decisions

These three alone are enough for the first compelling prototype.

## 16. Non-Goals For V1

Do not build these yet:
- full faction system
- large open world
- hand-authored campaign
- inventory-heavy RPG system
- full combat simulation
- multiplayer
- elaborate crafting tree
- custom asset editor before the first slice works

## 17. Technical Plan Summary

### Runtime

- Bevy for world simulation, camera, VFX, and rendering
- egui for HUD, panels, drill-downs, inventory/profile/replay surfaces
- fixture-driven replay mode alongside live backend mode

### Content

- procedural world + shader backgrounds
- data-driven mission/event fixtures
- SVG/icon/text-based symbolic asset system

### Architecture

- backend adapters feed canonical simulation state
- canonical state drives both RTS and RPG presentation
- replay/fixtures allow deterministic iteration

## 18. Recommendation

Build the hybrid game as:

- **one command-center screen**
- **one canonical simulation**
- **one tiny playable mission loop**
- **one meaningful RPG intervention path**
- **procedural visuals first**

That is the fastest route to something real, impressive, and extensible.
