# Control Center Wireframes (Textual)

## Mission Overview Board

```
┌───────────────────────────────────────────────────────────────────────────────┐
│ Objective Stack        |                     3D Ops Canvas                    │
│ ┌─────────────┐        |  ┌───────────────────────────────────────────────┐  │
│ │ OBJ-42      │        |  │   • Agent Swarm (status color)               │  │
│ │ Revenue AI  │        |  │   ○ Data Lake Node                          │  │
│ │ due 16:00   │        |  │  /  Dependency edges                        │  │
│ ├─────────────┤        |  │  Floating widgets: progress, alerts         │  │
│ │ OBJ-77 ...  │        |  └───────────────────────────────────────────────┘  │
│ └─────────────┘        |                                                     │
│ Filters / Personas     | Event Ticker (Right Rail)                          │
│ [Operator] [Builder]   | ┌──────────────────────────────┐                    │
│ [Reviewer]             | │ 14:05 Agent Synapse spawned  │                    │
│ Search + Tag clouds    | │ 14:07 Tool "vectorize" run   │                    │
└────────────────────────┴─┴──────────────────────────────┴────────────────────┘
```

## Agent Forge Panel

```
┌──────────────────────────────┬───────────────────────────────────────────────┐
│ Template Catalog             │ Agent Blueprint Inspector                     │
│ ┌─────────────┐              │ Name: Synapse-Beta                            │
│ │ Data Scout  │ drag → drop  │ Tabs: Skills | Data Feeds | Guardrails        │
│ │ Synthesizer │────────────► │                                               │
│ │ Negotiator  │              │ Skills Tab:                                   │
│ │ ...         │              │  [x] Vector search      Sensitivity slider     │
│ Filters, tags │              │  [ ] Negotiation kit    Retry/backoff config  │
│ Search bar    │              │                                               │
│               │              │ Launch Timeline Ribbon:                       │
│ Preview card  │              │  Stage 1 Config ▒▒▒▒                          │
│ with metrics  │              │  Stage 2 Deployment ▓▓▓▓▓                     │
└───────────────┴──────────────┴───────────────────────────────────────────────┘
```

## Monitoring Lens

```
┌──────────────────────────────┬───────────────────────────────────────────────┐
│ 3D Canvas (pan/zoom)         │ Inspector Stack                               │
│ ┌──────────────────────────┐ │ ┌───────────┐ ┌──────────┐ ┌────────────┐     │
│ │ Agent nodes, links,      │ │ │ Timeline  │ │ Tool Run │ │ Output Snap│     │
│ │ heatmaps, alert icons    │ │ │ (Gantt)   │ │ Log Table│ │ Gallery    │     │
│ │ Floating mini-widgets    │ │ └───────────┘ └──────────┘ └────────────┘     │
│ │ (log preview, actions)   │ │ Auto-focus on selected agent                  │
│ └──────────────────────────┘ │ Bottom: contextual quick actions (Pause, Replan│
└──────────────────────────────┴───────────────────────────────────────────────┘
```

## Review Deck

```
┌───────────────────────────────────────────────────────────────────────────────┐
│ Outcome Grid (cards)                                                          │
│ ┌───────┬───────┬───────┬───────┐                                            │
│ │Card A│Card B │Card C │Card D │  Quick metrics, owners, tags                │
│ └───────┴───────┴───────┴───────┘                                            │
│ Comparison Slot                                                               │
│ ┌─────────────┬─────────────┐                                                │
│ │Drop card    │Drop card    │  ► generates diff view below                   │
│ └─────────────┴─────────────┘                                                │
│ Diff Panel: Metrics chart | Narrative summary | Attachments pane             │
│ Annotation Sidebar (right): threaded notes linked to cards                   │
└───────────────────────────────────────────────────────────────────────────────┘
```

## Command Palette & Suggestions

```
                           ┌──────────────────────────┐
                           │⌘K  Type command...       │
                           │ Suggestions:             │
                           │  • Spawn Negotiator      │
                           │  • Add MCP source       │
                           │  • Escalate to Reviewer │
                           │ Voice ◉    Shortcuts ?  │
                           └──────────────────────────┘

Floating chips appear near focused elements:
 [Clone Agent] [Inspect Tool Logs] [Attach Dataset] (auto-hide after inactivity)
```
