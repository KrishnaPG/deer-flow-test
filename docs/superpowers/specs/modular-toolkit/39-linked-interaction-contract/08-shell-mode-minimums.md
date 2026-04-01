## Shell Mode Minimums

Every shell mode that declares linked behavior must define a broker for:

- `selection`
- `focus`

Additionally, a shell mode must define a broker for:

- `filtering`, only where the shell mode declares shared filter behavior
- `replay_cursor` and `temporal_range`, where temporal reasoning matters
- `command_target`, where the shell mode is actionable
- `camera_sync` and `viewport_navigation`, where the shell mode is world-primary

Every required panel in a shell mode must participate as at least a `source` or
`sink`.

Purely decorative required panels are not allowed.

Minimal required loops:

- every inspectable shell mode must link current selection into `InspectorPanel`
- every actionable shell mode must link current selection and/or pinned context
  into its declared command surface
- every artifact-bearing shell mode must support artifact selection, pinning, and
  drill-down propagation across participating panels
- every temporally reasoning shell mode must support replay-style cursor or range
  propagation, even if the visual host is an ordered event list rather than a
  rich replay timeline

