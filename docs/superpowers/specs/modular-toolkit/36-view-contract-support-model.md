# Design: Modular Toolkit - View Contract Support Model

**Date:** 2026-03-31
**Status:** Draft revision

## Why This File Exists

Backend support should be judged by what the final UI can actually support.

That means support must be measured from views upward, not from raw backend nouns
downward.

## Pinned Rule

- `view contract` is the primary unit of backend support
- `panel` support is derived from supported hosted views
- hosted views are not semantic silos once co-present in the same panel set or
  layout
- `layout` support is derived from supported required panels plus required
  cross-view interaction contracts

So support is evaluated as:

`backend -> canonical mappings -> view support -> panel support -> layout support`

## View Contract Schema

Every view contract must define:

- `required canonical records`
- `optional canonical records`
- `required metadata`
- `interaction requirements`
- `cross-view participation requirements`, where the view can drive or receive
  linked dashboard interactions
- `output events/commands`
- `degradation behavior`
- `unsupported conditions`
- `view tiers` where relevant (`thumbnail`, `tooltip`, `panel`, `world`)

## Support Definitions

### View-Supported

A backend supports a view when it can provide:

- all required canonical mappings
- all required metadata
- all required interaction prerequisites

### Panel-Supported

A panel is supported when all of its required hosted views are supported.

Optional hosted views may degrade gracefully.

Panels may also expose shared selection, filter, focus, and correlation surfaces
for layout-level composition.

### Layout-Supported

A layout is supported when:

- all of its required panels are supported
- required cross-view interaction contracts across those panels are supported

Optional panels may be omitted or replaced by declared fallbacks.

## Coverage Labels

Recommended coverage labels:

- `full`
- `partial`
- `unsupported`

Where:

- `full` means all required and optional view/panel support is present
- `full` also means required linked dashboard interactions such as brushing and
  filtering are present where declared
- `partial` means all required support is present but some optional support is
  absent
- `unsupported` means one or more required contracts cannot be satisfied

## Design Consequence

Backend mapping docs should ultimately be expressed in terms of:

- view support
- panel support
- layout support
- linked dashboard interaction support

Not only in terms of backend-native entities.

## Anti-Drift Rule

Do not claim a backend "supports the UI" in the abstract.

Always specify:

- which views it supports
- which panels that enables
- which layouts that enables
- which linked brushing/filtering or other cross-view interactions those layouts
  enable

Do not treat adjacent hosted views as independent silos when the product depends
on them working together as one dashboard.
