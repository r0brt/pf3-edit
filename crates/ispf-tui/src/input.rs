use crossterm::event::{KeyCode, KeyEvent};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AppAction {
    MoveUp,
    MoveDown,
    End,
    Execute,
    None,
}

pub fn map_key(event: KeyEvent) -> AppAction {
    match event.code {
        KeyCode::Up => AppAction::MoveUp,
        KeyCode::Down => AppAction::MoveDown,
        KeyCode::F(3) => AppAction::End,
        KeyCode::Enter => AppAction::Execute,
        _ => AppAction::None,
    }
}
