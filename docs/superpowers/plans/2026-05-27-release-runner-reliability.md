# Release Runner Reliability Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Restore a dependable public release path after the initial `v0.1.0` tag stalled on GitHub's Intel macOS runner queue by narrowing the workflow to the currently reliable release lanes.

**Architecture:** Keep editor runtime behavior unchanged. Adjust only the release workflow and the public docs/spec text that describe which release binaries are actually published.

**Tech Stack:** GitHub Actions, Markdown docs, existing `pf3-edit` binary target.

---

## File Map

- Modify: `/Users/robert/code/ispf-editor/.github/workflows/release.yml`
- Modify: `/Users/robert/code/ispf-editor/README.md`
- Modify: `/Users/robert/code/ispf-editor/CHANGELOG.md`
- Modify: `/Users/robert/code/ispf-editor/docs/superpowers/specs/2026-05-23-ispf-editor-design.md`

## Task 1: Narrow Release Workflow To Reliable Targets

**Files:**
- Modify: `/Users/robert/code/ispf-editor/.github/workflows/release.yml`

- [ ] Remove the Intel macOS build target from the release matrix.
- [ ] Keep Linux x86_64 and macOS Apple Silicon packaging unchanged.
- [ ] Preserve the same tag-driven and manual release triggers.

## Task 2: Align Public Messaging

**Files:**
- Modify: `/Users/robert/code/ispf-editor/README.md`
- Modify: `/Users/robert/code/ispf-editor/CHANGELOG.md`

- [ ] Update install docs so they only promise release archives for the stable targets.
- [ ] Record the reliability-driven target narrowing in the changelog.
- [ ] Keep the limitation wording honest about Intel macOS being disabled for now.

## Task 3: Keep Design Docs Current

**Files:**
- Modify: `/Users/robert/code/ispf-editor/docs/superpowers/specs/2026-05-23-ispf-editor-design.md`

- [ ] Update the release-binaries note to match the narrowed target matrix.
- [ ] Refresh the spec review status to reflect the reliability correction.

## Task 4: Verify And Publish

- [ ] Run:

```bash
cargo fmt --all --check
cargo test
cargo clippy --all-targets --all-features -- -D warnings
```

- [ ] Commit with a message such as:

```bash
git add .github/workflows/release.yml README.md CHANGELOG.md docs/superpowers/specs/2026-05-23-ispf-editor-design.md docs/superpowers/plans/2026-05-27-release-runner-reliability.md
git commit -m "build: narrow release workflow targets"
```
