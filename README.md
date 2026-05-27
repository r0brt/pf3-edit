# pf3-edit

`pf3-edit` is a local Rust editor inspired by the z/OS ISPF editor.

It is not trying to emulate all of z/OS or all of ISPF. The project focuses on the interaction model that makes ISPF distinctive: `Command ===>`, a line command field, record-oriented editing, strong keyboard flow, and a recognizably host-like screen.

## Status

This repository is active work in progress, but it is already beyond a toy prototype.

Current implemented highlights:

- terminal UI built with `ratatui`
- reusable core/editor/session model
- `Command ===>` and line command workflows
- visible `Top of Data`, `Bottom of Data`, `=COLS>`, and `=BNDS>` lines
- undo, save/cancel/end, find/change, locate, scroll modes, bounds, numbering, caps
- direct data-area editing with overwrite, split, join, delete, and line feed
- line commands including `I`, `D`, `DD`, `R`, `RR`, `TS`, `TF`, `TE`, `C`, `CC`, `M`, `MM`, `A`, `B`, `O`, `OO`, `X`, `XX`, `LC`, `LCn`, `LCC`, `UC`, `UCn`, `UCC`
- stronger bounds-aware behavior, including split validation at active bounds and vertical cursor movement that preserves the intended column across shorter intermediate lines

Still missing are broader ISPF command coverage, dataset/member navigation, macros, persistent profiles, and deeper browse/recovery behavior.

## Supported Today

- single-file local editing in a host-like TUI
- ISPF-style primary command field and line command field
- bounds-aware direct editing, split/join, text flow, and text entry
- search/change/repeat-search/repeat-change flows
- copy/move/overlay/exclude workflows
- profile-driven `PAGE`, `HALF`, and `CSR` scrolling
- local development workflow via `Makefile`, `cargo test`, and `cargo clippy`

## Not Yet Supported

- dataset/member navigation
- macros or REXX
- persistent profiles
- full browse-mode semantics
- the broader long tail of ISPF command coverage
- packaged release binaries

## Run

```bash
cargo run -p ispf-tui -- fixtures/sample.txt
```

You can also pass your own file path:

```bash
cargo run -p ispf-tui -- /path/to/file.txt
```

Without a file argument, the editor starts with a tiny in-memory buffer.

## Quick Start

1. Start the editor on a file:

   ```bash
   pf3-edit path/to/file.txt
   ```

2. `Tab` into `Command ===>` and try:

   ```text
   FIND text
   ```

3. Use `F5` for `RFIND`, `F6` for `RCHANGE`, and `F3` to save and exit.

4. `Tab` into the line command field to try commands like `D`, `R`, `X`, `TS`, `TF`, or `TE`.

## Install

```bash
cargo install --path crates/ispf-tui
```

You can also install directly from GitHub:

```bash
cargo install --git https://github.com/r0brt/pf3-edit.git ispf-tui --bin pf3-edit
```

This installs the executable as:

```bash
pf3-edit
```

Useful top-level CLI flags:

```bash
pf3-edit --help
pf3-edit --version
pf3-edit --debug-keys
```

## Make Targets

The `Makefile` is only a small developer convenience layer for people working from a cloned repo.
You do not need it to install or use `pf3-edit`; it just shortens the most common local commands.

- `make run`
- `make test`
- `make lint`
- `make fmt`
- `make check`
- `make debug-keys`

The recommended local verification flow is:

```bash
make check
```

## Useful Keys

- `Tab`: cycle `Primary Command Field -> Line Command Field -> Data Area`
- `Shift+Tab`: cycle backward `Data Area -> Line Command Field -> Primary Command Field`
- `F3`: save and exit
- `F5`: `RFIND`
- `F6`: `RCHANGE`
- `F7` / `F8`: scroll up / down
- `F10` / `F11`: scroll left / right
- `F12`: cancel
- `Esc`: `END`
- `Enter` in data area: split the current line at the cursor
- `Shift+Enter` in data area: insert a blank line below
- `Ctrl+J`: move the cursor down
- `Home` / `End`: move to line start / line end
  On many MacBook keyboards this is usually `fn + Left` / `fn + Right`.
- `TE` / `TEn` moves focus directly into the data area
- `Enter` in `TE` mode: finish text entry
- `Shift+Enter` is disabled while `TE` mode is active

## Key Matrix

| Area | Key | Behavior |
| --- | --- | --- |
| global | `Tab` / `Shift+Tab` | cycle focus forward / backward |
| global | `F3` | save and exit |
| global | `F12` | cancel session |
| command field | `Enter` | execute primary command |
| command field | `F5` / `F6` | repeat find / repeat change |
| line command field | `Enter` | execute pending line command(s) |
| data area | `Enter` | split line at cursor |
| data area | `Shift+Enter` | insert blank line below |
| data area | `Ctrl+J` | move cursor down |
| data area | `Home` / `End` | move to line start / line end |
| data area | `F7` / `F8` | scroll using active scroll mode |
| data area | `F10` / `F11` | scroll left / right |

## Primary Commands

- `SAVE`
- `CANCEL`
- `END`
- `FIND <text>`
- `RFIND`
- `CHANGE <from> <to>`
- `RCHANGE`
- `LOCATE <line>`
- `L <line>`
- `COLS`
- `SCROLL PAGE`
- `SCROLL HALF`
- `SCROLL CSR`
- `BOUNDS`
- `BOUNDS <left> <right>`
- `RESET`
- `NUMBER`
- `UNNUM`
- `CAPS ON`
- `CAPS OFF`
- `UNDO`

Line commands can also be driven from the primary command field with `:`, for example `:D2`.

Primary and line commands are parsed case-insensitively. Command arguments keep their original case.
`F7` / `F8` use the active scroll mode; explicit `UP n` / `DOWN n` still use the given count.
The visible `Scroll ===>` field is currently display-only; change the active mode through `Command ===>` with `SCROLL PAGE`, `SCROLL HALF`, or `SCROLL CSR`.
`RFIND` / `RCHANGE` continue from the current cursor context, wrap once when needed, and then report `No further matches` instead of cycling forever.

## Line Commands

- `I`, `I3`
- `D`, `D2`, `DD`
- `R`, `R4`, `RR`
- `TS`, `TS3`
- `TF`, `TF50`
- `TE`, `TE3`
- `C`, `C2`, `CC`
- `M`, `M2`, `MM`
- `A`, `B`, `O`, `OO`
- `X`, `XX`
- `LC`, `LC3`, `LCC`
- `UC`, `UC2`, `UCC`

## Command Matrix

| Category | Commands |
| --- | --- |
| save / session | `SAVE`, `CANCEL`, `END`, `UNDO` |
| search / replace | `FIND`, `RFIND`, `CHANGE`, `RCHANGE` |
| navigation / view | `LOCATE`, `L`, `COLS`, `SCROLL`, `BOUNDS`, `RESET`, `NUMBER`, `UNNUM`, `CAPS` |
| insert / delete | `I`, `In`, `D`, `Dn`, `DD` |
| repeat / transform | `R`, `Rn`, `RR`, `TS`, `TSn`, `TF`, `TFn`, `TE`, `TEn`, `LC`, `LCn`, `LCC`, `UC`, `UCn`, `UCC` |
| transfer / placement | `C`, `Cn`, `CC`, `M`, `Mn`, `MM`, `A`, `B`, `O`, `OO` |
| exclude / reveal | `X`, `XX`, `S` |

## Project Layout

- [`crates/ispf-core`](crates/ispf-core): editor state, buffer, session logic, undo, command execution
- [`crates/ispf-command`](crates/ispf-command): parsing for primary and line commands
- [`crates/ispf-screen`](crates/ispf-screen): render model for the TUI
- [`crates/ispf-tui`](crates/ispf-tui): terminal runtime and interaction layer

## Verification

```bash
cargo test
cargo clippy --all-targets --all-features -- -D warnings
```

CI runs the same checks plus `cargo fmt --all -- --check` on GitHub Actions.

## Known Limitations

- this is still a single-file local editor, not a dataset/member browser
- no macros or REXX support yet
- profiles are session-local and not persisted across runs
- browse-mode behavior is intentionally incomplete
- release binaries are not published yet; install currently assumes a Rust toolchain
- command coverage is strong for the core workflow, but not yet full-ISPF

## Release Notes

- Changelog: [`CHANGELOG.md`](CHANGELOG.md)

## Superpowers Docs

- Contribution guide: [`CONTRIBUTING.md`](CONTRIBUTING.md)
- Spec: [`2026-05-23-ispf-editor-design.md`](docs/superpowers/specs/2026-05-23-ispf-editor-design.md)
- Plan: [`2026-05-23-ispf-editor-v0.1-implementation.md`](docs/superpowers/plans/2026-05-23-ispf-editor-v0.1-implementation.md)
