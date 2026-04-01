# RTS HUD Spec Revision Summary

- Revised Section 6 so HUD components consume read-only State Server projections and emit intents instead of implying local ownership of mission state.
- Updated Section 7 animation rules to be sequence-aware, idempotent, and rehydration-safe, with explicit suppression of one-shot visuals during replay catch-up.
- Replaced Section 8 with a projection-driven State Server model that defines snapshot/delta feeds, local projection resources, replay cursors, and idempotent reducers.
- Added a new intent boundary and pending-confirmed-rejected command lifecycle section so optimistic HUD feedback stays storage-native and rollback-safe under conflicts.
- Preserved the original visual/layout direction, panel structure, and interaction feel while aligning the spec with storage-native State Server mandates.
