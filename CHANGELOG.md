# Changelog

All notable changes to `pf3-edit` will be documented in this file.

The format is intentionally lightweight and optimized for a fast-moving local-first project.

## Unreleased

### Added

- ISPF-inspired terminal editor shell with `Command ===>`, line command field, PF-key legend, and `Top of Data` / `Bottom of Data`
- record-oriented editor core split into `ispf-core`, `ispf-command`, `ispf-screen`, and `ispf-tui`
- direct data-area editing with overwrite, split, join, delete, line feed, and text-entry mode
- primary commands including `SAVE`, `CANCEL`, `END`, `FIND`, `RFIND`, `CHANGE`, `RCHANGE`, `LOCATE`, `COLS`, `SCROLL`, `BOUNDS`, `RESET`, `NUMBER`, `UNNUM`, `CAPS`, and `UNDO`
- line commands including `I`, `D`, `DD`, `R`, `RR`, `TS`, `TF`, `TE`, `C`, `CC`, `M`, `MM`, `A`, `B`, `O`, `OO`, `X`, `XX`, `LC`, `LCC`, `UC`, and `UCC`
- excluded-block placeholder rows with local `S`
- developer convenience `Makefile`
- public CLI polish via `pf3-edit --help`, `--version`, and `--debug-keys`
- GitHub Actions CI for formatting, tests, and Clippy

### Changed

- scroll behavior now follows profile-driven `PAGE`, `HALF`, and `CSR` modes
- command parsing is case-insensitive while preserving operand case
- line command workflows now support command-line `:` variants
- session internals are split into focused editing, navigation, and transfer modules

### Fixed

- `RFIND` / `F5` now continue from the current match instead of restarting at the first occurrence
- quoted `FIND` / `CHANGE` arguments now work correctly
- `FIND` / `CHANGE` / `RCHANGE` place the cursor on the actual matched or changed text
- bounds-aware editing now rejects invalid split positions and preserves intended cursor columns more reliably
