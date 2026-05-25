mod editing;
mod navigation;
mod transfers;

use self::editing::{find_first_in_bounds, replace_first_in_bounds};
use crate::{CapsMode, EditBuffer, EditProfile, UndoEntry, UndoStack};
use ispf_command::{PrefixCommand, PrimaryCommand, ScrollMode};

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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TextEntryMode {
    pub start_row: usize,
    pub end_row: usize,
}

pub struct EditorSession {
    buffer: EditBuffer,
    original_buffer: EditBuffer,
    view: ViewState,
    desired_cursor_col: usize,
    scroll_rows_hint: usize,
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
    text_entry: Option<TextEntryMode>,
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
            desired_cursor_col: 0,
            scroll_rows_hint: 18,
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
            text_entry: None,
            should_exit: false,
            exit_disposition: ExitDisposition::KeepChanges,
        }
    }

    pub fn view(&self) -> &ViewState {
        &self.view
    }

    pub fn set_scroll_rows_hint(&mut self, rows: usize) {
        self.scroll_rows_hint = rows.max(1);
    }

    pub fn buffer(&self) -> &EditBuffer {
        &self.buffer
    }

    pub fn toggle_active_area(&mut self) {
        self.view.active_area = match self.view.active_area {
            ActiveArea::CommandLine => ActiveArea::LineCommandArea,
            ActiveArea::LineCommandArea => ActiveArea::DataArea,
            ActiveArea::DataArea => ActiveArea::CommandLine,
        };
    }

    pub fn toggle_active_area_backward(&mut self) {
        self.view.active_area = match self.view.active_area {
            ActiveArea::CommandLine => ActiveArea::DataArea,
            ActiveArea::LineCommandArea => ActiveArea::CommandLine,
            ActiveArea::DataArea => ActiveArea::LineCommandArea,
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
        self.desired_cursor_col = 0;
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
                self.text_entry = None;
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
                if let Some((index, col)) = self
                    .buffer
                    .records()
                    .iter()
                    .enumerate()
                    .find_map(|(index, record)| {
                        find_first_in_bounds(record.text.as_str(), &pattern, self.profile.bounds)
                            .map(|col| (index, col))
                    })
                {
                    self.view.cursor_row = index;
                    self.view.top_row = index;
                    self.view.cursor_col = col;
                    self.desired_cursor_col = col;
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
                if let Some((index, col)) = self
                    .buffer
                    .records()
                    .iter()
                    .enumerate()
                    .find_map(|(index, record)| {
                        find_first_in_bounds(record.text.as_str(), &from, self.profile.bounds)
                            .map(|col| (index, col))
                    })
                {
                    let previous = self.buffer.records()[index].text.clone();
                    let updated = replace_first_in_bounds(&previous, &from, &to, self.profile.bounds)
                        .ok_or_else(|| "Pattern not found".to_string())?;
                    self.buffer
                        .replace_line(index, &updated)
                        .ok_or_else(|| "invalid row".to_string())?;
                    self.view.cursor_row = index;
                    self.view.top_row = index;
                    self.view.cursor_col = col;
                    self.desired_cursor_col = col;
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
            PrimaryCommand::Scroll(mode) => self.profile.scroll_mode = mode,
            PrimaryCommand::Down(count) => {
                let step = count.unwrap_or_else(|| self.effective_vertical_scroll_rows());
                self.view.top_row = self.view.top_row.saturating_add(step);
                self.clamp_top_row();
            }
            PrimaryCommand::Up(count) => {
                let step = count.unwrap_or_else(|| self.effective_vertical_scroll_rows());
                self.view.top_row = self.view.top_row.saturating_sub(step);
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
            PrefixCommand::TextSplit(blank_lines) => {
                self.text_split_at_cursor(row, blank_lines)?;
                self.message = Some(SessionMessage {
                    text: match blank_lines {
                        0 => "Text split completed".into(),
                        count => format!("Text split completed with {count} blank lines"),
                    },
                    is_error: false,
                });
            }
            PrefixCommand::TextFlow(width) => {
                self.text_flow_paragraph(row, width)?;
                self.message = Some(SessionMessage {
                    text: "Text flow completed".into(),
                    is_error: false,
                });
            }
            PrefixCommand::TextEntry(extra_lines) => {
                self.begin_text_entry(row, extra_lines)?;
                self.message = Some(SessionMessage {
                    text: match extra_lines {
                        0 => "Text entry active".into(),
                        count => format!("Text entry active with {count} extra lines"),
                    },
                    is_error: false,
                });
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

    pub fn text_entry_mode(&self) -> Option<TextEntryMode> {
        self.text_entry
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
}
