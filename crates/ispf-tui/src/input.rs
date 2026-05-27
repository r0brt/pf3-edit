use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AppAction {
    Help,
    Split,
    ExitSave,
    CursorUp,
    CursorDown,
    CursorLeft,
    CursorRight,
    CursorLineStart,
    CursorLineEnd,
    RepeatFind,
    RepeatChange,
    ScrollUp,
    ScrollDown,
    ScrollLeft,
    ScrollRight,
    Swap,
    ToggleFocus,
    ToggleFocusBackward,
    Backspace,
    Delete,
    LineFeed,
    Cancel,
    End,
    Execute,
    None,
}

pub fn map_key(event: KeyEvent) -> AppAction {
    if event.code == KeyCode::Enter && event.modifiers.contains(KeyModifiers::SHIFT) {
        return AppAction::LineFeed;
    }

    if matches!(event.code, KeyCode::Char('j') | KeyCode::Char('J'))
        && event.modifiers.contains(KeyModifiers::CONTROL)
    {
        return AppAction::CursorDown;
    }

    if matches!(event.code, KeyCode::Char('a') | KeyCode::Char('A'))
        && event.modifiers.contains(KeyModifiers::CONTROL)
    {
        return AppAction::CursorLineStart;
    }

    if matches!(event.code, KeyCode::Char('e') | KeyCode::Char('E'))
        && event.modifiers.contains(KeyModifiers::CONTROL)
    {
        return AppAction::CursorLineEnd;
    }

    match event.code {
        KeyCode::F(1) => AppAction::Help,
        KeyCode::F(2) => AppAction::Split,
        KeyCode::F(3) => AppAction::ExitSave,
        KeyCode::Up => AppAction::CursorUp,
        KeyCode::Down => AppAction::CursorDown,
        KeyCode::Left => AppAction::CursorLeft,
        KeyCode::Right => AppAction::CursorRight,
        KeyCode::Home => AppAction::CursorLineStart,
        KeyCode::End => AppAction::CursorLineEnd,
        KeyCode::F(5) => AppAction::RepeatFind,
        KeyCode::F(6) => AppAction::RepeatChange,
        KeyCode::F(7) => AppAction::ScrollUp,
        KeyCode::F(8) => AppAction::ScrollDown,
        KeyCode::F(9) => AppAction::Swap,
        KeyCode::F(10) => AppAction::ScrollLeft,
        KeyCode::F(11) => AppAction::ScrollRight,
        KeyCode::F(12) => AppAction::Cancel,
        KeyCode::Esc => AppAction::End,
        KeyCode::BackTab => AppAction::ToggleFocusBackward,
        KeyCode::Tab => AppAction::ToggleFocus,
        KeyCode::Backspace => AppAction::Backspace,
        KeyCode::Delete => AppAction::Delete,
        KeyCode::Enter => AppAction::Execute,
        _ => AppAction::None,
    }
}

#[cfg(test)]
mod tests {
    use super::{AppAction, map_key};
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    #[test]
    fn maps_pf_navigation_keys() {
        assert_eq!(map_key(KeyEvent::from(KeyCode::F(1))), AppAction::Help);
        assert_eq!(map_key(KeyEvent::from(KeyCode::F(2))), AppAction::Split);
        assert_eq!(map_key(KeyEvent::from(KeyCode::F(3))), AppAction::ExitSave);
        assert_eq!(map_key(KeyEvent::from(KeyCode::Up)), AppAction::CursorUp);
        assert_eq!(
            map_key(KeyEvent::from(KeyCode::Down)),
            AppAction::CursorDown
        );
        assert_eq!(
            map_key(KeyEvent::from(KeyCode::Left)),
            AppAction::CursorLeft
        );
        assert_eq!(
            map_key(KeyEvent::from(KeyCode::Right)),
            AppAction::CursorRight
        );
        assert_eq!(
            map_key(KeyEvent::from(KeyCode::Home)),
            AppAction::CursorLineStart
        );
        assert_eq!(
            map_key(KeyEvent::from(KeyCode::End)),
            AppAction::CursorLineEnd
        );
        assert_eq!(
            map_key(KeyEvent::from(KeyCode::F(5))),
            AppAction::RepeatFind
        );
        assert_eq!(
            map_key(KeyEvent::from(KeyCode::F(6))),
            AppAction::RepeatChange
        );
        assert_eq!(map_key(KeyEvent::from(KeyCode::F(7))), AppAction::ScrollUp);
        assert_eq!(
            map_key(KeyEvent::from(KeyCode::F(8))),
            AppAction::ScrollDown
        );
        assert_eq!(map_key(KeyEvent::from(KeyCode::F(9))), AppAction::Swap);
        assert_eq!(
            map_key(KeyEvent::from(KeyCode::F(10))),
            AppAction::ScrollLeft
        );
        assert_eq!(
            map_key(KeyEvent::from(KeyCode::F(11))),
            AppAction::ScrollRight
        );
    }

    #[test]
    fn maps_backspace() {
        assert_eq!(
            map_key(KeyEvent::from(KeyCode::Backspace)),
            AppAction::Backspace
        );
        assert_eq!(map_key(KeyEvent::from(KeyCode::Delete)), AppAction::Delete);
        assert_eq!(map_key(KeyEvent::from(KeyCode::F(12))), AppAction::Cancel);
    }

    #[test]
    fn maps_escape_to_end() {
        assert_eq!(map_key(KeyEvent::from(KeyCode::Esc)), AppAction::End);
    }

    #[test]
    fn maps_tab_to_toggle_focus() {
        assert_eq!(
            map_key(KeyEvent::from(KeyCode::Tab)),
            AppAction::ToggleFocus
        );
        assert_eq!(
            map_key(KeyEvent::from(KeyCode::BackTab)),
            AppAction::ToggleFocusBackward
        );
    }

    #[test]
    fn maps_shift_enter_to_line_feed() {
        assert_eq!(
            map_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::SHIFT)),
            AppAction::LineFeed
        );
    }

    #[test]
    fn maps_ctrl_j_to_cursor_down() {
        assert_eq!(
            map_key(KeyEvent::new(KeyCode::Char('j'), KeyModifiers::CONTROL)),
            AppAction::CursorDown
        );
        assert_eq!(
            map_key(KeyEvent::new(KeyCode::Char('J'), KeyModifiers::CONTROL)),
            AppAction::CursorDown
        );
    }

    #[test]
    fn maps_ctrl_a_and_ctrl_e_to_line_start_and_end() {
        assert_eq!(
            map_key(KeyEvent::new(KeyCode::Char('a'), KeyModifiers::CONTROL)),
            AppAction::CursorLineStart
        );
        assert_eq!(
            map_key(KeyEvent::new(KeyCode::Char('A'), KeyModifiers::CONTROL)),
            AppAction::CursorLineStart
        );
        assert_eq!(
            map_key(KeyEvent::new(KeyCode::Char('e'), KeyModifiers::CONTROL)),
            AppAction::CursorLineEnd
        );
        assert_eq!(
            map_key(KeyEvent::new(KeyCode::Char('E'), KeyModifiers::CONTROL)),
            AppAction::CursorLineEnd
        );
    }
}
