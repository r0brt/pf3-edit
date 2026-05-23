use ispf_core::{
    ActiveArea, CapsMode, EditBuffer, EditProfile, EditorSession, UndoEntry, UndoStack,
};

#[test]
fn new_session_starts_in_data_area() {
    let buffer = EditBuffer::from_text("A\n").unwrap();
    let session = EditorSession::new(buffer);
    assert_eq!(session.view().cursor_row, 0);
    assert_eq!(session.view().cursor_col, 0);
    assert_eq!(session.view().top_row, 0);
    assert_eq!(session.view().left_col, 0);
    assert_eq!(session.view().active_area, ActiveArea::DataArea);
    assert_eq!(session.buffer().to_text(), "A\n");
    assert_eq!(session.message(), None);
    assert!(!session.can_undo());
}

#[test]
fn profile_defaults_match_v0_1_design() {
    let profile = EditProfile::default();
    assert_eq!(profile.caps_mode, CapsMode::Off);
    assert!(!profile.number_mode);
    assert_eq!(profile.bounds, None);
    assert_eq!(profile.tabs, vec![4, 8, 12, 16]);
}

#[test]
fn undo_stack_pops_last_entry_first() {
    let mut stack = UndoStack::default();
    stack.push(UndoEntry::InsertedLine { index: 1 });
    stack.push(UndoEntry::DeletedLine {
        index: 2,
        text: "B".to_string(),
    });

    assert_eq!(
        stack.pop(),
        Some(UndoEntry::DeletedLine {
            index: 2,
            text: "B".to_string(),
        })
    );
    assert_eq!(stack.pop(), Some(UndoEntry::InsertedLine { index: 1 }));
    assert_eq!(stack.pop(), None);
}
