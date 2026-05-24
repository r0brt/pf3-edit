use super::*;

impl EditorSession {
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

    pub fn row_is_excluded(&self, row: usize) -> bool {
        self.buffer
            .records()
            .get(row)
            .is_some_and(|record| record.excluded)
    }

    pub(super) fn move_cursor_to_nearest_visible(
        &mut self,
        preferred_start: usize,
        fallback_end: usize,
    ) {
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

    pub(super) fn current_row_is_excluded(&self) -> bool {
        self.buffer
            .records()
            .get(self.view.cursor_row)
            .is_some_and(|record| record.excluded)
    }

    pub(super) fn navigable_row_start(&self, row: usize) -> usize {
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

    pub(super) fn excluded_block_range_at(&self, row: usize) -> Option<(usize, usize)> {
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
}
