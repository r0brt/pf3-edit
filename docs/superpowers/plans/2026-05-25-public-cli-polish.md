# Public CLI Polish Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make `pf3-edit` meaningfully easier for others to discover, run, and install by polishing the binary entrypoint and documenting the happy path.

**Architecture:** Keep the CLI layer thin and dependency-light. Parse a small set of top-level flags in `main.rs`, delegate runtime work to the existing TUI module, and validate behavior through smoke-style process tests.

**Tech Stack:** Rust workspace, existing `ispf-tui` crate, standard library arg parsing, README/spec/plan docs.

---

## File Map

- Modify:
  - `/Users/robert/code/ispf-editor/crates/ispf-tui/Cargo.toml`
  - `/Users/robert/code/ispf-editor/crates/ispf-tui/src/main.rs`
  - `/Users/robert/code/ispf-editor/crates/ispf-tui/src/app.rs`
  - `/Users/robert/code/ispf-editor/crates/ispf-tui/tests/smoke.rs`
  - `/Users/robert/code/ispf-editor/README.md`
  - `/Users/robert/code/ispf-editor/docs/superpowers/specs/2026-05-23-ispf-editor-design.md`
  - `/Users/robert/code/ispf-editor/docs/superpowers/plans/2026-05-23-ispf-editor-v0.1-implementation.md`

## Task 1: Add Red Smoke Tests For Public CLI Behavior

- [ ] Add smoke tests for:
  - `pf3-edit --help`
  - `pf3-edit --version`
  - `pf3-edit --debug-keys`
  - invalid extra arguments returning a clear non-zero error

## Task 2: Rename The Binary And Add A Thin CLI Parser

- [ ] Expose the executable as `pf3-edit`
- [ ] Support:
  - `--help` / `-h`
  - `--version` / `-V`
  - `--debug-keys`
  - optional file path
- [ ] Reject unsupported argument combinations with a clear message

## Task 3: Document The Public Run And Install Paths

- [ ] Update README with:
  - `cargo run -p ispf-tui -- <file>`
  - `cargo install --path crates/ispf-tui`
  - resulting executable name `pf3-edit`
  - `--help` / `--version`

## Task 4: Verify And Commit

- [ ] Run:

```bash
cargo test
cargo clippy --all-targets --all-features -- -D warnings
```

- [ ] Commit:

```bash
git add crates README.md docs/superpowers/specs/2026-05-23-ispf-editor-design.md docs/superpowers/plans/2026-05-23-ispf-editor-v0.1-implementation.md docs/superpowers/plans/2026-05-25-public-cli-polish.md
git commit -m "feat: improve public cli experience"
```
