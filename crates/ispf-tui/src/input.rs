use crossterm::event::{KeyCode, KeyEvent};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AppAction {
    MoveUp,
    MoveDown,
    ScrollLeft,
    ScrollRight,
    End,
    Execute,
    None,
}

pub fn map_key(event: KeyEvent) -> AppAction {
    match event.code {
        KeyCode::Up => AppAction::MoveUp,
        KeyCode::Down => AppAction::MoveDown,
        KeyCode::F(7) => AppAction::MoveUp,
        KeyCode::F(8) => AppAction::MoveDown,
        KeyCode::F(10) => AppAction::ScrollLeft,
        KeyCode::F(11) => AppAction::ScrollRight,
        KeyCode::F(3) => AppAction::End,
        KeyCode::Enter => AppAction::Execute,
        _ => AppAction::None,
    }
}

#[cfg(test)]
mod tests {
    use super::{map_key, AppAction};
    use crossterm::event::{KeyCode, KeyEvent};

    #[test]
    fn maps_pf_navigation_keys() {
        assert_eq!(map_key(KeyEvent::from(KeyCode::F(7))), AppAction::MoveUp);
        assert_eq!(map_key(KeyEvent::from(KeyCode::F(8))), AppAction::MoveDown);
        assert_eq!(
            map_key(KeyEvent::from(KeyCode::F(10))),
            AppAction::ScrollLeft
        );
        assert_eq!(
            map_key(KeyEvent::from(KeyCode::F(11))),
            AppAction::ScrollRight
        );
    }
}
