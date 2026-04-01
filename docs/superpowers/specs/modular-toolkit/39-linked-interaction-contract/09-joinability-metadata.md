## Joinability And Metadata Requirements

Linked interaction support requires all of the following where the shell mode
declares them:

- stable canonical join keys across participating views
- normalized filter dimensions with compatible value semantics
- `IdentityMeta`
- `CorrelationMeta`
- `LineageMeta`
- level and plane where semantic cross-view filtering depends on them
- stable timestamps or sequence IDs where temporal brushing or replay alignment
  is required
- broker epoch or equivalent recovery sequence where linked recovery is
  supported
- freshness timestamp or last applied `sequence_id`
- explicit stale reason and staleness budget where live streams or queue state
  are shown
- `policy_epoch` and `policy_reason` where policy overlays may invalidate
  linked state
- exclusion or tombstone markers where objects may disappear from direct access
- spatial anchors and viewport target IDs where world/minimap linkage is
  required
- drill-down targets and provenance backlinks for escalated detail
- lifecycle and supersession semantics where linked context may outlive the
  source surface

Labels, display order, screen position, and styling are never valid join keys.

