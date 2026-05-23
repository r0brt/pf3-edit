#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ActiveArea {
    DataArea,
}

#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub struct ViewState;

#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub struct SessionMessage;

#[derive(Debug, Default)]
pub struct EditorSession;
