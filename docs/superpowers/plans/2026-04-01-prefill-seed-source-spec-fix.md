# Prefill Seed Source Spec Fix Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Align the modular-toolkit shell specs so `prefill` may be entered by the broader approved seed-source concept without weakening the explicit-action guardrails.

**Architecture:** Update the spec line in place by introducing `prefill_seed` as a shell-level category in the linked-interaction contract, then make `41-final-integration-tranche.md` consume that category in lifecycle entry rules and accidental-execution guardrails. Tighten `37-shell-mode-support-matrix.md` only where mode declarations need to acknowledge seed-capable linked gestures beyond the inherited defaults.

**Tech Stack:** Markdown specs in `docs/superpowers/specs/`; git-based review workflow

---

### Task 1: Add `prefill_seed` to the linked-interaction contract

**Files:**
- Modify: `docs/superpowers/specs/modular-toolkit/39-linked-interaction-contract.md`
- Test: manual spec review in `docs/superpowers/specs/modular-toolkit/39-linked-interaction-contract.md`

- [ ] **Step 1: Write the failing test**

Use this checklist as the failing review test:

```text
- `39-linked-interaction-contract.md` defines `action_intent_emission` and `command_target`
- `39-linked-interaction-contract.md` does NOT yet define a shell-level `prefill_seed` category
- `39-linked-interaction-contract.md` does NOT yet state that `command_target` or declared linked gestures may seed `prefill`
```

- [ ] **Step 2: Run test to verify it fails**

Run: review `docs/superpowers/specs/modular-toolkit/39-linked-interaction-contract.md:286` and confirm the missing `prefill_seed` rule
Expected: FAIL because the file allows `command_target` degradation to prefilled draft context but does not define a general seed-source rule

- [ ] **Step 3: Write minimal implementation**

Add a short shell-level section immediately after `action_intent_emission` and before `command_target` using this text:

```md
### `prefill_seed`

Purpose:

- create or refresh shell-local `prefill` intent context without granting
  editable ownership or submit authority

Satisfied by:

- any declared `action_intent_emission` source
- any declared `command_target` source
- any explicitly allowed linked-gesture seed source declared by the shell mode

Allowed effect:

- create `prefill`
- refresh `prefill`
- update suggested target, scope, parameter, or provenance context

Forbidden effect:

- create `draft` automatically
- enter `validated`
- enter `submitted`
- append any canonical `IntentRecord`

Rule:

- `prefill_seed` remains shell-local only; explicit operator promotion is still
  required for `prefill -> draft`
```

Then add this bullet under the top-level linked interaction rules near the top of the file:

```md
- `prefill_seed` is broader than `action_intent_emission`; command-target and
  explicitly allowed linked-gesture paths may seed or refresh shell-local
  `prefill` without becoming editable or submittable intent state
```

Then add this bullet under `command_target` after the degradation rule:

```md
- declared `command_target` sources may seed or refresh `prefill`, but may not
  advance beyond `prefill` without explicit operator action
```

Then add this bullet under `viewport_navigation` after the degradation rule:

```md
- ordinary viewport navigation remains intent-neutral unless the shell mode
  explicitly declares a navigation gesture as `prefill_seed`-capable
```

- [ ] **Step 4: Run test to verify it passes**

Run: review `docs/superpowers/specs/modular-toolkit/39-linked-interaction-contract.md`
Expected: PASS because `prefill_seed` is now defined, `command_target` can seed `prefill`, and ordinary navigation stays intent-neutral by default

- [ ] **Step 5: Commit**

```bash
git add docs/superpowers/specs/modular-toolkit/39-linked-interaction-contract.md
git commit -m "docs: define prefill seed sources"
```

### Task 2: Update lifecycle entry rules in the final integration tranche

**Files:**
- Modify: `docs/superpowers/specs/modular-toolkit/41-final-integration-tranche.md`
- Test: manual spec review in `docs/superpowers/specs/modular-toolkit/41-final-integration-tranche.md`

- [ ] **Step 1: Write the failing test**

Use this checklist as the failing review test:

```text
- `41-final-integration-tranche.md` still says `prefill` may be entered by any declared `action_intent_emission` source
- the same file still allows linked gestures, replay scrubbing, world picks, camera moves, and attention interrupts to create or refresh `prefill`
- those two statements conflict because not all of those paths are `action_intent_emission` sources
```

- [ ] **Step 2: Run test to verify it fails**

Run: review `docs/superpowers/specs/modular-toolkit/41-final-integration-tranche.md:345`
Expected: FAIL because the lifecycle table is narrower than the guardrail bullets

- [ ] **Step 3: Write minimal implementation**

Replace the lifecycle table row:

```md
| `prefill` | any declared `action_intent_emission` source | non-submittable seed only |
```

with:

```md
| `prefill` | any declared `prefill_seed` source | non-submittable seed only |
```

Then insert this paragraph immediately before the lifecycle table:

```md
`prefill_seed` is satisfied by any declared `action_intent_emission` source,
any declared `command_target` source, and any explicitly allowed linked-gesture
seed source declared by the shell mode.
```

Then replace the first accidental-execution bullet:

```md
- linked gestures, brushing, replay scrubbing, world picks, camera moves, and
  attention interrupts may create or refresh `prefill` only
```

with:

```md
- declared `prefill_seed` paths, including linked gestures, brushing, replay
  scrubbing, world picks, explicit camera-linked seed gestures, and attention
  interrupts, may create or refresh `prefill` only
```

Then add this bullet immediately after it:

```md
- ordinary navigation-only camera movement remains intent-neutral unless the
  shell mode explicitly declares it as seed-capable
```

- [ ] **Step 4: Run test to verify it passes**

Run: review `docs/superpowers/specs/modular-toolkit/41-final-integration-tranche.md`
Expected: PASS because the lifecycle table and accidental-execution guardrails now use the same seed-source model

- [ ] **Step 5: Commit**

```bash
git add docs/superpowers/specs/modular-toolkit/41-final-integration-tranche.md
git commit -m "docs: align prefill lifecycle sources"
```

### Task 3: Tighten shell-mode expectations for seed-capable gestures

**Files:**
- Modify: `docs/superpowers/specs/modular-toolkit/37-shell-mode-support-matrix.md`
- Test: manual spec review in `docs/superpowers/specs/modular-toolkit/37-shell-mode-support-matrix.md`

- [ ] **Step 1: Write the failing test**

Use this checklist as the failing review test:

```text
- `battle_command` declares `command_target`, `viewport_navigation`, `camera_sync`, `replay_cursor`, and `temporal_range`
- `battle_command` does NOT yet explicitly state which linked gesture paths are allowed to seed `prefill`
- this leaves the mode relying on implication instead of an explicit seed-capable declaration
```

- [ ] **Step 2: Run test to verify it fails**

Run: review `docs/superpowers/specs/modular-toolkit/37-shell-mode-support-matrix.md:340`
Expected: FAIL because the shell mode lists linked interactions but not the stricter seed-capable subset

- [ ] **Step 3: Write minimal implementation**

Add this subsection inside the `battle_command` mode after `Required broker ownership map` and before `Required temporal semantics`:

```md
Required `prefill_seed` paths:

- all declared `action_intent_emission` sources
- all declared `command_target` sources
- world picks
- replay-anchor gestures from replay-capable surfaces
- attention interrupts that surface a candidate intervention context

Not seed-capable by default:

- ordinary camera pan
- ordinary camera zoom
- plain viewport repositioning without explicit command-context intent
```

Then add this support-judgment bullet near the shell-supported rules:

```md
- shell modes that rely on linked-gesture command seeding declare which paths are
  `prefill_seed`-capable and which remain intent-neutral
```

- [ ] **Step 4: Run test to verify it passes**

Run: review `docs/superpowers/specs/modular-toolkit/37-shell-mode-support-matrix.md`
Expected: PASS because `battle_command` now explicitly distinguishes seed-capable linked gestures from navigation-only interactions

- [ ] **Step 5: Commit**

```bash
git add docs/superpowers/specs/modular-toolkit/37-shell-mode-support-matrix.md
git commit -m "docs: declare shell prefill seed paths"
```

### Task 4: Cross-spec consistency review

**Files:**
- Modify: `docs/superpowers/specs/modular-toolkit/39-linked-interaction-contract.md`
- Modify: `docs/superpowers/specs/modular-toolkit/41-final-integration-tranche.md`
- Modify: `docs/superpowers/specs/modular-toolkit/37-shell-mode-support-matrix.md`
- Test: manual consistency review across all three files

- [ ] **Step 1: Write the failing test**

Use this checklist as the failing review test:

```text
- the same source class should not be legal in one file and forbidden in another
- `prefill` should remain shell-local and non-submittable in all files
- navigation-only camera movement should remain intent-neutral by default in all files
- `prefill -> draft -> validated -> submitted` should still require explicit operator action
```

- [ ] **Step 2: Run test to verify it fails**

Run: compare the three target files before final cleanup
Expected: FAIL if any wording drift remains around `camera moves`, `world picks`, `command_target`, or `prefill`

- [ ] **Step 3: Write minimal implementation**

If any wording drift remains, normalize to these phrases exactly:

```md
`prefill_seed`
shell-local `prefill`
explicit operator promotion
ordinary navigation-only camera movement remains intent-neutral by default
```

Do not add new concepts beyond those already defined in Tasks 1-3.

- [ ] **Step 4: Run test to verify it passes**

Run: re-read all three target files line-by-line
Expected: PASS because the three files now tell the same story about seed sources, operator promotion, and navigation neutrality

- [ ] **Step 5: Commit**

```bash
git add docs/superpowers/specs/modular-toolkit/37-shell-mode-support-matrix.md docs/superpowers/specs/modular-toolkit/39-linked-interaction-contract.md docs/superpowers/specs/modular-toolkit/41-final-integration-tranche.md
git commit -m "docs: reconcile prefill seed rules"
```

## Self-Review

- Spec coverage:
  - `prefill_seed` concept from `docs/superpowers/specs/2026-04-01-prefill-seed-source-design.md` is implemented in `39`
  - lifecycle entry rules and accidental-execution guardrails are aligned in `41`
  - shell-mode declaration expectations are explicit in `37`
- Placeholder scan:
  - no `TBD`, `TODO`, or deferred wording in this plan
- Type consistency:
  - uses `prefill_seed`, `command_target`, `action_intent_emission`, and
    `explicit operator promotion` consistently across all tasks
