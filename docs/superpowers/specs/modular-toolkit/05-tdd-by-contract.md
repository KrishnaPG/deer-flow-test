# Design: Modular Toolkit - TDD By Contract

**Date:** 2026-03-31
**Status:** Draft revision

## Testing Position

TDD happens at the reusable module boundary, not at the final app screen.

The sequence is:

1. define the contract
2. write a failing fixture, contract, or golden test
3. implement the minimum module behavior
4. add interaction or state-behavior coverage
5. prove the module in one narrow tool app
6. only then compose it into `deer_gui`

## Test Pyramid

| Module Type | Primary Test Type |
| --- | --- |
| contracts and domain | pure unit tests |
| raw sources | fixture and protocol tests |
| normalizers | fixture transform tests |
| derivations | deterministic golden or snapshot tests |
| read models | reducer and state-machine tests |
| reusable views | behavior tests and snapshots where feasible |
| panel/layout runtime | interaction and serialization tests |
| app composition | thin integration tests |

## What "Through Tools" Means

The tool app is part of the testing story.

For example:

- chat transcript and composer modules are test-driven in their own crates, then
  proved inside `apps/deer_chat_lab`
- timeline module is test-driven in its own crate, then proved inside
  `apps/deer_replay`
- graph module is test-driven in its own crate, then proved inside
  `apps/deer_graph_lab`
- layout runtime is test-driven in its own crate, then proved inside
  `apps/deer_design`

That means the tool app is not replacing tests. It is the narrow integration
surface that proves the reusable module can survive real use.

## Minimum Acceptance For A Reusable Module

A module is ready for wider composition only when it has:

- typed contract definitions
- representative fixtures
- passing behavior tests
- a tool-app proof path
- no direct dependency on raw backend schema or app-specific fantasy types

For chat-oriented modules, acceptance should also include:

- stream fixture coverage for message, tool, and custom progress events
- artifact access coverage for both persisted and in-flight outputs
- clarification interrupt coverage

## Anti-Patterns

Avoid:

- views reading raw backend payloads
- panels owning normalization logic
- backend adapters importing Bevy or egui UI code
- app enums leaking into reusable crates
- tool apps forking runtime components
- building one giant `ui_core` dumping ground

## Review Question

For every major feature, ask:

- what is the reusable module?
- what is the contract?
- what is the failing test first?
- which tool app proves it before `deer_gui` uses it?

If there is no answer to those four questions, the feature is not yet on the
right path.
