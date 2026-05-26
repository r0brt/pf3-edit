# Scroll And Viewport Semantics Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Bring `pf3-edit` closer to documented ISPF scroll behavior by making vertical `CSR` scrolling cursor-anchored while preserving the existing `PAGE` / `HALF` behavior and explicit numeric scroll counts.

**Architecture:** Keep the implementation centered in `ispf-core` session navigation so scroll semantics live in one place, with a small app-level regression test to confirm `F7/F8` exercise the same path.

**Tech Stack:** Rust workspace, `ispf-core` session tests, `ispf-tui` app tests, existing profile-driven scroll modes.

---

## File Map

- Modify: `/Users/robert/code/ispf-editor/crates/ispf-core/src/session/navigation.rs`
- Modify: `/Users/robert/code/ispf-editor/crates/ispf-core/src/session.rs`
- Modify: `/Users/robert/code/ispf-editor/crates/ispf-core/tests/session_tests.rs`
- Modify: `/Users/robert/code/ispf-editor/crates/ispf-tui/src/app.rs`
- Modify: `/Users/robert/code/ispf-editor/docs/superpowers/specs/2026-05-23-ispf-editor-design.md`

## Task 1: Add Red Tests For Cursor-Anchored CSR Scrolling

- [ ] Add a core regression proving `DOWN` in `CSR` mode places the cursor row at the top of the next view.
- [ ] Add a core regression proving `UP` in `CSR` mode places the cursor row at the bottom of the next view when possible.
- [ ] Add a core regression proving explicit numeric counts still scroll by count even in `CSR` mode.
- [ ] Add a TUI regression proving `F7/F8` in `CSR` mode hit the cursor-anchored path.

## Task 2: Centralize Vertical Scroll Behavior In Navigation

- [ ] Introduce focused navigation helpers for vertical viewport scrolling.
- [ ] Route `PrimaryCommand::Up` and `PrimaryCommand::Down` through those helpers.
- [ ] Keep `PAGE` / `HALF` behavior unchanged.
- [ ] Apply cursor-anchored semantics only to `CSR` with implicit counts.

## Task 3: Verify And Document

- [ ] Run:

```bash
cargo test
cargo clippy --all-targets --all-features -- -D warnings
```

- [ ] Update the design spec to mention the new `CSR` behavior and the new horizontal-scroll follow-up focus.
