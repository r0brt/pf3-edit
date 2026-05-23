use crate::{CapsMode, EditBuffer, EditProfile, UndoEntry, UndoStack};
use ispf_command::{PrefixCommand, PrimaryCommand};

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

    pub fn buffer(&self) -> &EditBuffer {
        &self.buffer
    }

    pub fn execute_primary(&mut self, command: PrimaryCommand) -> Result<(), String> {
        match command {
            PrimaryCommand::Down(count) => self.view.top_row += count,
            PrimaryCommand::Up(count) => {
                self.view.top_row = self.view.top_row.saturating_sub(count);
            }
            PrimaryCommand::Left(count) => {
                self.view.left_col = self.view.left_col.saturating_sub(count);
            }
            PrimaryCommand::Right(count) => self.view.left_col += count,
            PrimaryCommand::Number(enabled) => self.profile.number_mode = enabled,
            PrimaryCommand::Caps(enabled) => {
                self.profile.caps_mode = if enabled { CapsMode::On } else { CapsMode::Off };
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
                let deleted = self
                    .buffer
                    .delete_at(row)
                    .ok_or_else(|| "invalid row".to_string())?;
                self.undo.push(UndoEntry::DeletedLine {
                    index: row,
                    text: deleted.text,
                });
            }
            PrefixCommand::Insert => self.buffer.insert_after(row, ""),
            PrefixCommand::Repeat => {
                let text = self
                    .buffer
                    .records()
                    .get(row)
                    .ok_or_else(|| "invalid row".to_string())?
                    .text
                    .clone();
                self.buffer.insert_after(row, &text);
            }
            PrefixCommand::Exclude => {
                let previous = self
                    .buffer
                    .records()
                    .get(row)
                    .ok_or_else(|| "invalid row".to_string())?
                    .excluded;
                self.buffer
                    .set_excluded(row, true)
                    .ok_or_else(|| "invalid row".to_string())?;
                self.undo.push(UndoEntry::SetExcluded {
                    index: row,
                    previous,
                });
            }
            PrefixCommand::After | PrefixCommand::Before | PrefixCommand::ExcludeBlock => {}
        }
        Ok(())
    }

    pub fn profile(&self) -> &EditProfile {
        &self.profile
    }

    pub fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }

    pub fn message(&self) -> Option<&SessionMessage> {
        self.message.as_ref()
    }

    fn undo_last(&mut self) -> Result<(), String> {
        match self.undo.pop() {
            Some(UndoEntry::DeletedLine { index, text }) => {
                self.buffer.insert_at(index, &text);
                Ok(())
            }
            Some(UndoEntry::SetExcluded { index, previous }) => self
                .buffer
                .set_excluded(index, previous)
                .ok_or_else(|| "invalid row".to_string()),
            Some(UndoEntry::InsertedLine { index }) => self
                .buffer
                .delete_at(index)
                .ok_or_else(|| "invalid row".to_string())
                .map(|_| ()),
            Some(UndoEntry::ReplacedLine { .. }) | None => Ok(()),
        }
    }
}
