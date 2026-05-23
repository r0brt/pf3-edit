use crate::{EditBuffer, EditProfile, UndoStack};

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

    pub fn profile(&self) -> &EditProfile {
        &self.profile
    }
}
