# Search And Replace UX Hardening Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make repeat search and repeat change behave more like a trustworthy editor workflow by continuing from the current cursor context, wrapping once when necessary, and reporting clearly when there are no further matches.

**Architecture:** Keep the repeat-search and repeat-change semantics in `ispf-core` session logic, reuse focused bounded-search helpers from `editing.rs`, and prove the user-facing behavior with both core and TUI regressions.

**Tech Stack:** Rust workspace, `ispf-core` session tests, `ispf-tui` app tests, existing `F5` / `F6` mappings.

---

## File Map

- Modify: `/Users/robert/code/ispf-editor/crates/ispf-core/src/session.rs`
- Modify: `/Users/robert/code/ispf-editor/crates/ispf-core/src/session/editing.rs`
- Modify: `/Users/robert/code/ispf-editor/crates/ispf-core/tests/session_tests.rs`
- Modify: `/Users/robert/code/ispf-editor/crates/ispf-tui/src/app.rs`
- Modify: `/Users/robert/code/ispf-editor/README.md`
- Modify: `/Users/robert/code/ispf-editor/CHANGELOG.md`
- Modify: `/Users/robert/code/ispf-editor/docs/superpowers/specs/2026-05-23-ispf-editor-design.md`

## Task 1: Lock The UX In Tests

- [x] Add core regressions for `RFIND` wrapping to an earlier occurrence.
- [x] Add core regressions for `RFIND` reporting `No further matches`.
- [x] Add core regressions for `RCHANGE` moving to the next occurrence and reporting when no further matches exist.
- [x] Add TUI regressions for `F5` wrapped repeat search and `F6` no-more-matches messaging.

## Task 2: Make Repeat Search And Change Cursor-Aware

- [x] Track the last successful find and change cursor positions.
- [x] Make `RFIND` and `RCHANGE` continue from the current cursor context.
- [x] Skip the current match only when the cursor is still sitting on the last repeated hit.
- [x] Wrap once through earlier rows when needed.
- [x] Report `No further matches` instead of silently reusing the same occurrence.

## Task 3: Verify And Document

- [x] Run:

```bash
cargo fmt --all --check
cargo test
cargo clippy --all-targets --all-features -- -D warnings
```

- [x] Update the README to explain the current `RFIND` / `RCHANGE` behavior.
- [x] Update the changelog and design spec.
