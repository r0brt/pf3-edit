# CI Node 24 Polish Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Remove the current GitHub-hosted runner deprecation warnings by updating the repo's workflow actions to Node-24-ready major versions without changing the actual verification or release behavior.

**Architecture:** Keep the CI and release job logic intact. Only update the relevant GitHub Action versions and align the public status docs with that maintenance change.

**Tech Stack:** GitHub Actions, Markdown docs, existing Rust verification and release workflows.

---

## File Map

- Modify: `/Users/robert/code/ispf-editor/.github/workflows/ci.yml`
- Modify: `/Users/robert/code/ispf-editor/.github/workflows/release.yml`
- Modify: `/Users/robert/code/ispf-editor/CHANGELOG.md`
- Modify: `/Users/robert/code/ispf-editor/docs/superpowers/specs/2026-05-23-ispf-editor-design.md`

## Task 1: Upgrade Workflow Actions To Node-24-Ready Majors

**Files:**
- Modify: `/Users/robert/code/ispf-editor/.github/workflows/ci.yml`
- Modify: `/Users/robert/code/ispf-editor/.github/workflows/release.yml`

- [ ] Upgrade `actions/checkout` to the current Node-24-ready major.
- [ ] Upgrade release-artifact upload/download actions to the current Node-24-ready majors.
- [ ] Upgrade `softprops/action-gh-release` to the current Node-24-ready major.
- [ ] Keep all existing job semantics otherwise unchanged.

## Task 2: Keep Project Status Docs Current

**Files:**
- Modify: `/Users/robert/code/ispf-editor/CHANGELOG.md`
- Modify: `/Users/robert/code/ispf-editor/docs/superpowers/specs/2026-05-23-ispf-editor-design.md`

- [ ] Record the CI maintenance change in the changelog.
- [ ] Update the implementation-status note in the design spec.

## Task 3: Verify And Publish

- [ ] Run:

```bash
cargo fmt --all --check
cargo test
cargo clippy --all-targets --all-features -- -D warnings
```

- [ ] Push the branch and confirm that the next GitHub CI run no longer reports the Node-20 deprecation warnings.
