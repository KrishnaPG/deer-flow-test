## Why This File Exists

`40-shell-and-linked-interaction-stress-test.md` showed that the current design
line is close, but not yet planning-ready.

This file is the final integration tranche that closes the remaining gaps across:

- world-primary shell support
- temporal, streaming, and failover semantics
- policy and access overlays
- intent lifecycle boundaries

## Pinned Rule

- planning readiness requires these concerns to be integrated into the shell
  model, not left as future implementation detail
- world-primary shells are peers to center-workspace shells, not optional visual
  flourishes
- shell-local linked state must be deterministic under failover, exclusion, and
  replay conditions
- linked gestures may create or refresh shell-local `prefill`, but may not
  silently execute work

