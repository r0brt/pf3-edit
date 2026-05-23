use ispf_core::{EditBuffer, Record};
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT_TEMP_ID: AtomicUsize = AtomicUsize::new(1);

fn unique_temp_path(name: &str) -> PathBuf {
    let id = NEXT_TEMP_ID.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!(
        "ispf-{name}-{}-{id}.txt",
        std::process::id()
    ))
}

#[test]
fn loads_records_from_text_preserving_order() {
    let buffer = EditBuffer::from_text("ONE\nTWO\nTHREE\n").unwrap();
    let texts: Vec<&str> = buffer.records().iter().map(Record::text).collect();
    assert_eq!(texts, vec!["ONE", "TWO", "THREE"]);
}

#[test]
fn insert_and_delete_update_dirty_state() {
    let mut buffer = EditBuffer::from_text("ALPHA\nBETA\n").unwrap();
    assert!(!buffer.is_dirty());

    buffer.insert_after(0, "GAMMA");
    assert!(buffer.is_dirty());
    assert_eq!(buffer.records()[1].text(), "GAMMA");

    buffer.delete_at(1).unwrap();
    assert_eq!(buffer.records()[1].text(), "BETA");
}

#[test]
fn exclude_marks_line_without_removing_record() {
    let mut buffer = EditBuffer::from_text("A\nB\n").unwrap();
    buffer.set_excluded(1, true).unwrap();
    assert!(buffer.records()[1].excluded);
}

#[test]
fn loads_and_saves_a_real_file() {
    let path = unique_temp_path("buffer-roundtrip");
    std::fs::write(&path, "LINE1\nLINE2\n").unwrap();

    let mut buffer = EditBuffer::from_path(&path).unwrap();
    buffer.insert_after(1, "LINE3");
    buffer.save().unwrap();

    let saved = std::fs::read_to_string(&path).unwrap();
    assert_eq!(saved, "LINE1\nLINE2\nLINE3\n");
}

#[test]
fn preserves_crlf_and_trailing_newline_on_round_trip_save() {
    let path = unique_temp_path("buffer-crlf-roundtrip");
    std::fs::write(&path, "LINE1\r\nLINE2\r\n").unwrap();

    let mut buffer = EditBuffer::from_path(&path).unwrap();
    buffer.save().unwrap();

    let saved = std::fs::read_to_string(&path).unwrap();
    assert_eq!(saved, "LINE1\r\nLINE2\r\n");
}

#[test]
fn preserves_missing_trailing_newline_on_round_trip_save() {
    let path = unique_temp_path("buffer-no-trailing-newline");
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
    let mut buffer = EditBuffer::from_text("ONE\nTWO\n").unwrap();
    buffer.insert_after(99, "THREE");

    let texts: Vec<&str> = buffer.records().iter().map(Record::text).collect();
    assert_eq!(texts, vec!["ONE", "TWO", "THREE"]);
}

#[test]
fn unchanged_excluded_value_does_not_mark_buffer_dirty() {
    let mut buffer = EditBuffer::from_text("A\nB\n").unwrap();
    assert!(!buffer.is_dirty());

    buffer.set_excluded(1, false).unwrap();

    assert!(!buffer.is_dirty());
    assert!(!buffer.records()[1].excluded);
}

#[test]
fn rejects_mixed_newline_files() {
    let path = unique_temp_path("buffer-mixed-newlines");
    std::fs::write(&path, "LINE1\r\nLINE2\nLINE3\r\n").unwrap();

    let error = EditBuffer::from_path(&path).unwrap_err();

    assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);
    assert!(
        error.to_string().contains("mixed newline"),
        "unexpected error: {error}"
    );
}

#[test]
fn rejects_mixed_newlines_from_text() {
    let error = EditBuffer::try_from_text("LINE1\r\nLINE2\nLINE3\r\n").unwrap_err();

    assert!(
        error.to_string().contains("mixed newline"),
        "unexpected error: {error}"
    );
}
