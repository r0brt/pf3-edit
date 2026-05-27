# v1 Release Audit Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Decide whether `pf3-edit` is already strong enough for a first honest `v1.0` release candidate by auditing the current implementation against real user workflows and the repo's stated support boundary.

**Architecture:** Keep runtime behavior unchanged during the audit itself. Capture a concrete release checklist, compare it against the implemented product surface, and only open a follow-up fix block if the audit reveals a real gap.

**Tech Stack:** Rust workspace, current public README/CHANGELOG/specs, existing CLI and release artifacts.

---

## File Map

- Modify: `/Users/robert/code/ispf-editor/README.md`
- Modify: `/Users/robert/code/ispf-editor/CHANGELOG.md`
- Modify: `/Users/robert/code/ispf-editor/docs/superpowers/specs/2026-05-23-ispf-editor-design.md`

## Task 1: Define A Concrete v1.0 Release Checklist

**Files:**
- Modify: `/Users/robert/code/ispf-editor/README.md`

- [ ] Add a short `v1.0 release checklist` section or equivalent status note.
- [ ] Make the checklist concrete enough to answer yes/no for:
  - editing stability
  - search/replace repeat flows
  - bounds/scroll integrity
  - release/install story
  - known limitations clarity

## Task 2: Audit The Current Product Against The Checklist

**Files:**
- Modify: `/Users/robert/code/ispf-editor/CHANGELOG.md`
- Modify: `/Users/robert/code/ispf-editor/docs/superpowers/specs/2026-05-23-ispf-editor-design.md`

- [ ] Record whether the current branch is already release-candidate quality or still needs one more polish block.
- [ ] Keep the public status wording honest and specific.
- [ ] If the audit surfaces a real blocker, capture it as the next planned block instead of hand-waving.

## Task 3: Verify And Decide

- [ ] Run:

```bash
cargo test
cargo clippy --all-targets --all-features -- -D warnings
```

- [ ] Produce a short audit summary with:
  - ready now
  - ready with one more polish block
  - not ready
