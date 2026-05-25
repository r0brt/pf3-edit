# v1.0 Stabilization And Makefile Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a small developer-friendly `Makefile` and harden the highest-risk bounds/text-editing workflows so `pf3-edit` moves closer to a trustworthy `v1.0`.

**Architecture:** Keep the `Makefile` thin and cargo-driven at the repo root. Contain correctness work inside existing `ispf-core` editing/transfer/navigation modules and validate behavior through targeted regression tests before touching implementation.

**Tech Stack:** Rust workspace (`cargo`, `clippy`, `ratatui`, `crossterm`), root `Makefile`, existing README/spec/plan docs.

---

## File Map

- Create: `/Users/robert/code/ispf-editor/Makefile`
- Create or modify tests in:
  - `/Users/robert/code/ispf-editor/crates/ispf-core/tests/session_tests.rs`
  - `/Users/robert/code/ispf-editor/crates/ispf-tui/src/app.rs`
- Modify core logic in:
  - `/Users/robert/code/ispf-editor/crates/ispf-core/src/session/editing.rs`
  - `/Users/robert/code/ispf-editor/crates/ispf-core/src/session/navigation.rs`
  - `/Users/robert/code/ispf-editor/crates/ispf-core/src/session/transfers.rs`
  - `/Users/robert/code/ispf-editor/crates/ispf-core/src/session.rs`
- Modify docs:
  - `/Users/robert/code/ispf-editor/README.md`
  - `/Users/robert/code/ispf-editor/docs/superpowers/specs/2026-05-23-ispf-editor-design.md`
  - `/Users/robert/code/ispf-editor/docs/superpowers/plans/2026-05-23-ispf-editor-v0.1-implementation.md`

## Task 1: Add A Thin Developer Makefile

**Files:**
- Create: `/Users/robert/code/ispf-editor/Makefile`
- Modify: `/Users/robert/code/ispf-editor/README.md`

- [ ] **Step 1: Write the root Makefile**

```make
.PHONY: run test lint fmt check debug-keys

run:
	cargo run -p ispf-tui -- fixtures/sample.txt

test:
	cargo test

lint:
	cargo clippy --all-targets --all-features -- -D warnings

fmt:
	cargo fmt --all

check: test lint

debug-keys:
	cargo run -p ispf-tui -- --debug-keys
```

- [ ] **Step 2: Document the Makefile shortcuts**

Add a short section to `/Users/robert/code/ispf-editor/README.md` after the run instructions:

```markdown
## Make Targets

- `make run`
- `make test`
- `make lint`
- `make fmt`
- `make check`
- `make debug-keys`
```

- [ ] **Step 3: Verify formatting and commands**

Run:

```bash
make fmt
make test
make lint
```

Expected:
- formatter succeeds
- test suite passes
- clippy passes with zero warnings

- [ ] **Step 4: Commit the Makefile block**

```bash
git add Makefile README.md
git commit -m "build: add developer make targets"
```

## Task 2: Add Failing Bounds Regression Tests

**Files:**
- Modify: `/Users/robert/code/ispf-editor/crates/ispf-core/tests/session_tests.rs`

- [ ] **Step 1: Add one failing test for each critical invariant**

Append tests in `/Users/robert/code/ispf-editor/crates/ispf-core/tests/session_tests.rs` for:

```rust
#[test]
fn text_split_refuses_to_split_left_of_the_start_bound() {
    let mut session = EditorSession::new(EditBuffer::from_text("ABCDE\n").unwrap());
    session.execute_primary(PrimaryCommand::Bounds(Some((3, 5)))).unwrap();
    session.view_mut().cursor_row = 0;
    session.view_mut().cursor_col = 1;

    let result = session.execute_prefix(0, PrefixCommand::TextSplit(0));

    assert!(result.is_err());
    assert_eq!(session.buffer().to_text(), "ABCDE\n");
}

#[test]
fn text_split_refuses_to_split_right_of_the_end_bound() {
    let mut session = EditorSession::new(EditBuffer::from_text("ABCDE\n").unwrap());
    session.execute_primary(PrimaryCommand::Bounds(Some((1, 3)))).unwrap();
    session.view_mut().cursor_row = 0;
    session.view_mut().cursor_col = 4;

    let result = session.execute_prefix(0, PrefixCommand::TextSplit(0));

    assert!(result.is_err());
    assert_eq!(session.buffer().to_text(), "ABCDE\n");
}

#[test]
fn insert_blank_line_after_preserves_cursor_column_within_bounds() {
    let mut session = EditorSession::new(EditBuffer::from_text("AAAA\nBBBB\n").unwrap());
    session.execute_primary(PrimaryCommand::Bounds(Some((3, 6)))).unwrap();
    session.view_mut().cursor_row = 0;
    session.view_mut().cursor_col = 4;

    session.insert_blank_line_after(0);

    assert_eq!(session.view().cursor_row, 1);
    assert_eq!(session.view().cursor_col, 4);
}

#[test]
fn move_cursor_down_preserves_horizontal_column_across_shorter_lines() {
    let mut session = EditorSession::new(EditBuffer::from_text("ABCDE\nX\nABCDE\n").unwrap());
    session.view_mut().cursor_col = 4;

    session.move_cursor_down();
    session.move_cursor_down();

    assert_eq!(session.view().cursor_row, 2);
    assert_eq!(session.view().cursor_col, 4);
}
```

- [ ] **Step 2: Run only the new tests and confirm they fail for the right reasons**

Run:

```bash
cargo test -p ispf-core text_split_refuses_to_split_left_of_the_start_bound
cargo test -p ispf-core text_split_refuses_to_split_right_of_the_end_bound
cargo test -p ispf-core insert_blank_line_after_preserves_cursor_column_within_bounds
cargo test -p ispf-core move_cursor_down_preserves_horizontal_column_across_shorter_lines
```

Expected:
- at least one failure caused by current bounds/cursor behavior
- no compile errors

## Task 3: Harden Core Bounds And Cursor Behavior

**Files:**
- Modify: `/Users/robert/code/ispf-editor/crates/ispf-core/src/session/editing.rs`
- Modify: `/Users/robert/code/ispf-editor/crates/ispf-core/src/session/navigation.rs`
- Modify: `/Users/robert/code/ispf-editor/crates/ispf-core/src/session.rs`

- [ ] **Step 1: Make text split validate the active split column against bounds**

In `/Users/robert/code/ispf-editor/crates/ispf-core/src/session/editing.rs`, ensure the split path rejects cursor columns outside the editable range before mutating the buffer.

- [ ] **Step 2: Preserve the desired horizontal column when navigating across uneven lines**

In `/Users/robert/code/ispf-editor/crates/ispf-core/src/session/navigation.rs`, keep vertical motion from permanently collapsing `cursor_col` just because an intermediate line is short. Clamp for display/use, but preserve the intended target column for the next eligible line.

- [ ] **Step 3: Keep line-feed insertion inside the current bounds model**

In `/Users/robert/code/ispf-editor/crates/ispf-core/src/session/editing.rs`, make sure blank-line insertion leaves the cursor on the inserted row and keeps the active column if it is still valid under current bounds.

- [ ] **Step 4: Run the focused regression tests again**

Run:

```bash
cargo test -p ispf-core text_split_refuses_to_split_left_of_the_start_bound
cargo test -p ispf-core text_split_refuses_to_split_right_of_the_end_bound
cargo test -p ispf-core insert_blank_line_after_preserves_cursor_column_within_bounds
cargo test -p ispf-core move_cursor_down_preserves_horizontal_column_across_shorter_lines
```

Expected: PASS

- [ ] **Step 5: Commit the core stabilization**

```bash
git add crates/ispf-core/tests/session_tests.rs crates/ispf-core/src/session/editing.rs crates/ispf-core/src/session/navigation.rs crates/ispf-core/src/session.rs
git commit -m "fix: harden bounds and cursor invariants"
```

## Task 4: Stabilize TUI Integration Around The Hardened Core

**Files:**
- Modify: `/Users/robert/code/ispf-editor/crates/ispf-tui/src/app.rs`

- [ ] **Step 1: Add or adjust TUI regressions only if the core fixes require changed surface behavior**

Examples:
- if a new bounds error now appears in the UI, add a message assertion
- if cursor movement semantics changed, add a focused app-level test

- [ ] **Step 2: Run focused TUI tests if touched**

Run:

```bash
cargo test -p ispf-tui
```

Expected: PASS

- [ ] **Step 3: Commit the TUI follow-up only if there was a real TUI change**

```bash
git add crates/ispf-tui/src/app.rs
git commit -m "test: align tui behavior with stabilized bounds semantics"
```

## Task 5: Update Docs And Project Status

**Files:**
- Modify: `/Users/robert/code/ispf-editor/README.md`
- Modify: `/Users/robert/code/ispf-editor/docs/superpowers/specs/2026-05-23-ispf-editor-design.md`
- Modify: `/Users/robert/code/ispf-editor/docs/superpowers/plans/2026-05-23-ispf-editor-v0.1-implementation.md`

- [ ] **Step 1: Update README**

Document:
- the new `Makefile`
- the current recommended developer workflow (`make check`)
- any bounds/text-workflow clarifications that changed during stabilization

- [ ] **Step 2: Update spec and historical plan**

Add short status notes covering:
- the addition of repo-level make targets
- the specific bounds/text-workflow stabilization that landed

- [ ] **Step 3: Run the full verification suite one more time**

Run:

```bash
cargo test
cargo clippy --all-targets --all-features -- -D warnings
```

Expected: both commands succeed

- [ ] **Step 4: Commit the docs/status updates**

```bash
git add README.md docs/superpowers/specs/2026-05-23-ispf-editor-design.md docs/superpowers/plans/2026-05-23-ispf-editor-v0.1-implementation.md
git commit -m "docs: update stabilization status"
```
