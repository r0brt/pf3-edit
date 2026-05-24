use crate::{CapsMode, EditBuffer, EditProfile, UndoEntry, UndoStack};
use ispf_command::{PrefixCommand, PrimaryCommand};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ActiveArea {
    CommandLine,
    LineCommandArea,
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Destination {
    After(usize),
    Before(usize),
    Overlay(usize),
}

pub struct EditorSession {
    buffer: EditBuffer,
    original_buffer: EditBuffer,
    view: ViewState,
    profile: EditProfile,
    undo: UndoStack,
    message: Option<SessionMessage>,
    last_find: Option<String>,
    last_change: Option<(String, String)>,
    pending_exclude_block: Option<usize>,
    pending_delete_block: Option<usize>,
    pending_repeat_block: Option<usize>,
    pending_copy_block: Option<usize>,
    pending_copy_range: Option<(usize, usize)>,
    pending_copy_display: Option<&'static str>,
    pending_move_block: Option<usize>,
    pending_move_range: Option<(usize, usize)>,
    pending_move_display: Option<&'static str>,
    pending_overlay_block: Option<usize>,
    pending_overlay_range: Option<(usize, usize)>,
    pending_overlay_display: Option<&'static str>,
    pending_lowercase_block: Option<usize>,
    pending_uppercase_block: Option<usize>,
    pending_destination: Option<Destination>,
    should_exit: bool,
    exit_disposition: ExitDisposition,
}

impl EditorSession {
    pub fn new(buffer: EditBuffer) -> Self {
        Self {
            original_buffer: buffer.clone(),
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
            pending_exclude_block: None,
            pending_delete_block: None,
            pending_repeat_block: None,
            pending_copy_block: None,
            pending_copy_range: None,
            pending_copy_display: None,
            pending_move_block: None,
            pending_move_range: None,
            pending_move_display: None,
            pending_overlay_block: None,
            pending_overlay_range: None,
            pending_overlay_display: None,
            pending_lowercase_block: None,
            pending_uppercase_block: None,
            pending_destination: None,
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

    pub fn move_cursor_up(&mut self) {
        self.view.cursor_row = self
            .previous_navigable_row(self.view.cursor_row)
            .unwrap_or(self.view.cursor_row);
        self.clamp_cursor_col();
        if self.view.cursor_row < self.view.top_row {
            self.view.top_row = self.view.cursor_row;
        }
    }

    pub fn move_cursor_down(&mut self) {
        if self.buffer.records().is_empty() {
            self.view.cursor_row = 0;
            return;
        }
        self.view.cursor_row = self
            .next_navigable_row(self.view.cursor_row)
            .unwrap_or(self.view.cursor_row);
        self.clamp_cursor_col();
    }

    pub fn move_cursor_left(&mut self) {
        self.view.cursor_col = self
            .view
            .cursor_col
            .saturating_sub(1)
            .max(self.bounds_start_col());
    }

    pub fn move_cursor_right(&mut self) {
        let max_col = self.bounds_line_end_col();
        self.view.cursor_col = (self.view.cursor_col + 1).min(max_col);
    }

    pub fn move_cursor_to_line_start(&mut self) {
        self.view.cursor_col = self.bounds_start_col();
    }

    pub fn move_cursor_to_line_end(&mut self) {
        self.view.cursor_col = self.bounds_line_end_col();
    }

    pub fn toggle_active_area(&mut self) {
        self.view.active_area = match self.view.active_area {
            ActiveArea::CommandLine => ActiveArea::LineCommandArea,
            ActiveArea::LineCommandArea => ActiveArea::DataArea,
            ActiveArea::DataArea => ActiveArea::CommandLine,
        };
    }

    pub fn activate_data_area(&mut self) {
        self.view.active_area = ActiveArea::DataArea;
    }

    pub fn activate_command_line(&mut self) {
        self.view.active_area = ActiveArea::CommandLine;
    }

    pub fn activate_line_command_area(&mut self) {
        self.view.active_area = ActiveArea::LineCommandArea;
    }

    pub fn reset_view_to_top(&mut self) {
        self.view.cursor_row = 0;
        self.view.cursor_col = 0;
        self.view.top_row = 0;
        self.view.left_col = 0;
    }

    pub fn execute_primary(&mut self, command: PrimaryCommand) -> Result<(), String> {
        match command {
            PrimaryCommand::Save => {
                self.buffer.save().map_err(|err| err.to_string())?;
                self.original_buffer = self.buffer.clone();
                self.message = Some(SessionMessage {
                    text: "Save completed".into(),
                    is_error: false,
                });
            }
            PrimaryCommand::Cancel => {
                self.buffer = self.original_buffer.clone();
                self.undo = UndoStack::default();
                self.pending_exclude_block = None;
                self.pending_delete_block = None;
                self.pending_repeat_block = None;
                self.pending_copy_block = None;
                self.pending_copy_range = None;
                self.pending_copy_display = None;
                self.pending_move_block = None;
                self.pending_move_range = None;
                self.pending_move_display = None;
                self.pending_overlay_block = None;
                self.pending_overlay_range = None;
                self.pending_overlay_display = None;
                self.pending_lowercase_block = None;
                self.pending_uppercase_block = None;
                self.pending_destination = None;
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
                    .position(|record| record_text_in_bounds(record.text.as_str(), self.profile.bounds).contains(&pattern))
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
                    .position(|record| record_text_in_bounds(record.text.as_str(), self.profile.bounds).contains(&from))
                {
                    let previous = self.buffer.records()[index].text.clone();
                    let updated = replace_first_in_bounds(&previous, &from, &to, self.profile.bounds)
                        .ok_or_else(|| "Pattern not found".to_string())?;
                    self.buffer
                        .replace_line(index, &updated)
                        .ok_or_else(|| "invalid row".to_string())?;
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
            PrimaryCommand::Locate { target } => {
                let row = target.saturating_sub(1);
                if row >= self.buffer.records().len() {
                    self.message = Some(SessionMessage {
                        text: "Line not found".into(),
                        is_error: true,
                    });
                } else {
                    self.view.cursor_row = self.navigable_row_start(row);
                    self.view.top_row = self.view.cursor_row;
                    self.clamp_cursor_col();
                    self.message = Some(SessionMessage {
                        text: "LOCATE completed".into(),
                        is_error: false,
                    });
                }
            }
            PrimaryCommand::Cols => self.profile.cols_mode = !self.profile.cols_mode,
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
                self.message = Some(SessionMessage {
                    text: "RESET completed".into(),
                    is_error: false,
                });
            }
            PrimaryCommand::Undo => self.undo_last()?,
        }
        Ok(())
    }

    pub fn execute_prefix(&mut self, row: usize, command: PrefixCommand) -> Result<(), String> {
        match command {
            PrefixCommand::Delete(count) => {
                for _ in 0..count {
                    let deleted = self
                        .buffer
                        .delete_at(row)
                        .ok_or_else(|| "invalid row".to_string())?;
                    self.undo.push(UndoEntry::DeletedLine {
                        index: row,
                        record: deleted,
                    });
                }
            }
            PrefixCommand::DeleteBlock => {
                if let Some(start) = self.pending_delete_block.take() {
                    let lower = start.min(row);
                    let upper = start.max(row);
                    let deleted_count = upper - lower + 1;
                    for _ in 0..deleted_count {
                        let deleted = self
                            .buffer
                            .delete_at(lower)
                            .ok_or_else(|| "invalid row".to_string())?;
                        self.undo.push(UndoEntry::DeletedLine {
                            index: lower,
                            record: deleted,
                        });
                    }
                    self.view.cursor_row = lower.min(self.buffer.records().len().saturating_sub(1));
                    self.message = Some(SessionMessage {
                        text: format!("{deleted_count} lines deleted"),
                        is_error: false,
                    });
                } else {
                    self.pending_delete_block = Some(row);
                    self.message = Some(SessionMessage {
                        text: "Delete block start set".into(),
                        is_error: false,
                    });
                }
            }
            PrefixCommand::Insert(count) => {
                for _ in 0..count {
                    self.buffer.insert_before(row, "");
                    self.undo.push(UndoEntry::InsertedLine { index: row });
                }
                self.view.cursor_row = row.min(self.buffer.records().len().saturating_sub(1));
            }
            PrefixCommand::Repeat(count) => {
                let text = self
                    .buffer
                    .records()
                    .get(row)
                    .ok_or_else(|| "invalid row".to_string())?
                    .text
                    .clone();
                for offset in 0..count {
                    self.buffer.insert_after(row + offset, &text);
                    self.undo.push(UndoEntry::InsertedLine {
                        index: row.saturating_add(1 + offset).min(self.buffer.records().len() - 1),
                    });
                }
            }
            PrefixCommand::RepeatBlock => {
                if let Some(start) = self.pending_repeat_block.take() {
                    let lower = start.min(row);
                    let upper = start.max(row);
                    let lines: Vec<String> = self.buffer.records()[lower..=upper]
                        .iter()
                        .map(|record| record.text.clone())
                        .collect();
                    for (offset, text) in lines.iter().enumerate() {
                        self.buffer.insert_after(upper + offset, text);
                        self.undo.push(UndoEntry::InsertedLine {
                            index: upper + 1 + offset,
                        });
                    }
                    self.message = Some(SessionMessage {
                        text: format!("{} lines repeated", lines.len()),
                        is_error: false,
                    });
                } else {
                    self.pending_repeat_block = Some(row);
                    self.message = Some(SessionMessage {
                        text: "Repeat block start set".into(),
                        is_error: false,
                    });
                }
            }
            PrefixCommand::Copy(count) => {
                let last_row = self.buffer.records().len().saturating_sub(1);
                let end = row.saturating_add(count.saturating_sub(1)).min(last_row);
                self.pending_copy_range = Some((row, end));
                self.pending_copy_display = Some(if count == 1 { "C" } else { "CC" });
                self.message = Some(SessionMessage {
                    text: "Copy set".into(),
                    is_error: false,
                });
                self.try_complete_pending_transfer()?;
            }
            PrefixCommand::CopyBlock => {
                if let Some(start) = self.pending_copy_block.take() {
                    self.pending_copy_range = Some((start.min(row), start.max(row)));
                    self.pending_copy_display = Some("CC");
                    self.message = Some(SessionMessage {
                        text: "Copy block set".into(),
                        is_error: false,
                    });
                    self.try_complete_pending_transfer()?;
                } else {
                    self.pending_copy_block = Some(row);
                    self.pending_copy_display = Some("CC");
                    self.message = Some(SessionMessage {
                        text: "Copy block start set".into(),
                        is_error: false,
                    });
                }
            }
            PrefixCommand::Move(count) => {
                let last_row = self.buffer.records().len().saturating_sub(1);
                let end = row.saturating_add(count.saturating_sub(1)).min(last_row);
                self.pending_move_range = Some((row, end));
                self.pending_move_display = Some(if count == 1 { "M" } else { "MM" });
                self.message = Some(SessionMessage {
                    text: "Move set".into(),
                    is_error: false,
                });
                self.try_complete_pending_transfer()?;
            }
            PrefixCommand::MoveBlock => {
                if let Some(start) = self.pending_move_block.take() {
                    self.pending_move_range = Some((start.min(row), start.max(row)));
                    self.pending_move_display = Some("MM");
                    self.message = Some(SessionMessage {
                        text: "Move block set".into(),
                        is_error: false,
                    });
                    self.try_complete_pending_transfer()?;
                } else {
                    self.pending_move_block = Some(row);
                    self.pending_move_display = Some("MM");
                    self.message = Some(SessionMessage {
                        text: "Move block start set".into(),
                        is_error: false,
                    });
                }
            }
            PrefixCommand::Overlay => {
                self.pending_overlay_range = Some((row, row));
                self.pending_overlay_display = Some("O");
                self.message = Some(SessionMessage {
                    text: "Overlay destination set".into(),
                    is_error: false,
                });
                self.try_complete_pending_transfer()?;
            }
            PrefixCommand::OverlayBlock => {
                if let Some(start) = self.pending_overlay_block.take() {
                    self.pending_overlay_range = Some((start.min(row), start.max(row)));
                    self.pending_overlay_display = Some("OO");
                    self.message = Some(SessionMessage {
                        text: "Overlay block set".into(),
                        is_error: false,
                    });
                    self.try_complete_pending_transfer()?;
                } else {
                    self.pending_overlay_block = Some(row);
                    self.pending_overlay_display = Some("OO");
                    self.message = Some(SessionMessage {
                        text: "Overlay block start set".into(),
                        is_error: false,
                    });
                }
            }
            PrefixCommand::Lowercase(count) => {
                let last_row = self.buffer.records().len().saturating_sub(1);
                let end = row.saturating_add(count.saturating_sub(1)).min(last_row);
                self.apply_case_range(row, end, false)?;
                self.message = Some(SessionMessage {
                    text: match count {
                        1 => "1 line lowercased".into(),
                        _ => format!("{} lines lowercased", end.saturating_sub(row) + 1),
                    },
                    is_error: false,
                });
            }
            PrefixCommand::LowercaseBlock => {
                if let Some(start) = self.pending_lowercase_block.take() {
                    let lower = start.min(row);
                    let upper = start.max(row);
                    self.apply_case_range(lower, upper, false)?;
                    self.message = Some(SessionMessage {
                        text: format!("{} lines lowercased", upper - lower + 1),
                        is_error: false,
                    });
                } else {
                    self.pending_lowercase_block = Some(row);
                    self.message = Some(SessionMessage {
                        text: "Lowercase block start set".into(),
                        is_error: false,
                    });
                }
            }
            PrefixCommand::Uppercase(count) => {
                let last_row = self.buffer.records().len().saturating_sub(1);
                let end = row.saturating_add(count.saturating_sub(1)).min(last_row);
                self.apply_case_range(row, end, true)?;
                self.message = Some(SessionMessage {
                    text: match count {
                        1 => "1 line uppercased".into(),
                        _ => format!("{} lines uppercased", end.saturating_sub(row) + 1),
                    },
                    is_error: false,
                });
            }
            PrefixCommand::UppercaseBlock => {
                if let Some(start) = self.pending_uppercase_block.take() {
                    let lower = start.min(row);
                    let upper = start.max(row);
                    self.apply_case_range(lower, upper, true)?;
                    self.message = Some(SessionMessage {
                        text: format!("{} lines uppercased", upper - lower + 1),
                        is_error: false,
                    });
                } else {
                    self.pending_uppercase_block = Some(row);
                    self.message = Some(SessionMessage {
                        text: "Uppercase block start set".into(),
                        is_error: false,
                    });
                }
            }
            PrefixCommand::After => {
                self.pending_destination = Some(Destination::After(row));
                self.message = Some(SessionMessage {
                    text: "After destination set".into(),
                    is_error: false,
                });
                self.try_complete_pending_transfer()?;
            }
            PrefixCommand::Before => {
                self.pending_destination = Some(Destination::Before(row));
                self.message = Some(SessionMessage {
                    text: "Before destination set".into(),
                    is_error: false,
                });
                self.try_complete_pending_transfer()?;
            }
            PrefixCommand::Show => {
                let (start, end) = self
                    .excluded_block_range_at(row)
                    .ok_or_else(|| "S requires an excluded block".to_string())?;
                for index in start..=end {
                    self.buffer
                        .set_excluded(index, false)
                        .ok_or_else(|| "invalid row".to_string())?;
                }
                self.view.cursor_row = start;
                self.view.top_row = self.view.top_row.min(start);
                self.clamp_cursor_col();
                self.message = Some(SessionMessage {
                    text: match end - start + 1 {
                        1 => "1 line shown".into(),
                        count => format!("{count} lines shown"),
                    },
                    is_error: false,
                });
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
                self.move_cursor_to_nearest_visible(row.saturating_add(1), row.saturating_sub(1));
                self.message = Some(SessionMessage {
                    text: "Line excluded".into(),
                    is_error: false,
                });
            }
            PrefixCommand::ExcludeBlock => {
                if let Some(start) = self.pending_exclude_block.take() {
                    let lower = start.min(row);
                    let upper = start.max(row);
                    let mut excluded = 0usize;
                    for index in lower..=upper {
                        if self.buffer.set_excluded(index, true).is_some() {
                            excluded += 1;
                        }
                    }
                    self.move_cursor_to_nearest_visible(upper.saturating_add(1), lower.saturating_sub(1));
                    self.message = Some(SessionMessage {
                        text: format!("{excluded} lines excluded"),
                        is_error: false,
                    });
                } else {
                    self.pending_exclude_block = Some(row);
                    self.message = Some(SessionMessage {
                        text: "Exclude block start set".into(),
                        is_error: false,
                    });
                }
            }
        }
        Ok(())
    }

    pub fn insert_blank_line_after(&mut self, row: usize) {
        self.buffer.insert_after(row, "");
        let inserted_index = row.saturating_add(1).min(self.buffer.records().len().saturating_sub(1));
        self.view.cursor_row = inserted_index;
        self.undo.push(UndoEntry::InsertedLine {
            index: inserted_index,
        });
    }

    pub fn insert_char(&mut self, ch: char) -> Result<(), String> {
        if self.current_row_is_excluded() {
            return Err("Cannot edit excluded lines".into());
        }
        let (_, bounds_end) = self.edit_bounds();
        if self.view.cursor_col > bounds_end {
            return Ok(());
        }

        let row = self.view.cursor_row;
        let previous = self
            .buffer
            .records()
            .get(row)
            .ok_or_else(|| "invalid row".to_string())?
            .text
            .clone();
        let updated = overwrite_char_at(&previous, self.view.cursor_col, ch)?;
        self.buffer
            .replace_line(row, &updated)
            .ok_or_else(|| "invalid row".to_string())?;
        self.undo.push(UndoEntry::ReplacedLine {
            index: row,
            previous,
        });
        self.view.cursor_col = (self.view.cursor_col + 1).min(bounds_end.saturating_add(1));
        Ok(())
    }

    pub fn backspace_char(&mut self) -> Result<(), String> {
        if self.current_row_is_excluded() {
            return Err("Cannot edit excluded lines".into());
        }
        if self.view.cursor_col == 0 {
            let row = self.view.cursor_row;
            if row == 0 {
                return Ok(());
            }

            let previous_index = row - 1;
            let previous_text = self
                .buffer
                .records()
                .get(previous_index)
                .ok_or_else(|| "invalid row".to_string())?
                .text
                .clone();
            let current_record = self
                .buffer
                .delete_at(row)
                .ok_or_else(|| "invalid row".to_string())?;
            let previous_len = previous_text.chars().count();
            let joined = format!("{previous_text}{}", current_record.text);
            self.buffer
                .replace_line(previous_index, &joined)
                .ok_or_else(|| "invalid row".to_string())?;
            self.undo.push(UndoEntry::JoinedLine {
                index: previous_index,
                previous: previous_text,
                removed: current_record,
            });
            self.view.cursor_row = previous_index;
            self.view.cursor_col = previous_len;
            return Ok(());
        }

        let row = self.view.cursor_row;
        let previous = self
            .buffer
            .records()
            .get(row)
            .ok_or_else(|| "invalid row".to_string())?
            .text
            .clone();
        let updated = remove_char_before(&previous, self.view.cursor_col)?;
        self.buffer
            .replace_line(row, &updated)
            .ok_or_else(|| "invalid row".to_string())?;
        self.undo.push(UndoEntry::ReplacedLine {
            index: row,
            previous,
        });
        self.view.cursor_col -= 1;
        Ok(())
    }

    pub fn delete_char(&mut self) -> Result<(), String> {
        if self.current_row_is_excluded() {
            return Err("Cannot edit excluded lines".into());
        }
        let row = self.view.cursor_row;
        let current = self
            .buffer
            .records()
            .get(row)
            .ok_or_else(|| "invalid row".to_string())?
            .text
            .clone();
        let char_count = current.chars().count();

        if self.view.cursor_col < char_count {
            let updated = remove_char_at(&current, self.view.cursor_col)?;
            self.buffer
                .replace_line(row, &updated)
                .ok_or_else(|| "invalid row".to_string())?;
            self.undo.push(UndoEntry::ReplacedLine {
                index: row,
                previous: current,
            });
            return Ok(());
        }

        if row + 1 >= self.buffer.records().len() {
            return Ok(());
        }

        let next_record = self
            .buffer
            .delete_at(row + 1)
            .ok_or_else(|| "invalid row".to_string())?;
        let joined = format!("{current}{}", next_record.text);
        self.buffer
            .replace_line(row, &joined)
            .ok_or_else(|| "invalid row".to_string())?;
        self.undo.push(UndoEntry::JoinedLine {
            index: row,
            previous: current,
            removed: next_record,
        });
        Ok(())
    }

    pub fn split_line_at_cursor(&mut self) -> Result<(), String> {
        if self.current_row_is_excluded() {
            return Err("Cannot edit excluded lines".into());
        }
        let row = self.view.cursor_row;
        let current = self
            .buffer
            .records()
            .get(row)
            .ok_or_else(|| "invalid row".to_string())?
            .text
            .clone();
        let (left, right) = split_text_at(&current, self.view.cursor_col)?;

        self.buffer
            .replace_line(row, &left)
            .ok_or_else(|| "invalid row".to_string())?;
        self.buffer.insert_after(row, &right);
        self.undo.push(UndoEntry::SplitLine {
            index: row,
            previous: current,
        });
        self.view.cursor_row = row + 1;
        self.view.cursor_col = 0;
        Ok(())
    }

    pub fn profile(&self) -> &EditProfile {
        &self.profile
    }

    pub fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }

    pub fn pending_exclude_block(&self) -> Option<usize> {
        self.pending_exclude_block
    }

    pub fn pending_delete_block(&self) -> Option<usize> {
        self.pending_delete_block
    }

    pub fn pending_repeat_block(&self) -> Option<usize> {
        self.pending_repeat_block
    }

    pub fn pending_copy_block(&self) -> Option<usize> {
        self.pending_copy_block
    }

    pub fn pending_copy_range(&self) -> Option<(usize, usize)> {
        self.pending_copy_range
    }

    pub fn pending_copy_display(&self) -> Option<&'static str> {
        self.pending_copy_display
    }

    pub fn pending_move_block(&self) -> Option<usize> {
        self.pending_move_block
    }

    pub fn pending_move_range(&self) -> Option<(usize, usize)> {
        self.pending_move_range
    }

    pub fn pending_move_display(&self) -> Option<&'static str> {
        self.pending_move_display
    }

    pub fn pending_overlay_block(&self) -> Option<usize> {
        self.pending_overlay_block
    }

    pub fn pending_overlay_range(&self) -> Option<(usize, usize)> {
        self.pending_overlay_range
    }

    pub fn pending_overlay_display(&self) -> Option<&'static str> {
        self.pending_overlay_display
    }

    pub fn pending_lowercase_block(&self) -> Option<usize> {
        self.pending_lowercase_block
    }

    pub fn pending_uppercase_block(&self) -> Option<usize> {
        self.pending_uppercase_block
    }

    pub fn pending_destination(&self) -> Option<Destination> {
        self.pending_destination
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

    pub fn row_is_excluded(&self, row: usize) -> bool {
        self.buffer
            .records()
            .get(row)
            .is_some_and(|record| record.excluded)
    }

    fn undo_last(&mut self) -> Result<(), String> {
        match self.undo.pop() {
            Some(UndoEntry::DeletedLine { index, record }) => {
                self.buffer.insert_record_at(index, record);
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
            Some(UndoEntry::JoinedLine {
                index,
                previous,
                removed,
            }) => {
                self.buffer
                    .replace_line(index, &previous)
                    .ok_or_else(|| "invalid row".to_string())?;
                self.buffer.insert_record_at(index + 1, removed);
                Ok(())
            }
            Some(UndoEntry::SplitLine { index, previous }) => {
                self.buffer
                    .delete_at(index + 1)
                    .ok_or_else(|| "invalid row".to_string())?;
                self.buffer
                    .replace_line(index, &previous)
                    .ok_or_else(|| "invalid row".to_string())
            }
            None => Ok(()),
        }
    }

    fn clamp_cursor_col(&mut self) {
        self.view.cursor_col = self.view.cursor_col.min(self.current_line_char_len());
    }

    fn current_line_char_len(&self) -> usize {
        if self.current_row_is_excluded() {
            return 0;
        }
        self.buffer
            .records()
            .get(self.view.cursor_row)
            .map(|record| record.text.chars().count())
            .unwrap_or(0)
    }

    fn bounds_start_col(&self) -> usize {
        self.profile
            .bounds
            .map(|(left, _)| left.saturating_sub(1))
            .unwrap_or(0)
    }

    fn bounds_line_end_col(&self) -> usize {
        let line_len = self.current_line_char_len();

        let Some((left, right)) = self.profile.bounds else {
            return line_len;
        };

        let start = left.saturating_sub(1);
        let end = right.saturating_sub(1);

        if line_len <= start {
            start
        } else {
            line_len.saturating_sub(1).min(end).max(start)
        }
    }

    fn edit_bounds(&self) -> (usize, usize) {
        self.profile
            .bounds
            .map(|(left, right)| (left.saturating_sub(1), right.saturating_sub(1)))
            .unwrap_or((0, usize::MAX))
    }

    fn apply_case_range(&mut self, start: usize, end: usize, uppercase: bool) -> Result<(), String> {
        for index in start..=end {
            let previous = self
                .buffer
                .records()
                .get(index)
                .ok_or_else(|| "invalid row".to_string())?
                .text
                .clone();
            let updated = convert_case_in_bounds(&previous, self.profile.bounds, uppercase);
            self.buffer
                .replace_line(index, &updated)
                .ok_or_else(|| "invalid row".to_string())?;
            self.undo.push(UndoEntry::ReplacedLine { index, previous });
        }
        Ok(())
    }

    fn move_cursor_to_nearest_visible(&mut self, preferred_start: usize, fallback_end: usize) {
        if let Some(index) = self.find_visible_from(preferred_start) {
            self.view.cursor_row = index;
        } else if let Some(index) = self.find_visible_backwards(fallback_end) {
            self.view.cursor_row = index;
        } else {
            self.view.cursor_row = 0;
        }
        self.view.top_row = self.view.top_row.min(self.view.cursor_row);
        self.clamp_cursor_col();
    }

    fn current_row_is_excluded(&self) -> bool {
        self.buffer
            .records()
            .get(self.view.cursor_row)
            .is_some_and(|record| record.excluded)
    }

    fn navigable_row_start(&self, row: usize) -> usize {
        self.excluded_block_start_for(row).unwrap_or(row)
    }

    fn next_navigable_row(&self, row: usize) -> Option<usize> {
        let start = self.navigable_row_start(row).saturating_add(1);
        (start..self.buffer.records().len()).find(|&index| self.is_navigable_row(index))
    }

    fn previous_navigable_row(&self, row: usize) -> Option<usize> {
        let current = self.navigable_row_start(row);
        (0..current).rev().find(|&index| self.is_navigable_row(index))
    }

    fn is_navigable_row(&self, row: usize) -> bool {
        let Some(record) = self.buffer.records().get(row) else {
            return false;
        };

        if !record.excluded {
            return true;
        }

        row == 0
            || self
                .buffer
                .records()
                .get(row - 1)
                .is_some_and(|previous| !previous.excluded)
    }

    fn excluded_block_start_for(&self, row: usize) -> Option<usize> {
        let records = self.buffer.records();
        if !records.get(row)?.excluded {
            return None;
        }

        let mut start = row;
        while start > 0 && records[start - 1].excluded {
            start -= 1;
        }
        Some(start)
    }

    fn excluded_block_range_at(&self, row: usize) -> Option<(usize, usize)> {
        let records = self.buffer.records();
        let start = self.excluded_block_start_for(row)?;
        let mut end = start;
        while end + 1 < records.len() && records[end + 1].excluded {
            end += 1;
        }
        Some((start, end))
    }

    fn find_visible_from(&self, start: usize) -> Option<usize> {
        self.buffer
            .records()
            .iter()
            .enumerate()
            .skip(start)
            .find_map(|(index, record)| (!record.excluded).then_some(index))
    }

    fn find_visible_backwards(&self, end: usize) -> Option<usize> {
        let max_index = end.min(self.buffer.records().len().saturating_sub(1));
        self.buffer
            .records()
            .iter()
            .enumerate()
            .take(max_index.saturating_add(1))
            .rev()
            .find_map(|(index, record)| (!record.excluded).then_some(index))
    }

    fn try_complete_pending_transfer(&mut self) -> Result<(), String> {
        if let (Some((start, end)), Some((target_start, target_end))) =
            (self.pending_copy_range, self.pending_overlay_range)
        {
            if let Err(err) = self.apply_copy_overlay_range(start, end, target_start, target_end) {
                self.message = Some(SessionMessage {
                    text: err.clone(),
                    is_error: true,
                });
                return Err(err);
            }
            self.pending_copy_range = None;
            self.pending_copy_display = None;
            self.pending_overlay_range = None;
            self.pending_overlay_display = None;
            return Ok(());
        }

        if let (Some((start, end)), Some((target_start, target_end))) =
            (self.pending_move_range, self.pending_overlay_range)
        {
            if let Err(err) = self.apply_move_overlay_range(start, end, target_start, target_end) {
                self.message = Some(SessionMessage {
                    text: err.clone(),
                    is_error: true,
                });
                return Err(err);
            }
            self.pending_move_range = None;
            self.pending_move_display = None;
            self.pending_overlay_range = None;
            self.pending_overlay_display = None;
            return Ok(());
        }

        if let (Some((start, end)), Some(destination)) =
            (self.pending_copy_range, self.pending_destination)
        {
            self.apply_copy_range(start, end, destination);
            self.pending_copy_range = None;
            self.pending_copy_display = None;
            self.pending_destination = None;
            return Ok(());
        }

        if let (Some((start, end)), Some(destination)) =
            (self.pending_move_range, self.pending_destination)
        {
            self.apply_move_range(start, end, destination)?;
            self.pending_move_range = None;
            self.pending_move_display = None;
            self.pending_destination = None;
        }

        Ok(())
    }

    fn apply_copy_range(&mut self, start: usize, end: usize, destination: Destination) {
        let lines: Vec<String> = self.buffer.records()[start..=end]
            .iter()
            .map(|record| record.text.clone())
            .collect();
        let insert_at = match destination {
            Destination::After(row) => row.saturating_add(1).min(self.buffer.records().len()),
            Destination::Before(row) => row.min(self.buffer.records().len()),
            Destination::Overlay(_) => return,
        };

        for (offset, text) in lines.iter().enumerate() {
            self.buffer.insert_before(insert_at + offset, text);
        }

        self.view.cursor_row = insert_at.min(self.buffer.records().len().saturating_sub(1));
        self.message = Some(SessionMessage {
            text: match lines.len() {
                1 => "1 line copied".into(),
                count => format!("{count} lines copied"),
            },
            is_error: false,
        });
    }

    fn apply_move_range(
        &mut self,
        start: usize,
        end: usize,
        destination: Destination,
    ) -> Result<(), String> {
        let destination_row = match destination {
            Destination::After(row) | Destination::Before(row) | Destination::Overlay(row) => row,
        };
        if (start..=end).contains(&destination_row) {
            self.message = Some(SessionMessage {
                text: "Move destination cannot be inside the moved block".into(),
                is_error: true,
            });
            return Err("move destination cannot be inside the moved block".into());
        }

        let moved_count = end - start + 1;
        let mut moved = Vec::with_capacity(moved_count);
        for _ in 0..moved_count {
            let record = self
                .buffer
                .delete_at(start)
                .ok_or_else(|| "invalid row".to_string())?;
            moved.push(record);
        }

        let adjusted_destination = if destination_row > end {
            destination_row - moved_count
        } else {
            destination_row
        };
        let insert_at = match destination {
            Destination::After(_) => adjusted_destination.saturating_add(1),
            Destination::Before(_) => adjusted_destination,
            Destination::Overlay(_) => adjusted_destination,
        };

        for (offset, record) in moved.into_iter().enumerate() {
            self.buffer.insert_record_at(insert_at + offset, record);
        }

        self.view.cursor_row = insert_at.min(self.buffer.records().len().saturating_sub(1));
        self.message = Some(SessionMessage {
            text: match moved_count {
                1 => "1 line moved".into(),
                count => format!("{count} lines moved"),
            },
            is_error: false,
        });
        Ok(())
    }

    fn apply_copy_overlay_range(
        &mut self,
        start: usize,
        end: usize,
        target_start: usize,
        target_end: usize,
    ) -> Result<(), String> {
        let lines: Vec<String> = self.buffer.records()[start..=end]
            .iter()
            .map(|record| record.text.clone())
            .collect();
        let target_rows =
            overlay_target_rows(lines.len(), target_start, target_end, self.buffer.records().len())?;

        for (source, target_row) in lines.iter().zip(target_rows.iter().copied()) {
            let previous = self
                .buffer
                .records()
                .get(target_row)
                .ok_or_else(|| "invalid row".to_string())?
                .text
                .clone();
            let updated = overlay_non_blank_in_bounds(&previous, source, self.profile.bounds);
            self.buffer
                .replace_line(target_row, &updated)
                .ok_or_else(|| "invalid row".to_string())?;
            self.undo.push(UndoEntry::ReplacedLine {
                index: target_row,
                previous,
            });
        }

        self.view.cursor_row = target_rows[0];
        self.message = Some(SessionMessage {
            text: match lines.len() {
                1 => "1 line overlaid".into(),
                count => format!("{count} lines overlaid"),
            },
            is_error: false,
        });
        Ok(())
    }

    fn apply_move_overlay_range(
        &mut self,
        start: usize,
        end: usize,
        target_start: usize,
        target_end: usize,
    ) -> Result<(), String> {
        let moved_count = end - start + 1;
        let lines: Vec<String> = self.buffer.records()[start..=end]
            .iter()
            .map(|record| record.text.clone())
            .collect();
        let target_rows =
            overlay_target_rows(lines.len(), target_start, target_end, self.buffer.records().len())?;

        if target_rows.iter().any(|row| (start..=end).contains(row)) {
            self.message = Some(SessionMessage {
                text: "Overlay target cannot overlap moved lines".into(),
                is_error: true,
            });
            return Err("overlay target cannot overlap moved lines".into());
        }

        for (source, target_row) in lines.iter().zip(target_rows.iter().copied()) {
            let previous = self
                .buffer
                .records()
                .get(target_row)
                .ok_or_else(|| "invalid row".to_string())?
                .text
                .clone();
            let updated = overlay_non_blank_in_bounds(&previous, source, self.profile.bounds);
            self.buffer
                .replace_line(target_row, &updated)
                .ok_or_else(|| "invalid row".to_string())?;
        }

        for _ in 0..moved_count {
            self.buffer
                .delete_at(start)
                .ok_or_else(|| "invalid row".to_string())?;
        }

        let adjusted_row = if start < target_rows[0] {
            target_rows[0].saturating_sub(moved_count)
        } else {
            target_rows[0]
        };
        self.view.cursor_row = adjusted_row.min(self.buffer.records().len().saturating_sub(1));
        self.message = Some(SessionMessage {
            text: match moved_count {
                1 => "1 line moved with overlay".into(),
                count => format!("{count} lines moved with overlay"),
            },
            is_error: false,
        });
        Ok(())
    }
}

fn insert_char_at(text: &str, column: usize, ch: char) -> Result<String, String> {
    let char_count = text.chars().count();
    if column > char_count {
        return Err("invalid column".to_string());
    }

    let mut chars: Vec<char> = text.chars().collect();
    chars.insert(column, ch);
    Ok(chars.into_iter().collect())
}

fn overwrite_char_at(text: &str, column: usize, ch: char) -> Result<String, String> {
    let char_count = text.chars().count();
    if column > char_count {
        return Err("invalid column".to_string());
    }

    if column == char_count {
        return insert_char_at(text, column, ch);
    }

    let mut chars: Vec<char> = text.chars().collect();
    chars[column] = ch;
    Ok(chars.into_iter().collect())
}

fn remove_char_before(text: &str, column: usize) -> Result<String, String> {
    let char_count = text.chars().count();
    if column == 0 || column > char_count {
        return Err("invalid column".to_string());
    }

    let mut chars: Vec<char> = text.chars().collect();
    chars.remove(column - 1);
    Ok(chars.into_iter().collect())
}

fn remove_char_at(text: &str, column: usize) -> Result<String, String> {
    let char_count = text.chars().count();
    if column >= char_count {
        return Err("invalid column".to_string());
    }

    let mut chars: Vec<char> = text.chars().collect();
    chars.remove(column);
    Ok(chars.into_iter().collect())
}

fn split_text_at(text: &str, column: usize) -> Result<(String, String), String> {
    let char_count = text.chars().count();
    if column > char_count {
        return Err("invalid column".to_string());
    }

    let chars: Vec<char> = text.chars().collect();
    let left = chars[..column].iter().collect();
    let right = chars[column..].iter().collect();
    Ok((left, right))
}

fn bounded_char_range(text: &str, bounds: Option<(usize, usize)>) -> (usize, usize) {
    let len = text.chars().count();
    match bounds {
        Some((left, right)) => {
            let start = left.saturating_sub(1).min(len);
            let end = right.min(len);
            (start.min(end), end)
        }
        None => (0, len),
    }
}

fn record_text_in_bounds(text: &str, bounds: Option<(usize, usize)>) -> String {
    let (start, end) = bounded_char_range(text, bounds);
    text.chars().skip(start).take(end.saturating_sub(start)).collect()
}

fn char_to_byte_index(text: &str, char_index: usize) -> usize {
    if char_index == 0 {
        return 0;
    }
    text.char_indices()
        .nth(char_index)
        .map(|(idx, _)| idx)
        .unwrap_or(text.len())
}

fn replace_first_in_bounds(
    text: &str,
    from: &str,
    to: &str,
    bounds: Option<(usize, usize)>,
) -> Option<String> {
    let (start_char, end_char) = bounded_char_range(text, bounds);
    let start_byte = char_to_byte_index(text, start_char);
    let end_byte = char_to_byte_index(text, end_char);
    let bounded = &text[start_byte..end_byte];
    let found = bounded.find(from)?;
    let absolute = start_byte + found;
    let after = absolute + from.len();
    Some(format!("{}{}{}", &text[..absolute], to, &text[after..]))
}

fn convert_case_in_bounds(text: &str, bounds: Option<(usize, usize)>, uppercase: bool) -> String {
    let (start, end) = bounded_char_range(text, bounds);
    text.chars()
        .enumerate()
        .map(|(index, ch)| {
            if (start..end).contains(&index) {
                if uppercase {
                    ch.to_ascii_uppercase()
                } else {
                    ch.to_ascii_lowercase()
                }
            } else {
                ch
            }
        })
        .collect()
}

fn overlay_target_rows(
    source_len: usize,
    target_start: usize,
    target_end: usize,
    total_rows: usize,
) -> Result<Vec<usize>, String> {
    if source_len == 0 {
        return Err("overlay requires at least one source line".into());
    }

    if target_start >= total_rows {
        return Err("invalid row".into());
    }

    if target_start == target_end {
        let last = target_start + source_len - 1;
        if last >= total_rows {
            return Err("Overlay target must fit within the buffer".into());
        }
        return Ok((target_start..=last).collect());
    }

    let target_len = target_end - target_start + 1;
    if target_len != source_len {
        return Err("Overlay target must match source line count".into());
    }

    Ok((target_start..=target_end).collect())
}

fn overlay_non_blank_in_bounds(dest: &str, source: &str, bounds: Option<(usize, usize)>) -> String {
    let mut dest_chars: Vec<char> = dest.chars().collect();
    let source_chars: Vec<char> = source.chars().collect();
    let (start, end) = bounded_char_range(source, bounds);
    let target_len = dest_chars.len().max(source_chars.len());
    dest_chars.resize(target_len, ' ');

    for index in start..end.min(source_chars.len()) {
        if source_chars[index] != ' ' {
            dest_chars[index] = source_chars[index];
        }
    }

    dest_chars.into_iter().collect()
}
