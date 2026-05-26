# Release Polish And CI Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make `pf3-edit` easier to trust and easier to consume publicly by adding a lightweight changelog, a GitHub Actions verification workflow, and clearer README guidance for users versus contributors.

**Architecture:** Keep runtime behavior unchanged. Focus this block on project-level assets at the repo root and `.github/workflows`, while keeping docs aligned with the current implemented command surface.

**Tech Stack:** Rust workspace, GitHub Actions, Markdown docs, existing `Makefile` and verification commands.

---

## File Map

- Create: `/Users/robert/code/ispf-editor/.github/workflows/ci.yml`
- Create: `/Users/robert/code/ispf-editor/CHANGELOG.md`
- Modify: `/Users/robert/code/ispf-editor/README.md`
- Modify: `/Users/robert/code/ispf-editor/docs/superpowers/specs/2026-05-23-ispf-editor-design.md`

## Task 1: Add Public Verification Workflow

**Files:**
- Create: `/Users/robert/code/ispf-editor/.github/workflows/ci.yml`

- [ ] Add a GitHub Actions workflow that runs on pushes to `main` and feature branches plus pull requests.
- [ ] Include `cargo fmt --all -- --check`, `cargo test`, and `cargo clippy --all-targets --all-features -- -D warnings`.
- [ ] Cache cargo artifacts to keep the workflow practical for a small public Rust repo.

## Task 2: Add A Lightweight Changelog

**Files:**
- Create: `/Users/robert/code/ispf-editor/CHANGELOG.md`

- [ ] Create a simple changelog with an `Unreleased` section.
- [ ] Capture the current major editor capabilities and recent correctness fixes at a high level.
- [ ] Keep the format short and sustainable rather than over-engineered.

## Task 3: Clarify Public README Expectations

**Files:**
- Modify: `/Users/robert/code/ispf-editor/README.md`

- [ ] Add a `Supported Today` section that helps users understand what is already trustworthy.
- [ ] Add a `Not Yet Supported` section to set honest boundaries.
- [ ] Document direct GitHub install via cargo in addition to local path install.
- [ ] Link the changelog and explain that CI mirrors the published verification commands.

## Task 4: Keep Design Docs Current

**Files:**
- Modify: `/Users/robert/code/ispf-editor/docs/superpowers/specs/2026-05-23-ispf-editor-design.md`

- [ ] Update current implementation status to mention CI and release-facing documentation polish.
- [ ] Refresh the “current focus” wording so it matches the present `v1.0` stabilization posture.

## Task 5: Verify And Publish

- [ ] Run:

```bash
cargo fmt --all
cargo test
cargo clippy --all-targets --all-features -- -D warnings
```

- [ ] Commit with a message such as:

```bash
git add .github/workflows/ci.yml CHANGELOG.md README.md docs/superpowers/specs/2026-05-23-ispf-editor-design.md docs/superpowers/plans/2026-05-26-release-polish-and-ci.md
git commit -m "docs: add release polish and ci plan"
```
