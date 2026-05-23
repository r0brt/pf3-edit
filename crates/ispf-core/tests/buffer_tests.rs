use ispf_core::{EditBuffer, Record};

#[test]
fn loads_records_from_text_preserving_order() {
    let buffer = EditBuffer::from_text("ONE\nTWO\nTHREE\n");
    let texts: Vec<&str> = buffer.records().iter().map(Record::text).collect();
    assert_eq!(texts, vec!["ONE", "TWO", "THREE"]);
}

#[test]
fn insert_and_delete_update_dirty_state() {
    let mut buffer = EditBuffer::from_text("ALPHA\nBETA\n");
    assert!(!buffer.is_dirty());

    buffer.insert_after(0, "GAMMA");
    assert!(buffer.is_dirty());
    assert_eq!(buffer.records()[1].text(), "GAMMA");

    buffer.delete_at(1).unwrap();
    assert_eq!(buffer.records()[1].text(), "BETA");
}

#[test]
fn exclude_marks_line_without_removing_record() {
    let mut buffer = EditBuffer::from_text("A\nB\n");
    buffer.set_excluded(1, true).unwrap();
    assert!(buffer.records()[1].excluded);
}

#[test]
fn loads_and_saves_a_real_file() {
    let dir = std::env::temp_dir();
    let path = dir.join("ispf-buffer-roundtrip.txt");
    std::fs::write(&path, "LINE1\nLINE2\n").unwrap();

    let mut buffer = EditBuffer::from_path(&path).unwrap();
    buffer.insert_after(1, "LINE3");
    buffer.save().unwrap();

    let saved = std::fs::read_to_string(&path).unwrap();
    assert_eq!(saved, "LINE1\nLINE2\nLINE3\n");
}

#[test]
fn preserves_crlf_and_trailing_newline_on_round_trip_save() {
    let dir = std::env::temp_dir();
    let path = dir.join("ispf-buffer-crlf-roundtrip.txt");
    std::fs::write(&path, "LINE1\r\nLINE2\r\n").unwrap();

    let mut buffer = EditBuffer::from_path(&path).unwrap();
    buffer.save().unwrap();

    let saved = std::fs::read_to_string(&path).unwrap();
    assert_eq!(saved, "LINE1\r\nLINE2\r\n");
}

#[test]
fn preserves_missing_trailing_newline_on_round_trip_save() {
    let dir = std::env::temp_dir();
    let path = dir.join("ispf-buffer-no-trailing-newline.txt");
    std::fs::write(&path, "LINE1\nLINE2").unwrap();

    let mut buffer = EditBuffer::from_path(&path).unwrap();
    buffer.save().unwrap();

    let saved = std::fs::read_to_string(&path).unwrap();
    assert_eq!(saved, "LINE1\nLINE2");
}

#[test]
fn insert_after_empty_buffer_inserts_first_record() {
    let mut buffer = EditBuffer::default();
    buffer.insert_after(0, "FIRST");

    assert_eq!(buffer.records().len(), 1);
    assert_eq!(buffer.records()[0].id.0, 1);
    assert_eq!(buffer.records()[0].text(), "FIRST");
    assert!(buffer.is_dirty());
}

#[test]
fn insert_after_out_of_range_appends_record() {
    let mut buffer = EditBuffer::from_text("ONE\nTWO\n");
    buffer.insert_after(99, "THREE");

    let texts: Vec<&str> = buffer.records().iter().map(Record::text).collect();
    assert_eq!(texts, vec!["ONE", "TWO", "THREE"]);
}

#[test]
fn unchanged_excluded_value_does_not_mark_buffer_dirty() {
    let mut buffer = EditBuffer::from_text("A\nB\n");
    assert!(!buffer.is_dirty());

    buffer.set_excluded(1, false).unwrap();

    assert!(!buffer.is_dirty());
    assert!(!buffer.records()[1].excluded);
}
