# Design: Modular Toolkit - Shell Mode Support Matrix

**Date:** 2026-04-01
**Status:** Accepted for planning


- [Contracts and Traits](./37-shell-mode-support-matrix/01-contracts-and-traits.md)
- [Metadata and Dependencies](./37-shell-mode-support-matrix/02-metadata-and-dependencies.md)
- [Canonical Modes Intro](./37-shell-mode-support-matrix/03-canonical-modes-intro.md)
- [Mode: Control Center](./37-shell-mode-support-matrix/04-mode-control-center.md)
- [Mode: Battle Command](./37-shell-mode-support-matrix/05-mode-battle-command.md)
- [Mode: Live Meeting](./37-shell-mode-support-matrix/06-mode-live-meeting.md)
- [Mode: Artifact Analysis](./37-shell-mode-support-matrix/07-mode-artifact-analysis.md)
- [Mode: Replay](./37-shell-mode-support-matrix/08-mode-replay.md)
- [Mode: Forensics](./37-shell-mode-support-matrix/09-mode-forensics.md)
- [Support Judgment Rules](./37-shell-mode-support-matrix/10-support-judgment-rules.md)

## Reconciled Shell Entry Rule

Shell modes that accept linked or shell-local command carry-in must treat that
carry-in as shell-local `prefill_seed` only.

Promotion into editable `prefill`, operator-owned `draft`, validated intent, and
explicit `submitted` remains governed by the lifecycle in
`20-intent-and-mediated-read-model.md`.
