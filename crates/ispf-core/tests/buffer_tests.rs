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
