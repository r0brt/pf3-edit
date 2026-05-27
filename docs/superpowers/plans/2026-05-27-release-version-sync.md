# Release Version Sync Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Restore versioning consistency after the first successful GitHub release published `v0.1.1` assets that still embedded the workspace version `0.1.0`.

**Architecture:** Keep runtime behavior unchanged. Limit the work to release metadata: workspace version, lightweight status docs, and GitHub release hygiene around the stalled `v0.1.0` run.

**Tech Stack:** Rust workspace metadata, GitHub Actions release workflow, Markdown docs.

---

## File Map

- Modify: `/Users/robert/code/ispf-editor/Cargo.toml`
- Modify: `/Users/robert/code/ispf-editor/CHANGELOG.md`
- Modify: `/Users/robert/code/ispf-editor/docs/superpowers/specs/2026-05-23-ispf-editor-design.md`

## Task 1: Align Workspace Version With The Next Release Tag

**Files:**
- Modify: `/Users/robert/code/ispf-editor/Cargo.toml`

- [ ] Bump the workspace package version to `0.1.2`.
- [ ] Keep binary naming and packaging behavior unchanged otherwise.

## Task 2: Document The Versioning Hygiene Correction

**Files:**
- Modify: `/Users/robert/code/ispf-editor/CHANGELOG.md`
- Modify: `/Users/robert/code/ispf-editor/docs/superpowers/specs/2026-05-23-ispf-editor-design.md`

- [ ] Note that the next release restores alignment between release tags and embedded binary version metadata.
- [ ] Keep the design spec current about release-readiness work.

## Task 3: Release Hygiene

- [ ] Cancel the stalled `v0.1.0` release workflow run.
- [ ] Verify with:

```bash
cargo fmt --all --check
cargo test
cargo clippy --all-targets --all-features -- -D warnings
```

- [ ] Commit with a message such as:

```bash
git add Cargo.toml CHANGELOG.md docs/superpowers/specs/2026-05-23-ispf-editor-design.md docs/superpowers/plans/2026-05-27-release-version-sync.md
git commit -m "build: sync workspace version with next release"
```
