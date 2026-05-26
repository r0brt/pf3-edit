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

#[derive(Clone, Debug)]
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

    pub fn file_path(&self) -> Option<&std::path::Path> {
        self.file_path.as_deref()
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
        if self.trailing_newline {
            out.push_str(self.newline);
        }
        out
    }

    pub fn insert_after(&mut self, index: usize, text: &str) {
        let insert_at = if self.records.is_empty() {
            0
        } else {
            index.saturating_add(1).min(self.records.len())
        };
        self.insert_at(insert_at, text);
    }

    pub(crate) fn insert_at(&mut self, index: usize, text: &str) {
        let record = Record {
            id: RecordId(self.next_id),
            text: text.to_string(),
            excluded: false,
        };
        self.next_id += 1;
        self.insert_record_at(index, record);
    }

    pub fn insert_before(&mut self, index: usize, text: &str) {
        self.insert_at(index.min(self.records.len()), text);
    }

    pub(crate) fn insert_record_at(&mut self, index: usize, record: Record) {
        self.next_id = self.next_id.max(record.id.0.saturating_add(1));
        let insert_at = index.min(self.records.len());
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

    pub(crate) fn replace_line(&mut self, index: usize, text: &str) -> Option<()> {
        let record = self.records.get_mut(index)?;
        if record.text == text {
            return Some(());
        }
        record.text = text.to_string();
        self.dirty = true;
        Some(())
    }
}

fn detect_newline(input: &str) -> &'static str {
    if input.contains("\r\n") { "\r\n" } else { "\n" }
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
