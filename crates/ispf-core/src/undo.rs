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
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn push(&mut self, entry: UndoEntry) {
        self.entries.push(entry);
    }

    pub fn pop(&mut self) -> Option<UndoEntry> {
        self.entries.pop()
    }
}
