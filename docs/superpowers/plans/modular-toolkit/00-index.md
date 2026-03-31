# Modular Toolkit Discovery-First Plan Index

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Freeze the discovery-first contract tranche so later runtime implementation can proceed with clear storage-aware, generator-agnostic, trajectory-aware inputs and outputs.

**Architecture:** Planning now begins with architecture alignment, ontology design, lineage and trajectory contracts, retrieval/view/world contracts, and library/build-order freeze. Only after this discovery tranche is approved should crate-level implementation planning resume.

**Tech Stack:** Markdown specs/plans, `docs/architecture/state-server.md`, generator research inputs, existing repo/runtime references, and approved OSS decision inputs.

---

## Non-Negotiable Inputs

Read before implementation:

- `docs/superpowers/01-current-state.md`
- `docs/superpowers/02-rules-in-force.md`
- `docs/superpowers/specs/modular-toolkit/09-non-negotiables.md`
- `docs/superpowers/specs/modular-toolkit/10-planning-guardrails.md`

## Plan Files

| File | Purpose | Depends On | Parallel Lane |
| --- | --- | --- | --- |
| `01-architecture-alignment.md` | align toolkit plans to state-server truth, mediated reads, and intents | — | sequential |
| `02-data-ontology-and-canonical-contracts.md` | freeze canonical record families, level/plane occupancy, identity, and lineage metadata | 01 | sequential |
| `03-generator-and-normalizer-mapping.md` | freeze raw envelope families, generator capability, and promotion rules | 02 | lane A |
| `04-retrieval-view-and-world-contracts.md` | freeze retrieval/index contracts, entity view matrix, and world object families | 02 | lane B |
| `05-library-and-build-order-freeze.md` | freeze OSS choices, custom-code boundaries, and corrected post-discovery build order | 03, 04 | lane C |
| `06-post-discovery-build-order.md` | bridge from discovery into later runtime implementation sequence | 05 | final |

## Practical Success Definition

This tranche succeeds only when all of the following are true:

- state-server alignment is explicit
- L0-L5 and tri-plane semantics are attached to canonical contracts
- lineage, correlation, and trajectory rules are explicit
- entity view tiers and world object families are explicit
- generator mapping is no longer DeerFlow-only
- OSS choices and custom-code boundaries are frozen before implementation

## Definition Of Done

Do not mark the discovery tranche done unless it has:

- approved discovery specs for alignment, ontology, mapping, retrieval, views, and build order
- explicit success and no-go conditions before runtime implementation
- updated planning guardrails that block premature coding

## Library Policy

Use existing, battle-tested libraries first when later implementation resumes:

- use DeerFlow's Python client and current bridge transport before inventing a new Rust protocol client
- use `petgraph` for graph structures and traversal helpers before rolling a graph core
- use `egui_dock` for docking/layout primitives before custom docking
- use existing Bevy plugins already in the repo before new rendering infrastructure

Only replace or supplement a library when:

- a required capability is missing
- performance evidence shows it is necessary
- the replacement is documented in the relevant plan task before code is written

## Global Verification

After each discovery plan file is completed, verify the required docs exist and are linked from the readiness checklist:

```bash
ls docs/superpowers/specs/modular-toolkit
```

Expected result:

- the new discovery docs exist
- the readiness checklist points to them
- crate implementation remains blocked until this tranche is approved

## Drift Prevention Checklist

Before starting any later implementation task, answer:

1. Which level and plane does this data inhabit?
2. What is the source -> transform -> destination trajectory?
3. Which generator families can populate this contract?
4. Which existing library or framework can solve most of it?
5. Why is this not debuting inside `deer_gui`?

If any answer is unclear, stop and refine the task before coding.
