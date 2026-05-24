use super::*;

impl EditorSession {
    pub(super) fn try_complete_pending_transfer(&mut self) -> Result<(), String> {
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
    let (start, end) = super::editing::bounded_char_range(source, bounds);
    let target_len = dest_chars.len().max(source_chars.len());
    dest_chars.resize(target_len, ' ');

    for index in start..end.min(source_chars.len()) {
        if source_chars[index] != ' ' {
            dest_chars[index] = source_chars[index];
        }
    }

    dest_chars.into_iter().collect()
}
