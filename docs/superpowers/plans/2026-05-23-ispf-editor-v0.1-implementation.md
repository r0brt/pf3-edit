# ISPF Editor v0.1 Implementation Plan

Status note as of 2026-05-25: this file is now mainly historical. The project has implemented the original workspace/core/TUI plan and moved beyond the first `v0.1` slice with additional command coverage, richer data-area editing, visible `COLS`/`BOUNDS`, stronger bounds-aware behavior, and overlay support through `O/OO`.

Current active follow-up block:

- continue release hardening around bounds, text workflows, and viewport behavior after the first `TS` / `TF` / `TE` slice

Most recent implemented follow-up items:

- repo-level `Makefile` with `run`, `test`, `lint`, `fmt`, `check`, and `debug-keys` targets
- profile-driven scroll modes (`SCROLL PAGE|HALF|CSR`) with PF7/PF8 using the active mode and the header showing the real `Scroll ===>` state
- public CLI polish with a `pf3-edit` binary, `--help`, `--version`, `--debug-keys`, and a documented `cargo install --path crates/ispf-tui` path
- stronger bounds/text-workflow invariants: direct splits and `TS` reject out-of-bounds cursor positions, and vertical cursor movement preserves the intended horizontal column across shorter intermediate lines
- minimal `TE` text-entry mode with bounds-aware wrapping
- `TE` activation now drops straight into the data area, and `Shift+Enter` is blocked while text entry is active
- backward focus cycling with `Shift+Tab`
- case-insensitive primary and line command parsing
- initial `TS` and `TF` text-workflow line commands
- excluded-block placeholder rows plus local `S`
- stronger PF-key parity for `F2=Split`
- retained `&` command-line primary commands
- internal `ispf-core` session refactor into focused navigation, editing, and transfer submodules before the next larger text-workflow block

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a Rust workspace for a local ISPF-inspired editor with a reusable core, a thin TUI, and a working `v0.1` feature slice for single-file editing.

**Architecture:** The implementation is split into a UI-agnostic editor core, a small screen-model layer, a dedicated command parser crate, and a `ratatui`-based terminal frontend. The first iteration favors a working vertical slice over full ISPF command coverage while preserving the core interaction model: `Command ===>`, prefix commands, record-oriented editing, and ISPF-style status feedback.

**Tech Stack:** Rust stable, Cargo workspace, `ratatui`, `crossterm`, `anyhow`, `thiserror`, `unicode-width`, `insta`

**Execution Note:** Use the Medium model for execution tasks in this project unless the user asks otherwise.

---

## Planned File Structure

### Workspace Root

- Create: `/Users/robert/code/ispf-editor/Cargo.toml`
- Create: `/Users/robert/code/ispf-editor/.gitignore`
- Create: `/Users/robert/code/ispf-editor/rust-toolchain.toml`

### Core Crate

- Create: `/Users/robert/code/ispf-editor/crates/ispf-core/Cargo.toml`
- Create: `/Users/robert/code/ispf-editor/crates/ispf-core/src/lib.rs`
- Create: `/Users/robert/code/ispf-editor/crates/ispf-core/src/buffer.rs`
- Create: `/Users/robert/code/ispf-editor/crates/ispf-core/src/session.rs`
- Create: `/Users/robert/code/ispf-editor/crates/ispf-core/src/profile.rs`
- Create: `/Users/robert/code/ispf-editor/crates/ispf-core/src/undo.rs`
- Create: `/Users/robert/code/ispf-editor/crates/ispf-core/tests/buffer_tests.rs`
- Create: `/Users/robert/code/ispf-editor/crates/ispf-core/tests/session_tests.rs`

### Command Crate

- Create: `/Users/robert/code/ispf-editor/crates/ispf-command/Cargo.toml`
- Create: `/Users/robert/code/ispf-editor/crates/ispf-command/src/lib.rs`
- Create: `/Users/robert/code/ispf-editor/crates/ispf-command/src/primary.rs`
- Create: `/Users/robert/code/ispf-editor/crates/ispf-command/src/prefix.rs`
- Create: `/Users/robert/code/ispf-editor/crates/ispf-command/tests/parse_tests.rs`

### Screen Crate

- Create: `/Users/robert/code/ispf-editor/crates/ispf-screen/Cargo.toml`
- Create: `/Users/robert/code/ispf-editor/crates/ispf-screen/src/lib.rs`
- Create: `/Users/robert/code/ispf-editor/crates/ispf-screen/src/render.rs`
- Create: `/Users/robert/code/ispf-editor/crates/ispf-screen/tests/render_snapshot.rs`

### TUI Crate

- Create: `/Users/robert/code/ispf-editor/crates/ispf-tui/Cargo.toml`
- Create: `/Users/robert/code/ispf-editor/crates/ispf-tui/src/main.rs`
- Create: `/Users/robert/code/ispf-editor/crates/ispf-tui/src/app.rs`
- Create: `/Users/robert/code/ispf-editor/crates/ispf-tui/src/input.rs`
- Create: `/Users/robert/code/ispf-editor/crates/ispf-tui/tests/smoke.rs`

### Sample Fixtures

- Create: `/Users/robert/code/ispf-editor/fixtures/sample.txt`

## Task 1: Scaffold The Rust Workspace

**Files:**
- Create: `/Users/robert/code/ispf-editor/Cargo.toml`
- Create: `/Users/robert/code/ispf-editor/.gitignore`
- Create: `/Users/robert/code/ispf-editor/Cargo.lock`
- Create: `/Users/robert/code/ispf-editor/rust-toolchain.toml`
- Create: `/Users/robert/code/ispf-editor/crates/ispf-core/Cargo.toml`
- Create: `/Users/robert/code/ispf-editor/crates/ispf-core/src/lib.rs`
- Create: `/Users/robert/code/ispf-editor/crates/ispf-command/Cargo.toml`
- Create: `/Users/robert/code/ispf-editor/crates/ispf-command/src/lib.rs`
- Create: `/Users/robert/code/ispf-editor/crates/ispf-screen/Cargo.toml`
- Create: `/Users/robert/code/ispf-editor/crates/ispf-screen/src/lib.rs`
- Create: `/Users/robert/code/ispf-editor/crates/ispf-tui/Cargo.toml`
- Create: `/Users/robert/code/ispf-editor/crates/ispf-tui/src/main.rs`

- [ ] **Step 1: Create the root workspace manifest**

```toml
[workspace]
members = [
  "crates/ispf-core",
  "crates/ispf-command",
  "crates/ispf-screen",
  "crates/ispf-tui",
]
resolver = "2"

[workspace.package]
edition = "2024"
license = "MIT"
version = "0.1.0"

[workspace.dependencies]
anyhow = "1.0"
crossterm = "0.28"
insta = "1.43"
ratatui = "0.29"
thiserror = "2.0"
unicode-width = "0.2"
```

- [ ] **Step 2: Add a root `.gitignore`**

Preserve the existing linked-worktree safety rule so future `.worktrees/` directories stay ignored.

```gitignore
/target
/.superpowers
/.worktrees
```

- [ ] **Step 3: Pin the Rust toolchain**

```toml
[toolchain]
channel = "1.95.0"
components = ["rustfmt", "clippy"]
```

- [ ] **Step 4: Create crate manifests**

`/Users/robert/code/ispf-editor/crates/ispf-core/Cargo.toml`

```toml
[package]
name = "ispf-core"
edition.workspace = true
license.workspace = true
version.workspace = true

[dependencies]
anyhow.workspace = true
thiserror.workspace = true
unicode-width.workspace = true
```

`/Users/robert/code/ispf-editor/crates/ispf-command/Cargo.toml`

```toml
[package]
name = "ispf-command"
edition.workspace = true
license.workspace = true
version.workspace = true

[dependencies]
thiserror.workspace = true
```

`/Users/robert/code/ispf-editor/crates/ispf-screen/Cargo.toml`

```toml
[package]
name = "ispf-screen"
edition.workspace = true
license.workspace = true
version.workspace = true

[dependencies]
ispf-core = { path = "../ispf-core" }
unicode-width.workspace = true

[dev-dependencies]
insta.workspace = true
```

`/Users/robert/code/ispf-editor/crates/ispf-tui/Cargo.toml`

```toml
[package]
name = "ispf-tui"
edition.workspace = true
license.workspace = true
version.workspace = true

[dependencies]
anyhow.workspace = true
crossterm.workspace = true
ispf-command = { path = "../ispf-command" }
ispf-core = { path = "../ispf-core" }
ispf-screen = { path = "../ispf-screen" }
ratatui.workspace = true
```

- [ ] **Step 5: Add minimal source stubs so Cargo can validate the workspace**

`/Users/robert/code/ispf-editor/crates/ispf-core/src/lib.rs`

```rust
//! Core library stub for workspace validation.
```

`/Users/robert/code/ispf-editor/crates/ispf-command/src/lib.rs`

```rust
//! Command parser stub for workspace validation.
```

`/Users/robert/code/ispf-editor/crates/ispf-screen/src/lib.rs`

```rust
//! Screen model stub for workspace validation.
```

`/Users/robert/code/ispf-editor/crates/ispf-tui/src/main.rs`

```rust
fn main() {}
```

- [ ] **Step 6: Run Cargo metadata to verify the workspace resolves**

Run: `cargo metadata --format-version 1 --no-deps`
Expected: command exits `0` and lists the four workspace packages

- [ ] **Step 7: Commit the workspace scaffold**

```bash
git add Cargo.toml Cargo.lock .gitignore rust-toolchain.toml crates
git commit -m "chore: scaffold Rust workspace"
```

## Task 2: Build The Record-Oriented Core Buffer

**Files:**
- Create: `/Users/robert/code/ispf-editor/crates/ispf-core/src/lib.rs`
- Create: `/Users/robert/code/ispf-editor/crates/ispf-core/src/buffer.rs`
- Create: `/Users/robert/code/ispf-editor/crates/ispf-core/src/profile.rs`
- Create: `/Users/robert/code/ispf-editor/crates/ispf-core/src/session.rs`
- Create: `/Users/robert/code/ispf-editor/crates/ispf-core/src/undo.rs`
- Test: `/Users/robert/code/ispf-editor/crates/ispf-core/tests/buffer_tests.rs`

- [ ] **Step 1: Write the failing buffer tests**

`/Users/robert/code/ispf-editor/crates/ispf-core/tests/buffer_tests.rs`

```rust
use ispf_core::{EditBuffer, Record};

#[test]
fn loads_records_from_text_preserving_order() {
    let buffer = EditBuffer::from_text("ONE\nTWO\nTHREE\n").unwrap();
    let texts: Vec<&str> = buffer.records().iter().map(Record::text).collect();
    assert_eq!(texts, vec!["ONE", "TWO", "THREE"]);
}

#[test]
fn insert_and_delete_update_dirty_state() {
    let mut buffer = EditBuffer::from_text("ALPHA\nBETA\n").unwrap();
    assert!(!buffer.is_dirty());

    buffer.insert_after(0, "GAMMA");
    assert!(buffer.is_dirty());
    assert_eq!(buffer.records()[1].text(), "GAMMA");

    buffer.delete_at(1).unwrap();
    assert_eq!(buffer.records()[1].text(), "BETA");
}

#[test]
fn exclude_marks_line_without_removing_record() {
    let mut buffer = EditBuffer::from_text("A\nB\n").unwrap();
    buffer.set_excluded(1, true).unwrap();
    assert!(buffer.records()[1].excluded);
}

#[test]
fn loads_and_saves_a_real_file() {
    let dir = std::env::temp_dir();
    let path = dir.join("ispf-buffer-roundtrip.txt");
    std::fs::write(&path, "LINE1\nLINE2\n").unwrap();

    let mut buffer = EditBuffer::from_path(&path).unwrap();
    buffer.insert_after(1, "LINE3");
    buffer.save().unwrap();

    let saved = std::fs::read_to_string(&path).unwrap();
    assert_eq!(saved, "LINE1\nLINE2\nLINE3\n");
}
```

- [ ] **Step 2: Run the core buffer test file and confirm it fails**

Run: `. "$HOME/.cargo/env" && cargo test -p ispf-core --test buffer_tests`
Expected: FAIL because `ispf_core` and `EditBuffer` are not implemented yet

- [ ] **Step 3: Implement the core buffer types**

`/Users/robert/code/ispf-editor/crates/ispf-core/src/lib.rs`

```rust
mod buffer;
mod profile;
mod session;
mod undo;

pub use buffer::{EditBuffer, Record, RecordId};
pub use profile::{CapsMode, EditProfile};
pub use session::{ActiveArea, EditorSession, SessionMessage, ViewState};
pub use undo::{UndoEntry, UndoStack};
```

`/Users/robert/code/ispf-editor/crates/ispf-core/src/buffer.rs`

```rust
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct RecordId(pub u64);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Record {
    pub id: RecordId,
    pub text: String,
    pub excluded: bool,
}

impl Record {
    pub fn text(&self) -> &str {
        &self.text
    }
}

#[derive(Debug)]
pub struct EditBuffer {
    records: Vec<Record>,
    dirty: bool,
    next_id: u64,
    file_path: Option<std::path::PathBuf>,
    newline: &'static str,
    trailing_newline: bool,
}

impl Default for EditBuffer {
    fn default() -> Self {
        Self {
            records: Vec::new(),
            dirty: false,
            next_id: 1,
            file_path: None,
            newline: "\n",
            trailing_newline: false,
        }
    }
}

impl EditBuffer {
    pub fn from_text(input: &str) -> std::io::Result<Self> {
        Self::try_from_text(input)
    }

    pub fn try_from_text(input: &str) -> std::io::Result<Self> {
        validate_single_newline_style(input)?;
        let mut next_id = 1;
        let mut records = Vec::new();
        for line in input.lines() {
            records.push(Record {
                id: RecordId(next_id),
                text: line.to_string(),
                excluded: false,
            });
            next_id += 1;
        }
        Ok(Self {
            records,
            next_id,
            newline: detect_newline(input),
            trailing_newline: input.ends_with('\n'),
            ..Self::default()
        })
    }

    pub fn from_path(path: &std::path::Path) -> std::io::Result<Self> {
        let text = std::fs::read_to_string(path)?;
        let mut buffer = Self::try_from_text(&text)?;
        buffer.file_path = Some(path.to_path_buf());
        Ok(buffer)
    }

    pub fn records(&self) -> &[Record] {
        &self.records
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    pub fn save(&mut self) -> std::io::Result<()> {
        let path = self
            .file_path
            .clone()
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "missing file path"))?;
        std::fs::write(path, self.to_text())?;
        self.dirty = false;
        Ok(())
    }

    pub fn to_text(&self) -> String {
        if self.records.is_empty() {
            return String::new();
        }
        let mut out = self
            .records
            .iter()
            .map(|record| record.text.as_str())
            .collect::<Vec<_>>()
            .join(self.newline);
        if self.trailing_newline {
            out.push_str(self.newline);
        }
        out
    }

    pub fn insert_after(&mut self, index: usize, text: &str) {
        let record = Record {
            id: RecordId(self.next_id),
            text: text.to_string(),
            excluded: false,
        };
        self.next_id += 1;
        let insert_at = if self.records.is_empty() {
            0
        } else {
            index.saturating_add(1).min(self.records.len())
        };
        self.records.insert(insert_at, record);
        self.dirty = true;
    }

    pub fn delete_at(&mut self, index: usize) -> Option<Record> {
        if index >= self.records.len() {
            return None;
        }
        self.dirty = true;
        Some(self.records.remove(index))
    }

    pub fn set_excluded(&mut self, index: usize, excluded: bool) -> Option<()> {
        let record = self.records.get_mut(index)?;
        if record.excluded == excluded {
            return Some(());
        }
        record.excluded = excluded;
        self.dirty = true;
        Some(())
    }
}

fn detect_newline(input: &str) -> &'static str {
    if input.contains("\r\n") {
        "\r\n"
    } else {
        "\n"
    }
}

fn validate_single_newline_style(input: &str) -> std::io::Result<()> {
    let bytes = input.as_bytes();
    let mut saw_lf = false;
    let mut saw_crlf = false;
    let mut index = 0;

    while index < bytes.len() {
        match bytes[index] {
            b'\r' => {
                if bytes.get(index + 1) != Some(&b'\n') {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "unsupported carriage return newline in text file",
                    ));
                }
                saw_crlf = true;
                index += 2;
            }
            b'\n' => {
                saw_lf = true;
                index += 1;
            }
            _ => {
                index += 1;
            }
        }

        if saw_lf && saw_crlf {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "mixed newline styles are not supported",
            ));
        }
    }

    Ok(())
}
```

Create minimal stubs so the planned `lib.rs` compiles before Task 3 expands them:

`/Users/robert/code/ispf-editor/crates/ispf-core/src/profile.rs`

```rust
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CapsMode {
    Off,
}

#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub struct EditProfile;
```

`/Users/robert/code/ispf-editor/crates/ispf-core/src/session.rs`

```rust
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ActiveArea {
    DataArea,
}

#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub struct ViewState;

#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub struct SessionMessage;

#[derive(Debug, Default)]
pub struct EditorSession;
```

`/Users/robert/code/ispf-editor/crates/ispf-core/src/undo.rs`

```rust
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum UndoEntry {}

#[derive(Debug, Default)]
pub struct UndoStack;
```

- [ ] **Step 4: Re-run the buffer tests**

Run: `. "$HOME/.cargo/env" && cargo test -p ispf-core --test buffer_tests`
Expected: PASS for all four tests

Post-review hardening for this task is allowed and expected:

- preserve CRLF and missing trailing newline on no-op save
- reject mixed newline styles explicitly instead of silently rewriting them
- keep the constructor contract consistent by using a fallible in-memory text constructor path
- make `insert_after` safe for empty and out-of-range indexes
- keep `EditBuffer::default()` valid
- add focused edge-case tests for these behaviors

- [ ] **Step 5: Commit the buffer foundation**

```bash
git add crates/ispf-core
git commit -m "feat: add record-oriented edit buffer"
```

## Task 3: Add Session State, Profiles, And Undo

**Files:**
- Create: `/Users/robert/code/ispf-editor/crates/ispf-core/src/session.rs`
- Create: `/Users/robert/code/ispf-editor/crates/ispf-core/src/profile.rs`
- Create: `/Users/robert/code/ispf-editor/crates/ispf-core/src/undo.rs`
- Test: `/Users/robert/code/ispf-editor/crates/ispf-core/tests/session_tests.rs`

- [ ] **Step 1: Write the failing session tests**

```rust
use ispf_core::{ActiveArea, CapsMode, EditBuffer, EditProfile, EditorSession};

#[test]
fn new_session_starts_in_data_area() {
    let buffer = EditBuffer::from_text("A\n").unwrap();
    let session = EditorSession::new(buffer);
    assert_eq!(session.view().active_area, ActiveArea::DataArea);
}

#[test]
fn profile_defaults_match_v0_1_design() {
    let profile = EditProfile::default();
    assert_eq!(profile.caps_mode, CapsMode::Off);
    assert!(!profile.number_mode);
    assert_eq!(profile.bounds, None);
}
```

- [ ] **Step 2: Run the session tests and confirm they fail**

Run: `cargo test -p ispf-core --test session_tests`
Expected: FAIL because the session and profile modules do not exist yet

- [ ] **Step 3: Implement view, profile, and undo state**

`/Users/robert/code/ispf-editor/crates/ispf-core/src/profile.rs`

```rust
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CapsMode {
    Off,
    On,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EditProfile {
    pub caps_mode: CapsMode,
    pub number_mode: bool,
    pub bounds: Option<(usize, usize)>,
    pub tabs: Vec<usize>,
}

impl Default for EditProfile {
    fn default() -> Self {
        Self {
            caps_mode: CapsMode::Off,
            number_mode: false,
            bounds: None,
            tabs: vec![4, 8, 12, 16],
        }
    }
}
```

`/Users/robert/code/ispf-editor/crates/ispf-core/src/undo.rs`

```rust
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum UndoEntry {
    InsertedLine { index: usize },
    DeletedLine { index: usize, text: String },
    ReplacedLine { index: usize, previous: String },
    SetExcluded { index: usize, previous: bool },
}

#[derive(Debug, Default)]
pub struct UndoStack {
    entries: Vec<UndoEntry>,
}

impl UndoStack {
    pub fn push(&mut self, entry: UndoEntry) {
        self.entries.push(entry);
    }

    pub fn pop(&mut self) -> Option<UndoEntry> {
        self.entries.pop()
    }
}
```

`/Users/robert/code/ispf-editor/crates/ispf-core/src/session.rs`

```rust
use crate::{EditBuffer, EditProfile, UndoStack};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ActiveArea {
    CommandLine,
    PrefixArea,
    DataArea,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ViewState {
    pub cursor_row: usize,
    pub cursor_col: usize,
    pub top_row: usize,
    pub left_col: usize,
    pub active_area: ActiveArea,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SessionMessage {
    pub text: String,
    pub is_error: bool,
}

pub struct EditorSession {
    buffer: EditBuffer,
    view: ViewState,
    profile: EditProfile,
    undo: UndoStack,
    message: Option<SessionMessage>,
}

impl EditorSession {
    pub fn new(buffer: EditBuffer) -> Self {
        Self {
            buffer,
            view: ViewState {
                cursor_row: 0,
                cursor_col: 0,
                top_row: 0,
                left_col: 0,
                active_area: ActiveArea::DataArea,
            },
            profile: EditProfile::default(),
            undo: UndoStack::default(),
            message: None,
        }
    }

    pub fn view(&self) -> &ViewState {
        &self.view
    }

    pub fn profile(&self) -> &EditProfile {
        &self.profile
    }
}
```

- [ ] **Step 4: Re-run the session tests**

Run: `cargo test -p ispf-core --test session_tests`
Expected: PASS for both session tests

- [ ] **Step 5: Commit the session layer**

```bash
git add crates/ispf-core
git commit -m "feat: add session profile and undo state"
```

## Task 4: Parse Primary And Prefix Commands

**Files:**
- Create: `/Users/robert/code/ispf-editor/crates/ispf-command/src/lib.rs`
- Create: `/Users/robert/code/ispf-editor/crates/ispf-command/src/primary.rs`
- Create: `/Users/robert/code/ispf-editor/crates/ispf-command/src/prefix.rs`
- Test: `/Users/robert/code/ispf-editor/crates/ispf-command/tests/parse_tests.rs`

- [ ] **Step 1: Write the failing parser tests**

```rust
use ispf_command::{parse_prefix, parse_primary, PrefixCommand, PrimaryCommand};

#[test]
fn parses_basic_primary_commands() {
    assert_eq!(parse_primary("SAVE").unwrap(), PrimaryCommand::Save);
    assert_eq!(parse_primary("UNNUM").unwrap(), PrimaryCommand::Number(false));
    assert_eq!(parse_primary("CAPS ON").unwrap(), PrimaryCommand::Caps(true));
}

#[test]
fn parses_find_and_change_commands() {
    assert_eq!(
        parse_primary("FIND ALPHA").unwrap(),
        PrimaryCommand::Find { pattern: "ALPHA".into() }
    );
    assert_eq!(
        parse_primary("CHANGE OLD NEW").unwrap(),
        PrimaryCommand::Change { from: "OLD".into(), to: "NEW".into() }
    );
}

#[test]
fn parses_prefix_commands() {
    assert_eq!(parse_prefix("I").unwrap(), PrefixCommand::Insert);
    assert_eq!(parse_prefix("XX").unwrap(), PrefixCommand::ExcludeBlock);
}
```

- [ ] **Step 2: Run the parser tests and confirm they fail**

Run: `cargo test -p ispf-command --test parse_tests`
Expected: FAIL because the parser API does not exist yet

- [ ] **Step 3: Implement the command parser crate**

`/Users/robert/code/ispf-editor/crates/ispf-command/src/lib.rs`

```rust
mod prefix;
mod primary;

pub use prefix::{parse_prefix, PrefixCommand};
pub use primary::{parse_primary, PrimaryCommand};
```

`/Users/robert/code/ispf-editor/crates/ispf-command/src/primary.rs`

```rust
#[derive(Clone, Debug, Eq, PartialEq)]
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
    Caps(bool),
    Undo,
}

pub fn parse_primary(input: &str) -> Result<PrimaryCommand, String> {
    let parts: Vec<&str> = input.split_whitespace().collect();
    match parts.as_slice() {
        ["SAVE"] => Ok(PrimaryCommand::Save),
        ["CANCEL"] => Ok(PrimaryCommand::Cancel),
        ["END"] => Ok(PrimaryCommand::End),
        ["RFIND"] => Ok(PrimaryCommand::RFind),
        ["RCHANGE"] => Ok(PrimaryCommand::RChange),
        ["RESET"] => Ok(PrimaryCommand::Reset),
        ["UNDO"] => Ok(PrimaryCommand::Undo),
        ["UNNUM"] => Ok(PrimaryCommand::Number(false)),
        ["NUMBER"] => Ok(PrimaryCommand::Number(true)),
        ["CAPS", "ON"] => Ok(PrimaryCommand::Caps(true)),
        ["CAPS", "OFF"] => Ok(PrimaryCommand::Caps(false)),
        ["FIND", rest @ ..] if !rest.is_empty() => Ok(PrimaryCommand::Find {
            pattern: rest.join(" "),
        }),
        ["CHANGE", from, to] => Ok(PrimaryCommand::Change {
            from: (*from).into(),
            to: (*to).into(),
        }),
        ["UP", count] => Ok(PrimaryCommand::Up(count.parse().map_err(|_| "invalid UP count".to_string())?)),
        ["DOWN", count] => Ok(PrimaryCommand::Down(count.parse().map_err(|_| "invalid DOWN count".to_string())?)),
        ["LEFT", count] => Ok(PrimaryCommand::Left(count.parse().map_err(|_| "invalid LEFT count".to_string())?)),
        ["RIGHT", count] => Ok(PrimaryCommand::Right(count.parse().map_err(|_| "invalid RIGHT count".to_string())?)),
        _ => Err(format!("unknown primary command: {input}")),
    }
}
```

`/Users/robert/code/ispf-editor/crates/ispf-command/src/prefix.rs`

```rust
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PrefixCommand {
    Insert,
    Delete,
    Repeat,
    After,
    Before,
    Exclude,
    ExcludeBlock,
}

pub fn parse_prefix(input: &str) -> Result<PrefixCommand, String> {
    match input.trim() {
        "I" => Ok(PrefixCommand::Insert),
        "D" => Ok(PrefixCommand::Delete),
        "R" => Ok(PrefixCommand::Repeat),
        "A" => Ok(PrefixCommand::After),
        "B" => Ok(PrefixCommand::Before),
        "X" => Ok(PrefixCommand::Exclude),
        "XX" => Ok(PrefixCommand::ExcludeBlock),
        other => Err(format!("unknown prefix command: {other}")),
    }
}
```

- [ ] **Step 4: Re-run the parser tests**

Run: `cargo test -p ispf-command --test parse_tests`
Expected: PASS for all parser cases

- [ ] **Step 5: Commit the parser crate**

```bash
git add crates/ispf-command
git commit -m "feat: add primary and prefix command parsers"
```

## Task 5: Execute Core Editing Commands

**Files:**
- Modify: `/Users/robert/code/ispf-editor/crates/ispf-core/src/buffer.rs`
- Modify: `/Users/robert/code/ispf-editor/crates/ispf-core/src/session.rs`
- Test: `/Users/robert/code/ispf-editor/crates/ispf-core/tests/session_tests.rs`

- [ ] **Step 1: Extend the session tests with command execution cases**

```rust
use ispf_core::{EditBuffer, EditorSession};
use ispf_command::{PrefixCommand, PrimaryCommand};

#[test]
fn execute_primary_scrolls_and_toggles_profile() {
    let buffer = EditBuffer::from_text("A\nB\nC\n").unwrap();
    let mut session = EditorSession::new(buffer);

    session.execute_primary(PrimaryCommand::Down(2)).unwrap();
    assert_eq!(session.view().top_row, 2);

    session.execute_primary(PrimaryCommand::Number(true)).unwrap();
    assert!(session.profile().number_mode);
}

#[test]
fn execute_prefix_delete_removes_the_target_line() {
    let buffer = EditBuffer::from_text("A\nB\nC\n").unwrap();
    let mut session = EditorSession::new(buffer);

    session.execute_prefix(1, PrefixCommand::Delete).unwrap();
    assert_eq!(session.buffer().records()[1].text(), "C");
}
```

- [ ] **Step 2: Run the session tests and confirm the new cases fail**

Run: `cargo test -p ispf-core --test session_tests`
Expected: FAIL because command execution methods are missing

- [ ] **Step 3: Implement primary and prefix execution in the core**

`/Users/robert/code/ispf-editor/crates/ispf-core/src/session.rs`

```rust
use ispf_command::{PrefixCommand, PrimaryCommand};

impl EditorSession {
    pub fn buffer(&self) -> &EditBuffer {
        &self.buffer
    }

    pub fn execute_primary(&mut self, command: PrimaryCommand) -> Result<(), String> {
        match command {
            PrimaryCommand::Down(count) => self.view.top_row += count,
            PrimaryCommand::Up(count) => self.view.top_row = self.view.top_row.saturating_sub(count),
            PrimaryCommand::Left(count) => self.view.left_col = self.view.left_col.saturating_sub(count),
            PrimaryCommand::Right(count) => self.view.left_col += count,
            PrimaryCommand::Number(enabled) => self.profile.number_mode = enabled,
            PrimaryCommand::Caps(enabled) => {
                self.profile.caps_mode = if enabled { crate::CapsMode::On } else { crate::CapsMode::Off };
            }
            PrimaryCommand::Bounds(bounds) => self.profile.bounds = bounds,
            PrimaryCommand::Reset => {
                for index in 0..self.buffer.records().len() {
                    let _ = self.buffer.set_excluded(index, false);
                }
            }
            PrimaryCommand::Undo => self.undo_last()?,
            _ => {}
        }
        Ok(())
    }

    pub fn execute_prefix(&mut self, row: usize, command: PrefixCommand) -> Result<(), String> {
        match command {
            PrefixCommand::Delete => {
                let deleted = self.buffer.delete_at(row).ok_or_else(|| "invalid row".to_string())?;
                self.undo.push(crate::UndoEntry::DeletedLine { index: row, text: deleted.text });
            }
            PrefixCommand::Insert => self.buffer.insert_after(row, ""),
            PrefixCommand::Repeat => {
                let text = self.buffer.records().get(row).ok_or_else(|| "invalid row".to_string())?.text.clone();
                self.buffer.insert_after(row, &text);
            }
            PrefixCommand::Exclude => {
                let previous = self.buffer.records().get(row).ok_or_else(|| "invalid row".to_string())?.excluded;
                self.buffer.set_excluded(row, true).ok_or_else(|| "invalid row".to_string())?;
                self.undo.push(crate::UndoEntry::SetExcluded { index: row, previous });
            }
            PrefixCommand::After | PrefixCommand::Before | PrefixCommand::ExcludeBlock => {}
        }
        Ok(())
    }

    fn undo_last(&mut self) -> Result<(), String> {
        match self.undo.pop() {
            Some(crate::UndoEntry::DeletedLine { index, text }) => {
                if index == 0 && self.buffer.records().is_empty() {
                    self.buffer.insert_after(0, &text);
                } else if index == 0 {
                    self.buffer.records.insert(0, crate::Record {
                        id: crate::RecordId(0),
                        text,
                        excluded: false,
                    });
                } else {
                    self.buffer.insert_after(index.saturating_sub(1), &text);
                }
                Ok(())
            }
            Some(crate::UndoEntry::SetExcluded { index, previous }) => {
                self.buffer.set_excluded(index, previous).ok_or_else(|| "invalid row".to_string())
            }
            Some(crate::UndoEntry::InsertedLine { index }) => {
                self.buffer.delete_at(index).ok_or_else(|| "invalid row".to_string()).map(|_| ())
            }
            Some(crate::UndoEntry::ReplacedLine { .. }) | None => Ok(()),
        }
    }
}
```

`/Users/robert/code/ispf-editor/crates/ispf-core/Cargo.toml`

```toml
[dependencies]
anyhow.workspace = true
ispf-command = { path = "../ispf-command" }
thiserror.workspace = true
unicode-width.workspace = true
```

- [ ] **Step 4: Re-run the session tests**

Run: `cargo test -p ispf-core --test session_tests`
Expected: PASS for the original and new command-execution tests

- [ ] **Step 5: Commit command execution in the core**

```bash
git add crates/ispf-core/Cargo.toml crates/ispf-core/src crates/ispf-core/tests
git commit -m "feat: execute core primary and prefix commands"
```

## Task 6: Build The Screen Model Layer

**Files:**
- Create: `/Users/robert/code/ispf-editor/crates/ispf-screen/src/lib.rs`
- Create: `/Users/robert/code/ispf-editor/crates/ispf-screen/src/render.rs`
- Test: `/Users/robert/code/ispf-editor/crates/ispf-screen/tests/render_snapshot.rs`

- [ ] **Step 1: Write a failing screen snapshot-style test**

```rust
use ispf_core::{EditBuffer, EditorSession};
use ispf_screen::render_screen;

#[test]
fn renders_command_line_prefix_area_and_data_rows() {
    let session = EditorSession::new(EditBuffer::from_text("ONE\nTWO\n").unwrap());
    let screen = render_screen(&session, 80, 24);

    assert_eq!(screen.command_prompt, "Command ===>");
    assert_eq!(screen.scroll_label, "Scroll ===>");
    assert_eq!(screen.rows[0].text.trim_end(), "ONE");
}
```

- [ ] **Step 2: Run the screen test and confirm it fails**

Run: `cargo test -p ispf-screen --test render_snapshot`
Expected: FAIL because the render API is not implemented yet

- [ ] **Step 3: Implement the screen model**

`/Users/robert/code/ispf-editor/crates/ispf-screen/src/lib.rs`

```rust
mod render;

pub use render::{render_screen, ScreenModel, ScreenRow};
```

`/Users/robert/code/ispf-editor/crates/ispf-screen/src/render.rs`

```rust
use ispf_core::EditorSession;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScreenRow {
    pub prefix: String,
    pub text: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScreenModel {
    pub title: String,
    pub command_prompt: String,
    pub scroll_label: String,
    pub rows: Vec<ScreenRow>,
    pub message: String,
}

pub fn render_screen(session: &EditorSession, _width: u16, height: u16) -> ScreenModel {
    let visible_rows = height.saturating_sub(4) as usize;
    let rows = session
        .buffer()
        .records()
        .iter()
        .skip(session.view().top_row)
        .filter(|record| !record.excluded)
        .take(visible_rows)
        .map(|record| ScreenRow {
            prefix: String::new(),
            text: record.text.clone(),
        })
        .collect();

    ScreenModel {
        title: "ISPF Editor".into(),
        command_prompt: "Command ===>".into(),
        scroll_label: "Scroll ===>".into(),
        rows,
        message: String::new(),
    }
}
```

- [ ] **Step 4: Re-run the screen test**

Run: `cargo test -p ispf-screen --test render_snapshot`
Expected: PASS for the render assertions

- [ ] **Step 5: Commit the screen model**

```bash
git add crates/ispf-screen
git commit -m "feat: add screen rendering model"
```

## Task 7: Wire Up The TUI Shell

**Files:**
- Create: `/Users/robert/code/ispf-editor/crates/ispf-tui/src/main.rs`
- Create: `/Users/robert/code/ispf-editor/crates/ispf-tui/src/app.rs`
- Create: `/Users/robert/code/ispf-editor/crates/ispf-tui/src/input.rs`
- Create: `/Users/robert/code/ispf-editor/fixtures/sample.txt`
- Test: `/Users/robert/code/ispf-editor/crates/ispf-tui/tests/smoke.rs`

- [ ] **Step 1: Add a failing smoke test for application startup**

```rust
#[test]
fn sample_fixture_exists_for_manual_smoke_runs() {
    let fixture = std::path::Path::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../fixtures/sample.txt"
    ));
    assert!(fixture.exists());
}
```

- [ ] **Step 2: Run the smoke test and confirm it fails**

Run: `cargo test -p ispf-tui --test smoke`
Expected: FAIL because the fixture and TUI crate sources are missing

- [ ] **Step 3: Implement the TUI entry point and fixture**

`/Users/robert/code/ispf-editor/crates/ispf-tui/src/main.rs`

```rust
mod app;
mod input;

use anyhow::Result;

fn main() -> Result<()> {
    app::run()
}
```

`/Users/robert/code/ispf-editor/crates/ispf-tui/src/app.rs`

```rust
use anyhow::Result;
use ispf_core::{EditBuffer, EditorSession};
use ispf_screen::render_screen;

pub fn run() -> Result<()> {
    let buffer = EditBuffer::from_text("ISPF EDITOR\n").unwrap();
    let session = EditorSession::new(buffer);
    let _screen = render_screen(&session, 80, 24);
    Ok(())
}
```

`/Users/robert/code/ispf-editor/crates/ispf-tui/src/input.rs`

```rust
use crossterm::event::{KeyCode, KeyEvent};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AppAction {
    MoveUp,
    MoveDown,
    End,
    Execute,
    None,
}

pub fn map_key(event: KeyEvent) -> AppAction {
    match event.code {
        KeyCode::Up => AppAction::MoveUp,
        KeyCode::Down => AppAction::MoveDown,
        KeyCode::F(3) => AppAction::End,
        KeyCode::Enter => AppAction::Execute,
        _ => AppAction::None,
    }
}
```

`/Users/robert/code/ispf-editor/fixtures/sample.txt`

```text
000100 IDENTIFICATION DIVISION.
000200 PROGRAM-ID. SAMPLE.
000300 PROCEDURE DIVISION.
000400     DISPLAY 'HELLO, ISPF'.
```

- [ ] **Step 4: Re-run the smoke test**

Run: `cargo test -p ispf-tui --test smoke`
Expected: PASS for the fixture existence test

- [ ] **Step 5: Run a manual startup smoke check**

Run: `cargo run -p ispf-tui`
Expected: command exits `0` without panicking

- [ ] **Step 6: Commit the TUI shell**

```bash
git add crates/ispf-tui fixtures
git commit -m "feat: add initial TUI shell"
```

## Task 8: Finish The v0.1 Vertical Slice

**Files:**
- Modify: `/Users/robert/code/ispf-editor/crates/ispf-core/src/buffer.rs`
- Modify: `/Users/robert/code/ispf-editor/crates/ispf-core/src/session.rs`
- Modify: `/Users/robert/code/ispf-editor/crates/ispf-screen/src/render.rs`
- Modify: `/Users/robert/code/ispf-editor/crates/ispf-tui/src/app.rs`
- Modify: `/Users/robert/code/ispf-editor/crates/ispf-tui/src/input.rs`
- Test: `/Users/robert/code/ispf-editor/crates/ispf-core/tests/session_tests.rs`
- Test: `/Users/robert/code/ispf-editor/crates/ispf-screen/tests/render_snapshot.rs`

- [ ] **Step 1: Add failing tests for `FIND`, `CHANGE`, `RESET`, and `UNDO`**

```rust
#[test]
fn find_positions_cursor_on_matching_record() {
    let buffer = EditBuffer::from_text("ZERO\nALPHA\nOMEGA\n").unwrap();
    let mut session = EditorSession::new(buffer);
    session.execute_primary(PrimaryCommand::Find { pattern: "ALPHA".into() }).unwrap();
    assert_eq!(session.view().cursor_row, 1);
}

#[test]
fn change_replaces_text_in_place() {
    let buffer = EditBuffer::from_text("OLD VALUE\n").unwrap();
    let mut session = EditorSession::new(buffer);
    session.execute_primary(PrimaryCommand::Change { from: "OLD".into(), to: "NEW".into() }).unwrap();
    assert_eq!(session.buffer().records()[0].text(), "NEW VALUE");
}

#[test]
fn undo_restores_deleted_line() {
    let buffer = EditBuffer::from_text("A\nB\n").unwrap();
    let mut session = EditorSession::new(buffer);
    session.execute_prefix(1, PrefixCommand::Delete).unwrap();
    session.execute_primary(PrimaryCommand::Undo).unwrap();
    assert_eq!(session.buffer().records()[1].text(), "B");
}
```

- [ ] **Step 2: Run the focused session test file and confirm it fails**

Run: `cargo test -p ispf-core --test session_tests`
Expected: FAIL because search, change, and full undo are incomplete

- [ ] **Step 3: Implement the remaining v0.1 behaviors**

Code targets:

- update `EditBuffer` with a `replace_first` helper and a safer insert-before helper
- update `EditorSession::execute_primary` to support `Find`, `RFind`, `Change`, `RChange`, `Save`, `Cancel`, and `End`
- update `EditorSession::execute_prefix` to finish `A`, `B`, `X`, and `XX`
- update `render_screen` to display line numbers when `NUMBER` is enabled
- update `map_key` to include `F7`, `F8`, `F10`, and `F11`
- update `app::run` to accept an optional first CLI argument and load that file through `EditBuffer::from_path`

Implementation sketch for `replace_first`:

```rust
pub fn replace_first(&mut self, from: &str, to: &str) -> Option<usize> {
    for (index, record) in self.records.iter_mut().enumerate() {
        if record.text.contains(from) {
            record.text = record.text.replacen(from, to, 1);
            self.dirty = true;
            return Some(index);
        }
    }
    None
}
```

Implementation sketch for `Find`:

```rust
PrimaryCommand::Find { pattern } => {
    if let Some(index) = self
        .buffer
        .records()
        .iter()
        .position(|record| record.text.contains(&pattern))
    {
        self.view.cursor_row = index;
        self.view.top_row = index;
        self.message = Some(SessionMessage { text: "FIND completed".into(), is_error: false });
    } else {
        self.message = Some(SessionMessage { text: "Pattern not found".into(), is_error: true });
    }
}
```

Implementation sketch for `RFind` and `RChange`:

```rust
PrimaryCommand::RFind => {
    let pattern = self
        .last_find
        .clone()
        .ok_or_else(|| "No previous FIND pattern".to_string())?;
    self.execute_primary(PrimaryCommand::Find { pattern })?;
}
PrimaryCommand::RChange => {
    let (from, to) = self
        .last_change
        .clone()
        .ok_or_else(|| "No previous CHANGE arguments".to_string())?;
    self.execute_primary(PrimaryCommand::Change { from, to })?;
}
```

Implementation sketch for `A`, `B`, and `XX`:

```rust
PrefixCommand::After => {
    self.buffer.insert_after(row, "");
    self.view.cursor_row = row + 1;
}
PrefixCommand::Before => {
    self.buffer.insert_before(row, "");
    self.view.cursor_row = row;
}
PrefixCommand::ExcludeBlock => {
    let upper = (row + 1).min(self.buffer.records().len());
    for idx in row..upper {
        let _ = self.buffer.set_excluded(idx, true);
    }
}
```

Implementation sketch for `SAVE`, `CANCEL`, and `END`:

```rust
PrimaryCommand::Save => {
    self.buffer.save().map_err(|err| err.to_string())?;
    self.message = Some(SessionMessage { text: "Save completed".into(), is_error: false });
}
PrimaryCommand::Cancel => {
    self.should_exit = true;
    self.exit_disposition = ExitDisposition::DiscardChanges;
}
PrimaryCommand::End => {
    if self.buffer.is_dirty() {
        self.message = Some(SessionMessage { text: "Use SAVE or CANCEL before END".into(), is_error: true });
    } else {
        self.should_exit = true;
        self.exit_disposition = ExitDisposition::KeepChanges;
    }
}
```

- [ ] **Step 4: Run all crate tests**

Run: `cargo test`
Expected: PASS across `ispf-core`, `ispf-command`, `ispf-screen`, and `ispf-tui`

- [ ] **Step 5: Run lint checks**

Run: `cargo clippy --all-targets --all-features -- -D warnings`
Expected: PASS with no warnings

- [ ] **Step 6: Commit the v0.1 vertical slice**

```bash
git add crates
git commit -m "feat: deliver ISPF editor v0.1 vertical slice"
```

## Spec Coverage Check

- Architecture split into `ispf-core`, `ispf-screen`, `ispf-command`, and `ispf-tui`: covered by Tasks 1, 2, 3, 4, 6, and 7
- Record-oriented buffer with stable record IDs: covered by Task 2
- Session-local profile settings `CAPS`, `NUMBER`, `BOUNDS`, `TABS`: covered by Tasks 3, 5, and 8
- Primary commands `SAVE`, `CANCEL`, `END`, `FIND`, `RFIND`, `CHANGE`, `RCHANGE`, `RESET`, `UP`, `DOWN`, `LEFT`, `RIGHT`, `BOUNDS`, `NUMBER`, `UNNUM`, `CAPS`, `UNDO`: initial parsing in Task 4, execution in Tasks 5 and 8
- Prefix commands `I`, `D`, `R`, `A`, `B`, `X`, `XX`: parsing in Task 4, execution foundation in Task 5, finish in Task 8
- ISPF-like screen structure with command line, scroll field, prefix area, and message line: covered by Tasks 6 and 7
- Thin TUI with PF-like key mappings: covered by Tasks 7 and 8
- Test strategy centered on core and screen crates: covered throughout, especially Tasks 2, 3, 4, 6, and 8

## Self-Review Notes

- No placeholders such as `TODO` or `TBD` remain in the task steps.
- File paths are absolute and consistent with the approved project layout.
- The only deliberate flexibility is in Task 8, Step 3, where the implementation sketch lists concrete code targets instead of full final file contents. When executing, keep the public API names introduced earlier unchanged.
