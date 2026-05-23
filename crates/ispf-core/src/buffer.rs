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

#[derive(Debug, Default)]
pub struct EditBuffer {
    records: Vec<Record>,
    dirty: bool,
    next_id: u64,
    file_path: Option<std::path::PathBuf>,
    newline: &'static str,
}

impl EditBuffer {
    pub fn from_text(input: &str) -> Self {
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
        Self {
            records,
            dirty: false,
            next_id,
            file_path: None,
            newline: "\n",
        }
    }

    pub fn from_path(path: &std::path::Path) -> std::io::Result<Self> {
        let text = std::fs::read_to_string(path)?;
        let mut buffer = Self::from_text(&text);
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
        let path = self.file_path.clone().ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::NotFound, "missing file path")
        })?;
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
        out.push_str(self.newline);
        out
    }

    pub fn insert_after(&mut self, index: usize, text: &str) {
        let record = Record {
            id: RecordId(self.next_id),
            text: text.to_string(),
            excluded: false,
        };
        self.next_id += 1;
        self.records.insert(index + 1, record);
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
        record.excluded = excluded;
        self.dirty = true;
        Some(())
    }
}
