# Runtime And Operations Mapping

This file maps live operational data into RTS and RPG concepts.

## Core Runtime Data

| Data Point | RTS Mapping | RPG Mapping | UI / Interaction Notes |
| --- | --- | --- | --- |
| Chat sessions | Command channels between flagship and fleets | Dialogue scenes or party conversations | Open as tactical comms in RTS; open as cinematic conversation logs in RPG |
| Agent orchestrations | Fleet formations, task forces, carrier-issued orders | Squad tactics, party formations, coordinated abilities | Show orchestration scope as fleet radius in RTS; as party plan cards in RPG |
| Execution traces | Flight paths, mission trails, convoy routes | Quest logs, action journals, investigation timelines | Trace drill-down should always support replay, filtering, and branching |
| Metrics | Empire telemetry, sector health, production rates | Character stats, squad morale, stamina, encounter difficulty | Use trend arrows, buffs/debuffs, and color-coded deltas |
| Current live agent progress bars | Build bars, warp-charge bars, refinery completion | Cast bars, channeling bars, ritual completion | Live bars should pulse and support interruption / boost actions |
| Agent health | Hull integrity, shield level, combat readiness | HP, shield, fatigue, wound state | Health must support threshold color shifts and recovery actions |
| Agent history | Battle record, deployment history, campaign medals | Biography, quest history, alignment ledger | Make long-term history feel prestigious, not like raw logs |
| Agent performance | Combat efficiency, harvest yield, command rank | DPS / utility rating, proficiency, loyalty, mastery | Performance should unlock promotions, perks, and trust badges |
| Agent work queues | Build queues, logistics queues, docking schedules | Skill queue, crafting queue, quest backlog | Reordering the queue is a core player action in both modes |
| Agent track record | Veteran status, ace pilot ledger, fleet commendations | Character reputation, renown, class specialization history | Track record should change portrait treatment and available actions |
| Agent health history | Damage log, maintenance history | Injury log, trauma log, resilience profile | Useful for prediction of future failures and intervention hints |
| Agent performance history | Campaign performance timeline | Level progression, stat growth graph | Visualize as rank stripes or skill-tree growth arcs |
| Agent queue pressure | Supply congestion, dock overload | Mental load, over-encumbrance, over-commitment | Queue pressure should manifest as warning states before failure |

## Session And Control Surfaces

| Data Point | RTS Mapping | RPG Mapping | UI / Interaction Notes |
| --- | --- | --- | --- |
| Session list | Active theaters / sectors | Active missions / chapters | Sessions should appear spatial in RTS and narrative in RPG |
| Session branching | Alternate battle plans, split fleets | Alternate quest branches, dialogue forks | Branches should be first-class, not hidden in history |
| Session ownership | Fleet commander assignment | Party lead / companion ownership | Ownership should affect authority and available commands |
| Session state | Peace / combat / hyperspace / lockdown | Exploration / combat / dialogue / camp | State drives layout, HUD density, and soundscape |
| Operator override | Direct command override, tactical pause | Interrupt, charm, intimidate, manual intervention | Override actions should feel powerful and deliberate |
| HITL intervention | Hero unit direct-control mode | Timed dialogue choice or combat reaction | Reserve strongest visual feedback for these moments |
| HOTL review | Refinery checkpoint, production inspection | Ritual review, analyst checkpoint | HOTL should feel supervisory, not fully manual |
| Multi-agent parallel work | Drone swarm allocation | Party split-task assignment | Parallelism should be visible as separate squads / subparties |

## Recommended Interaction Layer

| Concern | RTS Recommendation | RPG Recommendation |
| --- | --- | --- |
| Default camera fantasy | Commander above the map | Captain inside the story |
| Best default metaphor | Logistics and fleet control | Companions and relationships |
| Best surface for live progress | Production console / fleet panel | Character action bars / cast bars |
| Best surface for trace inspection | Tactical replay | Quest journal replay |
| Best emotional framing for failures | Supply chain disruption or destroyed units | Companion injury, betrayal, or failed quest |

## Notes

- Operational data works best when it is never shown as a naked table first.
- In RTS mode, think in terms of command radius, formations, build queues, and strategic overlays.
- In RPG mode, think in terms of party trust, class roles, dialogue consequences, and journals.
- The same backend object should support both views without changing the underlying state model.
