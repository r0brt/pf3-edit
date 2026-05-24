use ispf_core::{
    ActiveArea, CapsMode, EditBuffer, EditProfile, EditorSession, UndoEntry, UndoStack,
};
use ispf_command::{PrefixCommand, PrimaryCommand};

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
        record: ispf_core::Record {
            id: ispf_core::RecordId(2),
            text: "B".to_string(),
            excluded: false,
        },
    });

    assert_eq!(
        stack.pop(),
        Some(UndoEntry::DeletedLine {
            index: 2,
            record: ispf_core::Record {
                id: ispf_core::RecordId(2),
                text: "B".to_string(),
                excluded: false,
            },
        })
    );
    assert_eq!(stack.pop(), Some(UndoEntry::InsertedLine { index: 1 }));
    assert_eq!(stack.pop(), None);
}

#[test]
fn execute_primary_scrolls_and_toggles_profile() {
    let buffer = EditBuffer::from_text("A\nB\nC\n").unwrap();
    let mut session = EditorSession::new(buffer);

    session.execute_primary(PrimaryCommand::Down(2)).unwrap();
    assert_eq!(session.view().top_row, 2);

    session.execute_primary(PrimaryCommand::Number(true)).unwrap();
    assert!(session.profile().number_mode);

    session.execute_primary(PrimaryCommand::Cols).unwrap();
    assert!(session.profile().cols_mode);

    session
        .execute_primary(PrimaryCommand::Bounds(Some((7, 70))))
        .unwrap();
    assert_eq!(session.profile().bounds, Some((7, 70)));
}

#[test]
fn home_and_end_respect_bounds() {
    let buffer = EditBuffer::from_text("ABCDEFG\n").unwrap();
    let mut session = EditorSession::new(buffer);

    session
        .execute_primary(PrimaryCommand::Bounds(Some((3, 5))))
        .unwrap();

    session.move_cursor_to_line_start();
    assert_eq!(session.view().cursor_col, 2);

    session.move_cursor_to_line_end();
    assert_eq!(session.view().cursor_col, 4);
}

#[test]
fn insert_char_respects_the_right_bound() {
    let buffer = EditBuffer::from_text("ABCDE\n").unwrap();
    let mut session = EditorSession::new(buffer);

    session
        .execute_primary(PrimaryCommand::Bounds(Some((1, 3))))
        .unwrap();

    session.move_cursor_to_line_end();
    session.insert_char('Z').unwrap();

    assert_eq!(session.buffer().records()[0].text(), "ABZDE");
    assert_eq!(session.view().cursor_col, 3);

    session.insert_char('Q').unwrap();
    assert_eq!(session.buffer().records()[0].text(), "ABZDE");
    assert_eq!(session.view().cursor_col, 3);
}

#[test]
fn horizontal_cursor_movement_clamps_to_line_length() {
    let buffer = EditBuffer::from_text("ABCD\nX\n").unwrap();
    let mut session = EditorSession::new(buffer);

    session.move_cursor_right();
    session.move_cursor_right();
    session.move_cursor_right();
    session.move_cursor_right();
    session.move_cursor_right();
    assert_eq!(session.view().cursor_col, 4);

    session.move_cursor_down();
    assert_eq!(session.view().cursor_col, 1);

    session.move_cursor_left();
    session.move_cursor_left();
    assert_eq!(session.view().cursor_col, 0);
}

#[test]
fn line_start_and_end_move_the_cursor_within_the_current_record() {
    let buffer = EditBuffer::from_text("ABCD\nX\n").unwrap();
    let mut session = EditorSession::new(buffer);

    session.move_cursor_right();
    session.move_cursor_right();
    session.move_cursor_to_line_end();
    assert_eq!(session.view().cursor_col, 4);

    session.move_cursor_to_line_start();
    assert_eq!(session.view().cursor_col, 0);
}

#[test]
fn insert_char_overwrites_existing_character_and_advances_cursor() {
    let buffer = EditBuffer::from_text("AB\n").unwrap();
    let mut session = EditorSession::new(buffer);

    session.move_cursor_right();
    session.insert_char('X').unwrap();

    assert_eq!(session.buffer().records()[0].text(), "AX");
    assert_eq!(session.view().cursor_col, 2);
    assert!(session.buffer().is_dirty());
}

#[test]
fn backspace_removes_the_character_before_the_cursor() {
    let buffer = EditBuffer::from_text("AB\n").unwrap();
    let mut session = EditorSession::new(buffer);

    session.move_cursor_right();
    session.move_cursor_right();
    session.backspace_char().unwrap();

    assert_eq!(session.buffer().records()[0].text(), "A");
    assert_eq!(session.view().cursor_col, 1);

    session.execute_primary(PrimaryCommand::Undo).unwrap();
    assert_eq!(session.buffer().records()[0].text(), "AB");
    assert_eq!(session.view().cursor_col, 1);
}

#[test]
fn backspace_at_column_zero_joins_the_previous_line() {
    let buffer = EditBuffer::from_text("AB\nCD\n").unwrap();
    let mut session = EditorSession::new(buffer);

    session.move_cursor_down();
    session.backspace_char().unwrap();

    assert_eq!(session.buffer().to_text(), "ABCD\n");
    assert_eq!(session.view().cursor_row, 0);
    assert_eq!(session.view().cursor_col, 2);
}

#[test]
fn delete_char_removes_the_character_under_the_cursor() {
    let buffer = EditBuffer::from_text("AB\n").unwrap();
    let mut session = EditorSession::new(buffer);

    session.delete_char().unwrap();

    assert_eq!(session.buffer().to_text(), "B\n");
    assert_eq!(session.view().cursor_col, 0);
}

#[test]
fn delete_at_end_of_line_joins_the_next_line() {
    let buffer = EditBuffer::from_text("AB\nCD\n").unwrap();
    let mut session = EditorSession::new(buffer);

    session.move_cursor_right();
    session.move_cursor_right();
    session.delete_char().unwrap();

    assert_eq!(session.buffer().to_text(), "ABCD\n");
    assert_eq!(session.view().cursor_row, 0);
    assert_eq!(session.view().cursor_col, 2);
}

#[test]
fn split_line_at_cursor_creates_a_new_line_below() {
    let buffer = EditBuffer::from_text("ABCD\n").unwrap();
    let mut session = EditorSession::new(buffer);

    session.move_cursor_right();
    session.move_cursor_right();
    session.split_line_at_cursor().unwrap();

    assert_eq!(session.buffer().to_text(), "AB\nCD\n");
    assert_eq!(session.view().cursor_row, 1);
    assert_eq!(session.view().cursor_col, 0);
}

#[test]
fn undo_restores_a_split_line() {
    let buffer = EditBuffer::from_text("ABCD\n").unwrap();
    let mut session = EditorSession::new(buffer);

    session.move_cursor_right();
    session.move_cursor_right();
    session.split_line_at_cursor().unwrap();
    session.execute_primary(PrimaryCommand::Undo).unwrap();

    assert_eq!(session.buffer().to_text(), "ABCD\n");
}

#[test]
fn undo_restores_a_joined_line() {
    let buffer = EditBuffer::from_text("AB\nCD\n").unwrap();
    let mut session = EditorSession::new(buffer);

    session.move_cursor_down();
    session.backspace_char().unwrap();
    session.execute_primary(PrimaryCommand::Undo).unwrap();

    assert_eq!(session.buffer().to_text(), "AB\nCD\n");
}

#[test]
fn execute_prefix_delete_removes_the_target_line() {
    let buffer = EditBuffer::from_text("A\nB\nC\n").unwrap();
    let mut session = EditorSession::new(buffer);

    session.execute_prefix(1, PrefixCommand::Delete(1)).unwrap();
    assert_eq!(session.buffer().records()[1].text(), "C");
}

#[test]
fn execute_prefix_delete_count_removes_multiple_lines() {
    let buffer = EditBuffer::from_text("A\nB\nC\nD\n").unwrap();
    let mut session = EditorSession::new(buffer);

    session.execute_prefix(1, PrefixCommand::Delete(2)).unwrap();

    assert_eq!(session.buffer().to_text(), "A\nD\n");
}

#[test]
fn find_positions_cursor_on_matching_record() {
    let buffer = EditBuffer::from_text("ZERO\nALPHA\nOMEGA\n").unwrap();
    let mut session = EditorSession::new(buffer);

    session
        .execute_primary(PrimaryCommand::Find {
            pattern: "ALPHA".into(),
        })
        .unwrap();

    assert_eq!(session.view().cursor_row, 1);
}

#[test]
fn locate_moves_cursor_and_view_to_the_requested_line_number() {
    let buffer = EditBuffer::from_text("A\nB\nC\nD\n").unwrap();
    let mut session = EditorSession::new(buffer);

    session
        .execute_primary(PrimaryCommand::Locate { target: 3 })
        .unwrap();

    assert_eq!(session.view().cursor_row, 2);
    assert_eq!(session.view().top_row, 2);
}

#[test]
fn change_replaces_text_in_place() {
    let buffer = EditBuffer::from_text("OLD VALUE\n").unwrap();
    let mut session = EditorSession::new(buffer);

    session
        .execute_primary(PrimaryCommand::Change {
            from: "OLD".into(),
            to: "NEW".into(),
        })
        .unwrap();

    assert_eq!(session.buffer().records()[0].text(), "NEW VALUE");
}

#[test]
fn find_respects_bounds() {
    let buffer = EditBuffer::from_text("ZERO ALPHA\n").unwrap();
    let mut session = EditorSession::new(buffer);
    session
        .execute_primary(PrimaryCommand::Bounds(Some((1, 4))))
        .unwrap();

    session
        .execute_primary(PrimaryCommand::Find {
            pattern: "ALPHA".into(),
        })
        .unwrap();

    assert_eq!(session.message().unwrap().text, "Pattern not found");
}

#[test]
fn change_respects_bounds() {
    let buffer = EditBuffer::from_text("ZERO ALPHA\n").unwrap();
    let mut session = EditorSession::new(buffer);
    session
        .execute_primary(PrimaryCommand::Bounds(Some((1, 4))))
        .unwrap();

    session
        .execute_primary(PrimaryCommand::Change {
            from: "ALPHA".into(),
            to: "BETA".into(),
        })
        .unwrap();

    assert_eq!(session.buffer().to_text(), "ZERO ALPHA\n");
    assert_eq!(session.message().unwrap().text, "Pattern not found");
}

#[test]
fn lowercase_line_command_respects_bounds() {
    let buffer = EditBuffer::from_text("ABCD EFGH\n").unwrap();
    let mut session = EditorSession::new(buffer);
    session
        .execute_primary(PrimaryCommand::Bounds(Some((6, 9))))
        .unwrap();

    session.execute_prefix(0, PrefixCommand::Lowercase(1)).unwrap();

    assert_eq!(session.buffer().to_text(), "ABCD efgh\n");
}

#[test]
fn uppercase_count_line_command_updates_multiple_lines() {
    let buffer = EditBuffer::from_text("ab\ncd\nef\n").unwrap();
    let mut session = EditorSession::new(buffer);

    session.execute_prefix(0, PrefixCommand::Uppercase(2)).unwrap();

    assert_eq!(session.buffer().to_text(), "AB\nCD\nef\n");
}

#[test]
fn lowercase_block_updates_the_marked_range() {
    let buffer = EditBuffer::from_text("AB\nCD\nEF\n").unwrap();
    let mut session = EditorSession::new(buffer);

    session.execute_prefix(0, PrefixCommand::LowercaseBlock).unwrap();
    session.execute_prefix(1, PrefixCommand::LowercaseBlock).unwrap();

    assert_eq!(session.buffer().to_text(), "ab\ncd\nEF\n");
}

#[test]
fn undo_restores_deleted_line() {
    let buffer = EditBuffer::from_text("A\nB\n").unwrap();
    let mut session = EditorSession::new(buffer);

    session.execute_prefix(1, PrefixCommand::Delete(1)).unwrap();
    session.execute_primary(PrimaryCommand::Undo).unwrap();

    assert_eq!(session.buffer().records()[1].text(), "B");
}

#[test]
fn exclude_block_hides_the_full_marked_range() {
    let buffer = EditBuffer::from_text("A\nB\nC\nD\nE\n").unwrap();
    let mut session = EditorSession::new(buffer);

    session.execute_prefix(1, PrefixCommand::ExcludeBlock).unwrap();
    session.execute_prefix(3, PrefixCommand::ExcludeBlock).unwrap();

    assert!(!session.buffer().records()[0].excluded);
    assert!(session.buffer().records()[1].excluded);
    assert!(session.buffer().records()[2].excluded);
    assert!(session.buffer().records()[3].excluded);
    assert!(!session.buffer().records()[4].excluded);
}

#[test]
fn exclude_moves_cursor_to_the_next_visible_row_and_sets_a_message() {
    let buffer = EditBuffer::from_text("A\nB\nC\n").unwrap();
    let mut session = EditorSession::new(buffer);

    session.execute_prefix(0, PrefixCommand::Exclude).unwrap();

    assert!(session.buffer().records()[0].excluded);
    assert_eq!(session.view().cursor_row, 1);
    assert_eq!(session.message().unwrap().text, "Line excluded");
}

#[test]
fn reset_restores_hidden_rows_and_sets_a_message() {
    let buffer = EditBuffer::from_text("A\nB\nC\n").unwrap();
    let mut session = EditorSession::new(buffer);

    session.execute_prefix(0, PrefixCommand::Exclude).unwrap();
    session.execute_prefix(1, PrefixCommand::Exclude).unwrap();
    session.execute_primary(PrimaryCommand::Reset).unwrap();

    assert!(!session.buffer().records()[0].excluded);
    assert!(!session.buffer().records()[1].excluded);
    assert_eq!(session.message().unwrap().text, "RESET completed");
}

#[test]
fn cancel_restores_the_original_buffer_contents() {
    let buffer = EditBuffer::from_text("A\nB\nC\n").unwrap();
    let mut session = EditorSession::new(buffer);

    session.execute_prefix(1, PrefixCommand::Delete(1)).unwrap();
    session.execute_primary(PrimaryCommand::Cancel).unwrap();

    assert_eq!(session.buffer().to_text(), "A\nB\nC\n");
    assert!(session.should_exit());
}

#[test]
fn undo_restores_deleted_line_with_same_record_id() {
    let buffer = EditBuffer::from_text("A\nB\n").unwrap();
    let original_id = buffer.records()[1].id;
    let mut session = EditorSession::new(buffer);

    session.execute_prefix(1, PrefixCommand::Delete(1)).unwrap();
    session.execute_primary(PrimaryCommand::Undo).unwrap();

    assert_eq!(session.buffer().records()[1].id, original_id);
}

#[test]
fn execute_prefix_insert_count_inserts_multiple_blank_lines() {
    let buffer = EditBuffer::from_text("A\nB\n").unwrap();
    let mut session = EditorSession::new(buffer);

    session.execute_prefix(0, PrefixCommand::Insert(3)).unwrap();

    assert_eq!(session.buffer().records().len(), 5);
    assert_eq!(session.buffer().records()[0].text(), "");
    assert_eq!(session.buffer().records()[1].text(), "");
    assert_eq!(session.buffer().records()[2].text(), "");
    assert_eq!(session.buffer().records()[3].text(), "A");
    assert_eq!(session.buffer().records()[4].text(), "B");
}

#[test]
fn insert_blank_line_after_inserts_below_the_current_row() {
    let buffer = EditBuffer::from_text("A\nB\nC\n").unwrap();
    let mut session = EditorSession::new(buffer);

    session.insert_blank_line_after(0);

    assert_eq!(session.buffer().to_text(), "A\n\nB\nC\n");
    assert_eq!(session.view().cursor_row, 1);
}

#[test]
fn execute_prefix_repeat_count_repeats_line_multiple_times() {
    let buffer = EditBuffer::from_text("A\nB\n").unwrap();
    let mut session = EditorSession::new(buffer);

    session.execute_prefix(0, PrefixCommand::Repeat(2)).unwrap();

    assert_eq!(session.buffer().to_text(), "A\nA\nA\nB\n");
}

#[test]
fn delete_block_removes_the_marked_range() {
    let buffer = EditBuffer::from_text("A\nB\nC\nD\nE\n").unwrap();
    let mut session = EditorSession::new(buffer);

    session.execute_prefix(1, PrefixCommand::DeleteBlock).unwrap();
    session.execute_prefix(3, PrefixCommand::DeleteBlock).unwrap();

    assert_eq!(session.buffer().to_text(), "A\nE\n");
    assert_eq!(session.message().unwrap().text, "3 lines deleted");
}

#[test]
fn repeat_block_repeats_the_marked_range_below_the_block() {
    let buffer = EditBuffer::from_text("A\nB\nC\nD\n").unwrap();
    let mut session = EditorSession::new(buffer);

    session.execute_prefix(1, PrefixCommand::RepeatBlock).unwrap();
    session.execute_prefix(2, PrefixCommand::RepeatBlock).unwrap();

    assert_eq!(session.buffer().to_text(), "A\nB\nC\nB\nC\nD\n");
    assert_eq!(session.message().unwrap().text, "2 lines repeated");
}

#[test]
fn after_sets_a_destination_marker_without_inserting_a_blank_line() {
    let buffer = EditBuffer::from_text("A\nB\nC\n").unwrap();
    let mut session = EditorSession::new(buffer);

    session.execute_prefix(1, PrefixCommand::After).unwrap();

    assert_eq!(session.buffer().to_text(), "A\nB\nC\n");
    assert_eq!(session.pending_destination(), Some(ispf_core::Destination::After(1)));
    assert_eq!(session.message().unwrap().text, "After destination set");
}

#[test]
fn copy_block_copies_the_marked_range_after_the_destination() {
    let buffer = EditBuffer::from_text("A\nB\nC\nD\nE\n").unwrap();
    let mut session = EditorSession::new(buffer);

    session.execute_prefix(1, PrefixCommand::CopyBlock).unwrap();
    session.execute_prefix(2, PrefixCommand::CopyBlock).unwrap();
    session.execute_prefix(4, PrefixCommand::After).unwrap();

    assert_eq!(session.buffer().to_text(), "A\nB\nC\nD\nE\nB\nC\n");
    assert_eq!(session.message().unwrap().text, "2 lines copied");
    assert_eq!(session.pending_copy_range(), None);
    assert_eq!(session.pending_destination(), None);
}

#[test]
fn copy_copies_a_single_line_after_the_destination() {
    let buffer = EditBuffer::from_text("A\nB\nC\nD\n").unwrap();
    let mut session = EditorSession::new(buffer);

    session.execute_prefix(1, PrefixCommand::Copy(1)).unwrap();
    session.execute_prefix(3, PrefixCommand::After).unwrap();

    assert_eq!(session.buffer().to_text(), "A\nB\nC\nD\nB\n");
    assert_eq!(session.message().unwrap().text, "1 line copied");
}

#[test]
fn copy_count_copies_multiple_lines_after_the_destination() {
    let buffer = EditBuffer::from_text("A\nB\nC\nD\nE\n").unwrap();
    let mut session = EditorSession::new(buffer);

    session.execute_prefix(1, PrefixCommand::Copy(2)).unwrap();
    session.execute_prefix(4, PrefixCommand::After).unwrap();

    assert_eq!(session.buffer().to_text(), "A\nB\nC\nD\nE\nB\nC\n");
    assert_eq!(session.message().unwrap().text, "2 lines copied");
}

#[test]
fn move_block_moves_the_marked_range_before_the_destination() {
    let buffer = EditBuffer::from_text("A\nB\nC\nD\nE\n").unwrap();
    let mut session = EditorSession::new(buffer);

    session.execute_prefix(1, PrefixCommand::MoveBlock).unwrap();
    session.execute_prefix(2, PrefixCommand::MoveBlock).unwrap();
    session.execute_prefix(4, PrefixCommand::Before).unwrap();

    assert_eq!(session.buffer().to_text(), "A\nD\nB\nC\nE\n");
    assert_eq!(session.message().unwrap().text, "2 lines moved");
    assert_eq!(session.pending_move_range(), None);
    assert_eq!(session.pending_destination(), None);
}

#[test]
fn move_moves_a_single_line_before_the_destination() {
    let buffer = EditBuffer::from_text("A\nB\nC\nD\n").unwrap();
    let mut session = EditorSession::new(buffer);

    session.execute_prefix(1, PrefixCommand::Move(1)).unwrap();
    session.execute_prefix(3, PrefixCommand::Before).unwrap();

    assert_eq!(session.buffer().to_text(), "A\nC\nB\nD\n");
    assert_eq!(session.message().unwrap().text, "1 line moved");
}

#[test]
fn move_count_moves_multiple_lines_before_the_destination() {
    let buffer = EditBuffer::from_text("A\nB\nC\nD\nE\n").unwrap();
    let mut session = EditorSession::new(buffer);

    session.execute_prefix(1, PrefixCommand::Move(2)).unwrap();
    session.execute_prefix(4, PrefixCommand::Before).unwrap();

    assert_eq!(session.buffer().to_text(), "A\nD\nB\nC\nE\n");
    assert_eq!(session.message().unwrap().text, "2 lines moved");
}
