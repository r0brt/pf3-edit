# Horizontal Scroll Semantics Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Bring `pf3-edit` closer to documented ISPF horizontal scroll behavior by distinguishing exact-count `LEFT/RIGHT` commands from mode-driven PF10/PF11 behavior, adding `LEFT/RIGHT MAX`, and making horizontal `CSR` scrolling cursor-anchored.

**Architecture:** Keep horizontal viewport semantics centralized in `ispf-core` session navigation, extend the primary-command parser with explicit horizontal scroll forms, and keep one focused TUI regression proving PF10/PF11 travel through the same core path.

**Tech Stack:** Rust workspace, `ispf-command` parser tests, `ispf-core` session tests, `ispf-tui` app tests, existing profile-driven scroll modes.

---

## File Map

- Modify: `/Users/robert/code/ispf-editor/crates/ispf-command/src/primary.rs`
- Modify: `/Users/robert/code/ispf-editor/crates/ispf-command/src/lib.rs`
- Modify: `/Users/robert/code/ispf-editor/crates/ispf-command/tests/parse_tests.rs`
- Modify: `/Users/robert/code/ispf-editor/crates/ispf-core/src/session.rs`
- Modify: `/Users/robert/code/ispf-editor/crates/ispf-core/src/session/navigation.rs`
- Modify: `/Users/robert/code/ispf-editor/crates/ispf-core/tests/session_tests.rs`
- Modify: `/Users/robert/code/ispf-editor/crates/ispf-screen/tests/render_snapshot.rs`
- Modify: `/Users/robert/code/ispf-editor/crates/ispf-tui/src/app.rs`
- Modify: `/Users/robert/code/ispf-editor/docs/superpowers/specs/2026-05-23-ispf-editor-design.md`
- Modify: `/Users/robert/code/ispf-editor/CHANGELOG.md`

## Task 1: Fix The Contract With Red Tests

- [x] Add parser regressions for `LEFT MAX` and `RIGHT MAX`.
- [x] Add core regressions for exact-count horizontal scrolling.
- [x] Add core regressions for `LEFT/RIGHT MAX`.
- [x] Add core regressions for horizontal `CSR`, including the rule that explicit counts remain numeric.
- [x] Add a TUI regression proving PF10/PF11 in `CSR` mode exercise the cursor-anchored path.

## Task 2: Centralize Horizontal Viewport Rules

- [x] Introduce a typed horizontal-scroll form in the primary-command model instead of overloading raw `usize` counts.
- [x] Route `PrimaryCommand::Left` and `PrimaryCommand::Right` through dedicated navigation helpers.
- [x] Keep `LEFT/RIGHT n` exact and allow them to move into empty visible columns.
- [x] Make `LEFT/RIGHT MAX` jump to the extreme left or to the furthest useful right-hand viewport based on current content and bounds.
- [x] Make mode-driven `CSR` horizontal scrolling anchor the cursor column to the viewport edge while leaving explicit numeric counts untouched.

## Task 3: Verify And Document

- [x] Run:

```bash
cargo test
cargo clippy --all-targets --all-features -- -D warnings
```

- [x] Update the design spec to mention the new horizontal scroll semantics.
- [x] Update the changelog to capture the new viewport behavior.
