# ISPF Editor Local Rebuild Design

Date: 2026-05-23
Status: Implemented and extended in code, reviewed 2026-05-25

## Goal

Build a local Rust-based editor inspired by the z/OS ISPF editor.

The target for `v0.1` is not a full emulator of z/OS or ISPF. It is a practical, working editor that preserves the core ISPF interaction model:

- `Command ===>` driven editing
- line commands in the left line command field
- Record-oriented buffer behavior
- ISPF-like scrolling and panel structure
- Lightweight profile options such as `CAPS`, `NUMBER`, and `BOUNDS`

The editor should feel recognizably ISPF-like to experienced users while being internally modern, modular, and suitable for later extension.

## Product Direction

The chosen direction is a hybrid:

- externally close to ISPF in layout and editing flow
- internally designed as a modern Rust application with clear boundaries

The first release is intended to be a working daily-use editor for local files rather than a demo and not a full host-environment recreation.

## Current Implementation Status

As of 2026-05-25, the repository has moved beyond the original minimum `v0.1` slice.

Implemented highlights:

- Rust workspace with `ispf-core`, `ispf-command`, `ispf-screen`, and `ispf-tui`
- visible ISPF-like TUI shell with menu bar, `Command ===>`, `Scroll ===>`, PF-key legend, and data banners
- working primary commands including `SAVE`, `CANCEL`, `END`, `FIND`, `RFIND`, `CHANGE`, `RCHANGE`, `LOCATE`, `COLS`, `BOUNDS`, `RESET`, `UNDO`, `NUMBER`, `UNNUM`, and `CAPS`
- working line commands including `I`, `In`, `D`, `Dn`, `DD`, `R`, `Rn`, `RR`, `TS`, `TSn`, `TF`, `TFn`, `C`, `Cn`, `CC`, `M`, `Mn`, `MM`, `A`, `B`, `O`, `OO`, `X`, `XX`, `LC`, `LCn`, `LCC`, `UC`, `UCn`, and `UCC`
- direct data-area editing with overwrite behavior, delete, join, split, line feed, undo, and stronger bounds-aware cursor/edit behavior
- visible `=COLS>` and `=BNDS>` support
- overlay support through `O` and `OO`
- excluded-block placeholder rows with local `S` reveal behavior
- stronger PF-key parity including `F2=Split`, `F3=Save+Exit`, and retained `&` primary commands
- forward and backward focus cycling with `Tab` and `Shift+Tab`
- case-insensitive command parsing for both primary and line commands while preserving operand case

Current focus for the next `v1.0`-oriented block:

- render excluded ranges as visible placeholder rows instead of dropping them entirely from the screen model
- support local `S` behavior on an excluded placeholder row to reveal only that excluded block
- keep `RESET` as the global "show everything again" command

Still intentionally outside the implemented scope:

- dataset/member navigation
- macro execution
- persistent profiles
- full browse-mode behavior
- the broader long tail of ISPF commands

## Non-Goals For v0.1

The following are explicitly out of scope for the first version:

- z/OS dataset and member navigation
- full 3270 terminal emulation
- REXX or edit macro execution
- persistent edit profiles
- multiple open files or tabs
- browse and edit as separate panel systems
- hex editing mode
- full ISPF recovery semantics
- full command compatibility with every ISPF primary and line command

## Platform Strategy

The product will be built in layers:

- first deliver a terminal user interface
- keep the editor core independent from the terminal implementation
- allow a later desktop UI to reuse the same core

This gives us fast progress toward an authentic interaction model without coupling the domain logic to one UI technology.

## Recommended Architecture

The recommended architecture is "core first with a thin TUI."

This is preferred over:

- a monolithic terminal app, which would be faster initially but much harder to extend
- a command or event sourced architecture from day one, which would add complexity too early

The selected architecture keeps the application testable, modular, and suitable for later desktop reuse.

## Workspace Structure

The project should be organized as a Rust workspace:

```text
ispf-editor/
  Cargo.toml
  crates/
    ispf-core/
    ispf-screen/
    ispf-tui/
    ispf-command/
```

## Crate Responsibilities

### `ispf-core`

This crate owns editor behavior and state. It must not depend on the TUI framework.

Responsibilities:

- record-oriented text storage
- file loading and saving
- cursor and viewport state
- edit operations
- primary command execution
- prefix command execution
- undo support
- profile handling
- search and change operations
- excluded-line behavior

Implementation note as of 2026-05-25:

- the session layer has been internally decomposed into focused `navigation`, `editing`, and `transfers` submodules so the core can grow new command families without continuing to centralize all behavior in one monolithic file

### `ispf-screen`

This crate converts editor state into a render-oriented screen model.

Responsibilities:

- define screen zones such as title, command line, scroll field, prefix area, data area, and message line
- format status and error messages
- map editor state into visible rows and columns
- remain reusable by later frontends

### `ispf-tui`

This crate owns the terminal runtime and user interaction.

Responsibilities:

- terminal startup and shutdown
- key event handling
- focus management across command line, prefix area, and data area
- mapping PF-like workflows to available keyboard input
- rendering through `ratatui`

Recommended libraries:

- `ratatui` for drawing
- `crossterm` for terminal input and lifecycle

### `ispf-command`

This crate is optional but recommended early.

Responsibilities:

- parse primary commands from `Command ===>`
- parse prefix commands from the left command column
- validate operands, ranges, and forms
- return typed command structures and clear parse errors

Keeping parsing separate avoids mixing UI concerns with command semantics.

## Core Data Model

The editor should be modeled as records rather than a single text blob.

This is the most important design choice for achieving an ISPF-like feel.

### `EditorSession`

Represents the active editing session.

Suggested contents:

- current file metadata
- `EditBuffer`
- `ViewState`
- `EditProfile`
- undo history
- command line state
- pending prefix commands
- status and error message state

### `EditBuffer`

Represents the editable content.

Suggested structure:

- ordered collection of records
- dirty flag
- newline mode metadata
- file path metadata

Initial shape:

```rust
pub struct EditBuffer {
    pub records: Vec<Record>,
    pub dirty: bool,
}
```

### `Record`

Represents one logical line or record.

Suggested fields:

- `id`
- `text`
- `excluded`
- simple per-line state for inserted or changed markers

Initial shape:

```rust
pub struct Record {
    pub id: RecordId,
    pub text: String,
    pub excluded: bool,
}
```

### Stable Record Identity

Records should have stable IDs instead of being addressed only by current index.

Example:

```rust
pub struct RecordId(pub u64);
```

Why this matters:

- safer undo behavior
- cleaner block move and copy operations
- easier handling of exclusion and later labels

### `ViewState`

Tracks what the user is currently seeing and where input focus lives.

Suggested fields:

- `cursor_row`
- `cursor_col`
- `top_row`
- `left_col`
- visible screen dimensions
- active focus area

Example focus areas:

- `CommandLine`
- `PrefixArea`
- `DataArea`

### `EditProfile`

Represents session-level editing options.

For `v0.1` the profile should include:

- `CAPS`
- `NUMBER`
- `BOUNDS`
- optional basic `TABS`

These options are session-local in the first release and do not need persistence.

### `CommandLineState`

Represents the `Command ===>` area.

Suggested fields:

- current text
- command history
- last find state
- last change state

### `PrefixAreaState`

Represents pending commands entered in the left line-command column.

Suggested approach:

- store typed prefix text by visible row or record ID
- resolve and execute on `Enter`

This matches ISPF's delayed command execution model more closely than immediate execution on every keystroke.

## Command Model

Commands should be parsed into typed Rust enums early.

### Primary Commands

Example shape:

```rust
pub enum PrimaryCommand {
    Save,
    Cancel,
    End,
    Find { pattern: String },
    RFind,
    Change { from: String, to: String },
    RChange,
    Reset,
    Up(usize),
    Down(usize),
    Left(usize),
    Right(usize),
    Bounds(Option<(usize, usize)>),
    Number(bool),
    Caps(CapsMode),
    Undo,
}
```

### Prefix Commands

Example shape:

```rust
pub enum PrefixCommand {
    Insert,
    Delete,
    Repeat,
    After,
    Before,
    Exclude,
    ExcludeBlockStart,
    ExcludeBlockEnd,
    ReplicateBlockStart,
    ReplicateBlockEnd,
    CopyBlockStart,
    CopyBlockEnd,
    MoveBlockStart,
    MoveBlockEnd,
}
```

## Undo Model

For `v0.1`, use a pragmatic undo stack rather than full event sourcing.

Suggested approach:

- each edit operation produces inverse information
- inverse operations are pushed to an undo stack
- `UNDO` replays inverse operations safely

Suggested operation categories:

- insert line
- delete line
- replace text range
- move block
- copy block
- change case
- exclude lines

This keeps the implementation understandable while still supporting real editing workflows.

## User Interface Design

The screen should feel ISPF-like even if the implementation is modern.

The main editing screen should include:

- title or status line
- `Command ===>` line
- `Scroll ===>` field
- prefix command column on the left
- data area
- message line at the bottom

The TUI should favor keyboard operation and reduce modal complexity.

## Feature Scope For `v0.1`

### Included

#### File Workflow

- open a local text file
- edit the file
- save back to the same path
- detect dirty state
- support `CANCEL`
- support `END`

#### ISPF-Like Layout

- title or status line
- `Command ===>`
- `Scroll ===>`
- prefix area
- data area
- message line

#### Navigation

- cursor movement in the data area
- vertical scrolling
- horizontal scrolling
- command-driven scrolling through `UP`, `DOWN`, `LEFT`, `RIGHT`

#### Primary Commands

Required in `v0.1`:

- `SAVE`
- `CANCEL`
- `END`
- `FIND`
- `RFIND`
- `CHANGE`
- `RCHANGE`
- `RESET`
- `UP`
- `DOWN`
- `LEFT`
- `RIGHT`
- `BOUNDS`
- `NUMBER`
- `UNNUM`
- `CAPS ON`
- `CAPS OFF`
- `UNDO`

#### Prefix Commands

Required in `v0.1`:

- `I`
- `D`
- `R`
- `A`
- `B`
- `X`
- `XX`

Stretch goals if implementation remains simple enough:

- `RR`
- `CC`
- `MM`

The product should not depend on block copy or move for the first usable release.

#### Editing Behavior

- normal character entry
- backspace and delete
- line insertion flow
- bounds-aware editing

#### Profile Options

- `CAPS`
- `NUMBER`
- `BOUNDS`
- optional simple `TABS`

#### Undo

- functional undo over recent operations

### Excluded For `v0.1`

- dataset and member browser
- macro engine
- persistent profiles
- multi-file sessions
- split views
- complete command family compatibility
- full hex mode
- configurable PF key tables

## Keyboard Strategy

Modern terminals do not guarantee original PF-key behavior, so `v0.1` should provide two paths:

### Native Terminal Keys

- arrow keys for cursor movement
- `Enter` to execute commands
- `Esc` for focus or mode transitions when needed
- optional `Ctrl+S` as a save alias

### PF-Like Mappings

Provide familiar behaviors where practical:

- `F3` for `END`
- `F7` and `F8` for scroll up and down
- `F10` and `F11` for horizontal scroll

Also provide fallbacks for terminals where function keys are unreliable.

## Error Handling

The editor should use clear, non-destructive feedback.

Principles:

- parsing errors affect only the attempted command
- command errors do not terminate the session
- file I/O errors are shown in the message line
- excluded lines remain in the buffer and only change visibility

`CANCEL` and `END` should be intentionally distinct:

- `CANCEL` discards changes and exits the session
- `END` performs normal session termination and should protect against accidental loss when the buffer is dirty

The precise `END` safeguard can be confirmed during implementation, but loss of unsaved work must not happen silently.

## Testing Strategy

Most tests should target `ispf-core`.

### Unit Tests

Focus on:

- primary command parsing
- prefix command parsing
- buffer mutation operations
- bounds behavior
- find and change logic
- undo behavior
- exclude and reset flows

### Screen Tests

Use snapshot or golden-style tests for `ispf-screen`:

- title formatting
- message rendering
- prefix area layout
- visible data rows

### Integration Tests

Keep TUI integration tests narrow and high value:

- open, edit, save
- enter primary command and redraw
- enter prefix command and redraw

## Recommended Implementation Order

1. create the workspace and crate skeleton
2. implement `EditBuffer`, `Record`, `ViewState`, and file loading or saving
3. implement `Command ===>` and primary command parsing
4. build the TUI layout shell
5. add basic text editing in the data area
6. add prefix commands `I`, `D`, `R`, `A`, `B`, `X`, `XX`
7. add `FIND`, `CHANGE`, `RESET`, `BOUNDS`, `NUMBER`, `CAPS`
8. add undo and finish polish work

## Why This Design

This design deliberately balances authenticity and practicality.

It preserves the elements that define the ISPF editing experience:

- command-centric operation
- record awareness
- prefix-command workflows
- panel-like terminal structure

At the same time, it avoids forcing the implementation to mimic z/OS internals where that would slow delivery without improving the local experience.

The result should be a strong `v0.1` foundation that can later grow toward:

- richer line commands
- block operations
- profile persistence
- macros
- dataset or member navigation
- desktop UI reuse of the same core
