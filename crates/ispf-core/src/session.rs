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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ExitDisposition {
    KeepChanges,
    DiscardChanges,
}

pub struct EditorSession {
    buffer: EditBuffer,
    view: ViewState,
    profile: EditProfile,
    undo: UndoStack,
    message: Option<SessionMessage>,
    last_find: Option<String>,
    last_change: Option<(String, String)>,
    should_exit: bool,
    exit_disposition: ExitDisposition,
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
            last_find: None,
            last_change: None,
            should_exit: false,
            exit_disposition: ExitDisposition::KeepChanges,
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
            PrimaryCommand::Save => {
                self.buffer.save().map_err(|err| err.to_string())?;
                self.message = Some(SessionMessage {
                    text: "Save completed".into(),
                    is_error: false,
                });
            }
            PrimaryCommand::Cancel => {
                self.should_exit = true;
                self.exit_disposition = ExitDisposition::DiscardChanges;
            }
            PrimaryCommand::End => {
                if self.buffer.is_dirty() {
                    self.message = Some(SessionMessage {
                        text: "Use SAVE or CANCEL before END".into(),
                        is_error: true,
                    });
                } else {
                    self.should_exit = true;
                    self.exit_disposition = ExitDisposition::KeepChanges;
                }
            }
            PrimaryCommand::Find { pattern } => {
                self.last_find = Some(pattern.clone());
                if let Some(index) = self
                    .buffer
                    .records()
                    .iter()
                    .position(|record| record.text.contains(&pattern))
                {
                    self.view.cursor_row = index;
                    self.view.top_row = index;
                    self.message = Some(SessionMessage {
                        text: "FIND completed".into(),
                        is_error: false,
                    });
                } else {
                    self.message = Some(SessionMessage {
                        text: "Pattern not found".into(),
                        is_error: true,
                    });
                }
            }
            PrimaryCommand::RFind => {
                let pattern = self
                    .last_find
                    .clone()
                    .ok_or_else(|| "No previous FIND pattern".to_string())?;
                self.execute_primary(PrimaryCommand::Find { pattern })?;
            }
            PrimaryCommand::Change { from, to } => {
                self.last_change = Some((from.clone(), to.clone()));
                if let Some(index) = self
                    .buffer
                    .records()
                    .iter()
                    .position(|record| record.text.contains(&from))
                {
                    let previous = self.buffer.records()[index].text.clone();
                    let _ = self.buffer.replace_first(&from, &to);
                    self.view.cursor_row = index;
                    self.view.top_row = index;
                    self.undo.push(UndoEntry::ReplacedLine { index, previous });
                    self.message = Some(SessionMessage {
                        text: "CHANGE completed".into(),
                        is_error: false,
                    });
                } else {
                    self.message = Some(SessionMessage {
                        text: "Pattern not found".into(),
                        is_error: true,
                    });
                }
            }
            PrimaryCommand::RChange => {
                let (from, to) = self
                    .last_change
                    .clone()
                    .ok_or_else(|| "No previous CHANGE arguments".to_string())?;
                self.execute_primary(PrimaryCommand::Change { from, to })?;
            }
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
            PrefixCommand::Insert => {
                let insert_at = if self.buffer.records().is_empty() {
                    0
                } else {
                    row.saturating_add(1).min(self.buffer.records().len())
                };
                self.buffer.insert_after(row, "");
                self.undo.push(UndoEntry::InsertedLine { index: insert_at });
            }
            PrefixCommand::Repeat => {
                let text = self
                    .buffer
                    .records()
                    .get(row)
                    .ok_or_else(|| "invalid row".to_string())?
                    .text
                    .clone();
                self.buffer.insert_after(row, &text);
                self.undo.push(UndoEntry::InsertedLine {
                    index: row.saturating_add(1).min(self.buffer.records().len() - 1),
                });
            }
            PrefixCommand::After => {
                self.buffer.insert_after(row, "");
                self.view.cursor_row = row.saturating_add(1);
                self.undo.push(UndoEntry::InsertedLine {
                    index: row.saturating_add(1).min(self.buffer.records().len() - 1),
                });
            }
            PrefixCommand::Before => {
                self.buffer.insert_before(row, "");
                self.view.cursor_row = row.min(self.buffer.records().len().saturating_sub(1));
                self.undo.push(UndoEntry::InsertedLine { index: row });
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
            PrefixCommand::ExcludeBlock => {
                let upper = (row + 1).min(self.buffer.records().len());
                for index in row..upper {
                    let _ = self.buffer.set_excluded(index, true);
                }
            }
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

    pub fn should_exit(&self) -> bool {
        self.should_exit
    }

    pub fn exit_disposition(&self) -> ExitDisposition {
        self.exit_disposition
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
            Some(UndoEntry::ReplacedLine { index, previous }) => self
                .buffer
                .replace_line(index, &previous)
                .ok_or_else(|| "invalid row".to_string()),
            None => Ok(()),
        }
    }
}
