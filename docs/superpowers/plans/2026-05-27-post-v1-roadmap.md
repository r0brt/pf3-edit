# Post-v1 Roadmap Capture

Date: 2026-05-27

## Goal

Capture the intentional product direction after `v1.0.0` so the next work does not become a grab-bag of unrelated features.

## Why now

- `v1.0.0` is published and usable
- the project now needs a product path, not just a backlog of commands
- the user explicitly asked for a clearer picture of what a meaningful `v2` or `v3` could be

## Decision

Use release lines with distinct product goals:

- `v1.1`: polish, adoption, and public usability
- `v2.0`: a small ISPF-inspired workbench, especially file navigation and persistent working context
- `v3.0`: macros, scripting, and extension points

## Deliverables

- public roadmap document in `docs/ROADMAP.md`
- short README link to the roadmap
- spec updated so the active focus reflects post-`v1` priorities instead of pre-`v1` release work

## Explicit non-goals

- no promise of full historical ISPF command parity before product-level improvements
- no commitment yet to a full GUI rewrite
- no attempt to define a detailed implementation plan for `v1.1`, `v2.0`, or `v3.0` in this step

## Follow-up

When work on the next release line begins, create a scoped implementation plan for the chosen block rather than treating the roadmap itself as an execution checklist.
