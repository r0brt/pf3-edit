# Modern Keybindings and Insert Mode

Date: 2026-05-27
Status: Implemented on `feat/modern-keybindings-insert-mode`

## Goal

Improve daily-driver ergonomics in the data area without sacrificing the host-like interaction model.

## Scope

- add `Ctrl+A` and `Ctrl+E` as practical line start / line end aliases
- add explicit `INSERT ON` / `INSERT OFF` primary commands
- keep overwrite as the default editing mode
- show the active editing mode in the status line as `OVR` or `INS`
- make insert-mode typing bounds-aware in the data area

## Notes

- This is an intentionally hybrid feature: it is not meant to claim strict 3270 authenticity.
- The original ISPF documentation is overwrite-oriented and references `Ins` / `Del` hardware behavior; this block adds modern terminal-friendly equivalents for local use.
