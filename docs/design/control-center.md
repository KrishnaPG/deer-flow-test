# Control Center UX/DX Plan

## Objectives
- Replace chat-centric flows with a spatial/temporal control center where users define problems, spawn/orchestrate agents, monitor progress, and review outcomes.
- Surface widgets contextually in a 3D-inspired environment that reacts to agent and task state rather than linear conversation transcripts.

## Reference Technology Review
- **Tambo (MIT, React/TypeScript)** – Schema-driven generative components, MCP integration, local browser tools, persistent interactables, suggestion/context helpers, multi-provider routing. Blueprint for component registry, tool wiring, and streaming orchestration.
- **assistant-ui (MIT, React/TypeScript)** – Composable primitives for streaming, retries, attachments, approvals, analytics, keyboard/a11y. Informs structuring primitives (panes, inspectors, palettes) detached from chat bubbles.
- **tambo-template (MIT, Next.js)** – Production-grade starter showcasing mission-control layout, MCP modals, voice input, thread history, interactable panels. Serves as architectural reference for hybrid dashboard/chat orchestration.
- **egui (MIT/Apache-2.0, Rust)** – Immediate-mode GUI supporting native + Web builds, custom painting, 3D embedding, dynamic widget spawning, AccessKit. Ideal for the adaptive control-center environment layered over 3D canvases.

## Core Concepts to Adopt
1. **Schema Registry** – Define AI-controllable widgets as schemas (serde/JSON Schema) so agents can request widget instantiation via tool calls. Registry maps requests to React (Tambo/assistant-ui) or Rust (egui) renderers.
2. **Streaming Orchestration** – Maintain per-thread state in backend (DeerFlow). Borrow assistant-ui/Tambo streaming semantics for incremental props, retries, cancellations, and reflect updates in spawned panels.
3. **Tool & MCP Surfaces** – Provide MCP browsers, elicitation forms, and local-tool overlays inspired by Tambo Template, rendered with egui for lightweight contextual widgets.
4. **Context Hooks & Suggestions** – Feed scene metadata (selected agents, workspace focus, objective) into prompts; surface auto-suggestions or command palette entries as floating actions when relevant.
5. **3D Spatial Layout** – Central 3D canvas shows agents/tasks/resources. Floating/docked panels spawn near relevant nodes, lifecycle tied to agent progress (spawn on start, collapse on completion).
6. **DX Foundations** – Offer shared schema bindings in TypeScript and Rust so all surfaces honor same contracts. Document backend APIs (thread management, streaming protocol, tool schema) for future implementers.

## Next Steps
1. Finalize shared schema/contract between DeerFlow agents and UI surfaces (JSON Schema choice, serialization format, tool naming conventions).
2. Draft UX flows for the control center (problem definition, agent spawning, monitoring, review) referencing the reviewed toolkits but mapping onto spatial/panel layouts.
3. Outline developer experience story: CLI/project templates (TS for web, Rust for desktop), dev server with hot reload, mocked agent streams for local testing, and integration guide for the DeerFlow backend.
