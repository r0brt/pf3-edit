# Release Binaries Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make `pf3-edit` easier for public users to install by adding a GitHub Actions release workflow that packages prebuilt binaries for the first supported platforms and by documenting the release path clearly in the repo.

**Architecture:** Keep runtime editor behavior unchanged. Add a second GitHub workflow dedicated to release packaging, update public docs to explain the release path honestly, and keep the design spec aligned with the new distribution story.

**Tech Stack:** Rust workspace, GitHub Actions, Markdown docs, existing `pf3-edit` binary target.

---

## File Map

- Create: `/Users/robert/code/ispf-editor/.github/workflows/release.yml`
- Modify: `/Users/robert/code/ispf-editor/README.md`
- Modify: `/Users/robert/code/ispf-editor/CHANGELOG.md`
- Modify: `/Users/robert/code/ispf-editor/docs/superpowers/specs/2026-05-23-ispf-editor-design.md`

## Task 1: Add Release Packaging Workflow

**Files:**
- Create: `/Users/robert/code/ispf-editor/.github/workflows/release.yml`

- [ ] Add a workflow that runs on `v*` tags and via `workflow_dispatch`.
- [ ] Build native `pf3-edit` archives for:
  - `x86_64-unknown-linux-gnu`
  - `x86_64-apple-darwin`
  - `aarch64-apple-darwin`
- [ ] Package each binary together with lightweight release docs.
- [ ] Upload artifacts for manual runs and publish them automatically on tagged releases.

## Task 2: Clarify Install Story In Public Docs

**Files:**
- Modify: `/Users/robert/code/ispf-editor/README.md`
- Modify: `/Users/robert/code/ispf-editor/CHANGELOG.md`

- [ ] Explain that tagged GitHub releases publish prebuilt binaries for the first supported platforms.
- [ ] Keep `cargo install` documented as the fallback until the first tagged release exists.
- [ ] Update known limitations so the repository does not claim that release binaries are entirely unsupported anymore.
- [ ] Note the new release workflow in the changelog.

## Task 3: Keep Design Docs Current

**Files:**
- Modify: `/Users/robert/code/ispf-editor/docs/superpowers/specs/2026-05-23-ispf-editor-design.md`

- [ ] Update current implementation status to mention the GitHub release-binaries workflow.
- [ ] Refresh the `v1.0` focus language so installability is part of release readiness, not an afterthought.

## Task 4: Verify And Publish

- [ ] Run:

```bash
cargo fmt --all --check
cargo test
cargo clippy --all-targets --all-features -- -D warnings
```

- [ ] Commit with a message such as:

```bash
git add .github/workflows/release.yml README.md CHANGELOG.md docs/superpowers/specs/2026-05-23-ispf-editor-design.md docs/superpowers/plans/2026-05-27-release-binaries.md
git commit -m "build: add release binaries workflow"
```
