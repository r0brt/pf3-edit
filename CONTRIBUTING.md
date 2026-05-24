# Contributing to `pf3-edit`

`pf3-edit` is built as a pragmatic, ISPF-inspired Rust editor. The goal is not historical perfection at any cost. The goal is a strong ISPF interaction model with modern engineering discipline.

## Working Principles

- Prefer practical, stable behavior over nostalgia when the two conflict.
- Preserve the ISPF mental model where it matters: `Command ===>`, the line command field, record-oriented editing, bounds, columns, and keyboard-first workflows.
- Keep the codebase understandable. Small, clear steps beat cleverness.
- Keep docs, plan, and code aligned.

## Git and Branch Strategy

- Use small, focused branches for meaningful feature blocks.
- Small hotfixes may go directly to `main` when they are obviously isolated.
- Keep commits thematic. One commit should tell one story.
- Do not mix unrelated refactors with feature work unless the refactor is required to make the feature safe or clear.
- Keep `main` releasable. Merge back only after tests, clippy, and a short self-review pass.
- Prefer short-lived feature branches over long-running branches.
- Default merge rule: use squash merge when a branch contains several small implementation commits for one feature story.
- Preserve branch history only when the commits are already clean, semantically separated, and worth keeping individually.
- Delete or close feature branches after integration so the active branch list stays small.

Recommended branch naming:

- `feat/<topic>`
- `fix/<topic>`
- `docs/<topic>`
- `chore/<topic>`

## Code Review and Pairing

- Review for behavior, correctness, regressions, and maintainability first.
- Style comments are secondary to bugs, state handling, and command semantics.
- When a change is subtle, add or update a regression test before considering it done.
- Prefer short review loops over giant end-of-cycle reviews.

## Rust Standards

- Keep functions and modules small enough to understand locally.
- Avoid clever ownership patterns when a simpler design is clearer.
- Use enums and domain types instead of loose string or flag conventions where possible.
- Keep `Option` and `Result` semantics explicit.
- Avoid unnecessary cloning, but do not sacrifice readability for micro-optimizations.
- Treat `clippy` warnings as signals, not noise.
- Refactor before central files become bottlenecks.

Specific project guidance:

- `crates/ispf-core` should own editor behavior.
- `crates/ispf-tui` should stay thin and UI-focused.
- `crates/ispf-screen` should remain a render model, not a behavior layer.
- `crates/ispf-command` should stay the home of parsing and typed command structures.

## Clean Code and Refactoring

- Leave touched areas clearer than you found them when reasonable.
- Prefer targeted refactoring over speculative rewrites.
- If a file starts carrying multiple independent responsibilities, split it before it becomes fragile.
- Do not let large command families grow by copy-paste if a clear shared helper would reduce risk.

## Tests and Verification

Before calling work complete:

- run `cargo test`
- run `cargo clippy --all-targets --all-features -- -D warnings`
- manually exercise the changed workflow when it affects editor behavior or UI interaction

Testing guidance:

- put most behavioral tests in `crates/ispf-core/tests`
- keep parser behavior covered in `crates/ispf-command/tests`
- use `crates/ispf-screen/tests` for render-model expectations
- keep TUI tests focused on integration behavior and regressions

## Documentation Discipline

When the project meaningfully changes:

- update [README.md](/Users/robert/code/ispf-editor/README.md) if user-facing behavior changed
- update the active spec or status notes in [docs/superpowers/specs](/Users/robert/code/ispf-editor/docs/superpowers/specs)
- update plan notes in [docs/superpowers/plans](/Users/robert/code/ispf-editor/docs/superpowers/plans) when execution has clearly moved beyond the old plan

Spec and plan documents should not silently drift behind the code.

## Definition of Done

A change is done when:

- the behavior works
- tests are green
- `clippy` is green
- relevant docs are current
- the commit history is clean enough to understand later

If any of those are missing, the change is not done yet.
