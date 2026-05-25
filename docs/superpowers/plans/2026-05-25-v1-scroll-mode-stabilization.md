# v1.0 Scroll Mode Stabilization Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the static `Scroll ===> PAGE` display with a real scroll mode and make vertical scrolling behavior consistent enough for a trustworthy `v1.0`.

**Architecture:** Keep scroll mode in the editor profile so the session, renderer, and TUI can all consume one source of truth. Drive behavior through targeted `ispf-core` and `ispf-tui` regressions before changing implementation.

**Tech Stack:** Rust workspace, `ispf-core`, `ispf-command`, `ispf-screen`, `ispf-tui`, `cargo test`, `cargo clippy`.

---

## File Map

- Modify parser:
  - `/Users/robert/code/ispf-editor/crates/ispf-command/src/primary.rs`
  - `/Users/robert/code/ispf-editor/crates/ispf-command/tests/parse_tests.rs`
- Modify core session/profile:
  - `/Users/robert/code/ispf-editor/crates/ispf-core/src/profile.rs`
  - `/Users/robert/code/ispf-editor/crates/ispf-core/src/session.rs`
  - `/Users/robert/code/ispf-editor/crates/ispf-core/tests/session_tests.rs`
- Modify screen/TUI:
  - `/Users/robert/code/ispf-editor/crates/ispf-screen/src/render.rs`
  - `/Users/robert/code/ispf-editor/crates/ispf-tui/src/app.rs`
- Modify docs:
  - `/Users/robert/code/ispf-editor/README.md`
  - `/Users/robert/code/ispf-editor/docs/superpowers/specs/2026-05-23-ispf-editor-design.md`
  - `/Users/robert/code/ispf-editor/docs/superpowers/plans/2026-05-23-ispf-editor-v0.1-implementation.md`

## Task 1: Add Parser Coverage For Scroll Mode

- [ ] Add a `Scroll` primary command family for:
  - `SCROLL PAGE`
  - `SCROLL HALF`
  - `SCROLL CSR`
- [ ] Add parser tests that fail red first.

## Task 2: Add Core Scroll Mode And Vertical Scroll Semantics

- [ ] Add a profile-level scroll mode enum with at least `Page`, `Half`, and `Csr`.
- [ ] Keep the default at `Page`.
- [ ] Make `PrimaryCommand::Up` / `Down` accept either explicit counts or mode-driven scrolling.
  - Existing counted forms must keep working.
  - PF-key style scrolling should use the active mode.
- [ ] Add regression tests for:
  - default mode = `Page`
  - `SCROLL HALF` and `SCROLL CSR` update the profile
  - mode-driven vertical scrolling moves by a full page, half page, or cursor step respectively

## Task 3: Wire The TUI To The Scroll Mode

- [ ] Make `F7` / `F8` vertical scrolling use the active mode rather than hard-coded `1`.
- [ ] Update the rendered `Scroll ===>` value to show the actual mode.
- [ ] Add/adjust focused TUI tests if needed.

## Task 4: Update Docs And Verify

- [ ] Update README with supported `SCROLL` commands and the current behavior of `F7` / `F8`.
- [ ] Update spec and historical plan status notes.
- [ ] Run:

```bash
cargo test
cargo clippy --all-targets --all-features -- -D warnings
```

- [ ] Commit:

```bash
git add crates README.md docs/superpowers/specs/2026-05-23-ispf-editor-design.md docs/superpowers/plans/2026-05-23-ispf-editor-v0.1-implementation.md docs/superpowers/plans/2026-05-25-v1-scroll-mode-stabilization.md
git commit -m "feat: add scroll modes"
```
