# Post-Discovery Build Order Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Define the implementation order that becomes valid only after discovery-first contracts are frozen.

**Architecture:** This file is a bridge from discovery to runtime implementation. It does not add new architecture; it sequences the approved slices so implementation remains toolkit-first, storage-aware, and generator-agnostic.

**Tech Stack:** Approved spec set, approved library decision matrix.

---

## Corrected Order

1. foundation contracts/domain metadata envelopes
2. raw envelope and normalizer crates
3. canonical record families and lineage-aware replay base
4. derivations/read-models with level/plane metadata
5. retrieval-aware and view-tier-aware reusable surfaces
6. proof apps for chat/replay/graph/design
7. world projection and spatial views
8. thin `deer_gui` composition

## No-Go Rule

Do not execute this build order until the discovery-first spec tranche through
`docs/superpowers/specs/modular-toolkit/31-custom-code-boundaries.md` is approved.
