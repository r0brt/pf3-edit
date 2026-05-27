# Changelog

All notable changes to `pf3-edit` will be documented in this file.

The format is intentionally lightweight and optimized for a fast-moving local-first project.

## Unreleased

### Added

- GitHub Actions release workflow for tagged prebuilt binaries on macOS (Apple Silicon and Intel) plus Linux x86_64
- ISPF-inspired terminal editor shell with `Command ===>`, line command field, PF-key legend, and `Top of Data` / `Bottom of Data`
- record-oriented editor core split into `ispf-core`, `ispf-command`, `ispf-screen`, and `ispf-tui`
- direct data-area editing with overwrite, split, join, delete, line feed, and text-entry mode
- primary commands including `SAVE`, `CANCEL`, `END`, `FIND`, `RFIND`, `CHANGE`, `RCHANGE`, `LOCATE`, `COLS`, `SCROLL`, `BOUNDS`, `RESET`, `NUMBER`, `UNNUM`, `CAPS`, and `UNDO`
- line commands including `I`, `D`, `DD`, `R`, `RR`, `TS`, `TF`, `TE`, `C`, `CC`, `M`, `MM`, `A`, `B`, `O`, `OO`, `X`, `XX`, `LC`, `LCC`, `UC`, and `UCC`
- excluded-block placeholder rows with local `S`
- developer convenience `Makefile`
- public CLI polish via `pf3-edit --help`, `--version`, and `--debug-keys`
- GitHub Actions CI for formatting, tests, and Clippy
- modern data-area ergonomics with `Ctrl+A` / `Ctrl+E` aliases and explicit `INSERT ON|OFF`
- command history navigation in `Command ===>` via `Up` / `Down`

### Changed

- the explicit `v1.0` release audit now considers the current scoped editor ready for a first public `v1.0.0`
- GitHub workflow actions now target Node-24-ready major versions to remove the current Node-20 deprecation warnings from CI and release runs
- README now carries an explicit `v1.0` release-checklist/status note so release readiness can be judged against concrete criteria instead of vague momentum
- the next release line restores alignment between Git tags and embedded binary version metadata
- README install guidance now documents the staged move from Rust-based installs toward tagged GitHub release binaries
- release packaging now targets the two stable release lanes first: macOS Apple Silicon and Linux x86_64
- scroll behavior now follows profile-driven `PAGE`, `HALF`, and `CSR` modes
- horizontal scrolling now distinguishes exact counts, `MAX`, and mode-driven PF10/PF11 behavior, including cursor-anchored `CSR` semantics
- command parsing is case-insensitive while preserving operand case
- line command workflows now support command-line `:` variants
- session internals are split into focused editing, navigation, and transfer modules
- the status line now shows `OVR` / `INS` for the active data-area editing mode

### Fixed

- `RFIND` / `F5` now continue from the current match instead of restarting at the first occurrence
- `RFIND` / `RCHANGE` now continue from the current cursor context, wrap once when needed, and stop with `No further matches` instead of cycling forever
- quoted `FIND` / `CHANGE` arguments now work correctly
- `FIND` / `CHANGE` / `RCHANGE` place the cursor on the actual matched or changed text
- bounds-aware editing now rejects invalid split positions and preserves intended cursor columns more reliably
