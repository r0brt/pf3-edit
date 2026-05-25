use super::*;
use crate::Record;

impl EditorSession {
    pub fn insert_blank_line_after(&mut self, row: usize) {
        self.buffer.insert_after(row, "");
        let inserted_index =
            row.saturating_add(1).min(self.buffer.records().len().saturating_sub(1));
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

    pub(super) fn undo_last(&mut self) -> Result<(), String> {
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
            Some(UndoEntry::TextSplit {
                index,
                previous,
                blank_lines,
            }) => {
                for _ in 0..=blank_lines {
                    self.buffer
                        .delete_at(index + 1)
                        .ok_or_else(|| "invalid row".to_string())?;
                }
                self.buffer
                    .replace_line(index, &previous)
                    .ok_or_else(|| "invalid row".to_string())
            }
            Some(UndoEntry::ReflowParagraph {
                start,
                previous,
                new_len,
            }) => {
                for _ in 0..new_len {
                    self.buffer
                        .delete_at(start)
                        .ok_or_else(|| "invalid row".to_string())?;
                }
                for (offset, record) in previous.into_iter().enumerate() {
                    self.buffer.insert_record_at(start + offset, record);
                }
                Ok(())
            }
            None => Ok(()),
        }
    }

    pub(super) fn clamp_cursor_col(&mut self) {
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

    pub(super) fn bounds_start_col(&self) -> usize {
        self.profile
            .bounds
            .map(|(left, _)| left.saturating_sub(1))
            .unwrap_or(0)
    }

    pub(super) fn bounds_line_end_col(&self) -> usize {
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

    pub(super) fn edit_bounds(&self) -> (usize, usize) {
        self.profile
            .bounds
            .map(|(left, right)| (left.saturating_sub(1), right.saturating_sub(1)))
            .unwrap_or((0, usize::MAX))
    }

    pub(super) fn apply_case_range(
        &mut self,
        start: usize,
        end: usize,
        uppercase: bool,
    ) -> Result<(), String> {
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

    pub(super) fn text_split_at_cursor(
        &mut self,
        row: usize,
        blank_lines: usize,
    ) -> Result<(), String> {
        if self.current_row_is_excluded() {
            return Err("Cannot edit excluded lines".into());
        }

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
        for offset in 0..blank_lines {
            self.buffer.insert_after(row + offset, "");
        }
        self.buffer.insert_after(row + blank_lines, &right);
        self.undo.push(UndoEntry::TextSplit {
            index: row,
            previous: current,
            blank_lines,
        });
        self.view.cursor_row = row + blank_lines + 1;
        self.view.cursor_col = 0;
        Ok(())
    }

    pub(super) fn text_flow_paragraph(
        &mut self,
        start: usize,
        requested_width: Option<usize>,
    ) -> Result<(), String> {
        let end = self.paragraph_end(start);
        if end < start {
            return Ok(());
        }

        let previous: Vec<Record> = self.buffer.records()[start..=end].to_vec();
        let width = self.text_flow_width(requested_width);
        let flowed = flow_paragraph_lines(&previous, self.profile.bounds, width);
        let new_len = flowed.len();

        for _ in start..=end {
            self.buffer
                .delete_at(start)
                .ok_or_else(|| "invalid row".to_string())?;
        }
        for (offset, line) in flowed.iter().enumerate() {
            self.buffer.insert_before(start + offset, line);
        }

        self.undo.push(UndoEntry::ReflowParagraph {
            start,
            previous,
            new_len,
        });
        self.view.cursor_row = start;
        self.view.cursor_col = self.bounds_start_col();
        Ok(())
    }

    pub(super) fn begin_text_entry(
        &mut self,
        row: usize,
        extra_lines: usize,
    ) -> Result<(), String> {
        if self
            .buffer
            .records()
            .get(row)
            .ok_or_else(|| "invalid row".to_string())?
            .excluded
        {
            return Err("Cannot edit excluded lines".into());
        }

        for offset in 0..extra_lines {
            self.buffer.insert_after(row + offset, "");
            self.undo.push(UndoEntry::InsertedLine {
                index: row + offset + 1,
            });
        }

        self.text_entry = Some(TextEntryMode {
            start_row: row,
            end_row: row + extra_lines,
        });
        self.view.cursor_row = row;
        self.view.cursor_col = self.bounds_start_col();
        Ok(())
    }

    pub fn end_text_entry(&mut self) {
        self.text_entry = None;
    }

    pub fn text_entry_insert_char(&mut self, ch: char) -> Result<(), String> {
        let Some(mut mode) = self.text_entry else {
            return self.insert_char(ch);
        };

        if self.current_row_is_excluded() {
            return Err("Cannot edit excluded lines".into());
        }

        let (bounds_start, bounds_end) = self.edit_bounds();
        if self.view.cursor_col < bounds_start {
            self.view.cursor_col = bounds_start;
        }
        if self.view.cursor_col > bounds_end {
            self.advance_text_entry_row(&mut mode)?;
            self.view.cursor_col = bounds_start;
        }

        let row = self.view.cursor_row;
        let previous = self
            .buffer
            .records()
            .get(row)
            .ok_or_else(|| "invalid row".to_string())?
            .text
            .clone();
        let updated = write_char_with_padding(&previous, self.view.cursor_col, ch);
        self.buffer
            .replace_line(row, &updated)
            .ok_or_else(|| "invalid row".to_string())?;
        self.undo.push(UndoEntry::ReplacedLine {
            index: row,
            previous,
        });
        self.view.cursor_col += 1;
        self.text_entry = Some(mode);
        Ok(())
    }

    fn advance_text_entry_row(&mut self, mode: &mut TextEntryMode) -> Result<(), String> {
        if self.view.cursor_row >= mode.end_row {
            let insert_at = mode.end_row;
            self.buffer.insert_after(insert_at, "");
            self.undo.push(UndoEntry::InsertedLine {
                index: insert_at + 1,
            });
            mode.end_row += 1;
        }

        self.view.cursor_row = self.view.cursor_row.saturating_add(1).min(mode.end_row);
        Ok(())
    }

    fn paragraph_end(&self, start: usize) -> usize {
        let records = self.buffer.records();
        let mut end = start;
        while end < records.len() {
            let record = &records[end];
            if record.excluded || record.text.trim().is_empty() {
                break;
            }
            end += 1;
        }

        end.saturating_sub(1).max(start)
    }

    fn text_flow_width(&self, requested_width: Option<usize>) -> usize {
        let bounds_width = self
            .profile
            .bounds
            .map(|(left, right)| right.saturating_sub(left).saturating_add(1))
            .unwrap_or(72);

        requested_width
            .map(|width| width.min(bounds_width).max(1))
            .unwrap_or(bounds_width.max(1))
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

fn write_char_with_padding(text: &str, column: usize, ch: char) -> String {
    let mut chars: Vec<char> = text.chars().collect();
    while chars.len() < column {
        chars.push(' ');
    }

    if column == chars.len() {
        chars.push(ch);
    } else {
        chars[column] = ch;
    }

    chars.into_iter().collect()
}

pub(super) fn bounded_char_range(text: &str, bounds: Option<(usize, usize)>) -> (usize, usize) {
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

pub(super) fn record_text_in_bounds(text: &str, bounds: Option<(usize, usize)>) -> String {
    let (start, end) = bounded_char_range(text, bounds);
    text.chars()
        .skip(start)
        .take(end.saturating_sub(start))
        .collect()
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

pub(super) fn replace_first_in_bounds(
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

fn flow_paragraph_lines(records: &[Record], bounds: Option<(usize, usize)>, width: usize) -> Vec<String> {
    let left_padding = bounds.map(|(left, _)| left.saturating_sub(1)).unwrap_or(0);
    let words: Vec<String> = records
        .iter()
        .flat_map(|record| {
            record
                .text
                .split_whitespace()
                .map(str::to_string)
                .collect::<Vec<_>>()
        })
        .collect();

    if words.is_empty() {
        return vec![String::new()];
    }

    let mut lines = Vec::new();
    let mut current = String::new();

    for word in words {
        let next_len = if current.is_empty() {
            word.len()
        } else {
            current.len() + 1 + word.len()
        };

        if !current.is_empty() && next_len > width {
            lines.push(format!("{}{}", " ".repeat(left_padding), current));
            current = word;
        } else if current.is_empty() {
            current = word;
        } else {
            current.push(' ');
            current.push_str(&word);
        }
    }

    if !current.is_empty() {
        lines.push(format!("{}{}", " ".repeat(left_padding), current));
    }

    lines
}
