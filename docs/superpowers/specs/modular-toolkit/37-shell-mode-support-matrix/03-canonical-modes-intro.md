## Canonical Shell Modes

The shell should be judged by operating modes before it is judged by exact
screen geometry.

Mode interpretation rule:

- each shell mode below may list only the mode-specific requirements and deltas
  that are necessary to distinguish it
- any shell mode that declares a linked interaction path inherits the default
  broker, metadata, degradation, and support rules from
  `39-linked-interaction-contract.md` unless this file tightens them
- any shell mode that exposes temporal, failover, policy-overlay, or intent-
  boundary behavior inherits the default rules from
  `41-final-integration-tranche.md` unless this file tightens them
- required canonical record families, metadata, mediated reads, intents, and
  broker ownership may therefore be satisfied by explicit mode-local detail or
  by normative inheritance from the required views, panels, and referenced
  shell-level contracts
- when a shell mode introduces materially different or stricter behavior, it
  must state that behavior inline

