use ispf_command::PrimaryCommand;
use ispf_core::{EditBuffer, EditorSession};
use ispf_screen::render_screen;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT_TEMP_ID: AtomicUsize = AtomicUsize::new(1);

fn unique_temp_path(name: &str) -> PathBuf {
    let id = NEXT_TEMP_ID.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!(
        "ispf-screen-{name}-{}-{id}.txt",
        std::process::id()
    ))
}

#[test]
fn renders_command_line_line_command_area_and_data_rows() {
    let session = EditorSession::new(EditBuffer::from_text("ONE\nTWO\n").unwrap());
    let screen = render_screen(&session, 80, 24);

    assert!(screen.menu_bar.contains("Utilities"));
    assert_eq!(screen.header_left, "EDIT");
    assert_eq!(screen.command_prompt, "Command ===>");
    assert_eq!(screen.scroll_label, "Scroll ===>");
    assert_eq!(screen.scroll_value, "PAGE");
    let top_banner = screen.data_banner.as_deref().expect("top banner missing");
    assert!(top_banner.contains("Top of Data"));
    assert_eq!(top_banner.len(), 144);
    let bottom_banner = screen
        .bottom_banner
        .as_deref()
        .expect("bottom banner missing");
    assert!(bottom_banner.contains("Bottom of Data"));
    assert_eq!(bottom_banner.len(), 144);
    assert_eq!(screen.rows[0].text.trim_end(), "ONE");
}

#[test]
fn renders_line_numbers_when_number_mode_is_enabled() {
    let mut session = EditorSession::new(EditBuffer::from_text("ONE\n").unwrap());
    session.execute_primary(PrimaryCommand::Number(true)).unwrap();

    let screen = render_screen(&session, 80, 24);

    assert_eq!(screen.rows[0].line_number, "000001");
    assert_eq!(screen.rows[0].prefix.trim(), "");
}

#[test]
fn highlights_active_data_row_text() {
    let session = EditorSession::new(EditBuffer::from_text("ONE\nTWO\n").unwrap());
    let screen = render_screen(&session, 80, 24);

    assert!(screen.rows[0].text_selected);
    assert!(!screen.rows[0].prefix_selected);
}

#[test]
fn highlights_active_line_command_cell_when_line_command_area_is_focused() {
    let mut session = EditorSession::new(EditBuffer::from_text("ONE\nTWO\n").unwrap());
    session.toggle_active_area();
    session.toggle_active_area();
    let screen = render_screen(&session, 80, 24);

    assert!(screen.rows[0].prefix_selected);
    assert!(!screen.rows[0].text_selected);
}

#[test]
fn title_includes_file_name_and_dirty_marker() {
    let path = unique_temp_path("title");
    std::fs::write(&path, "ONE\n").unwrap();
    let file_name = path.file_name().unwrap().to_string_lossy().to_string();

    let mut session = EditorSession::new(EditBuffer::from_path(&path).unwrap());
    assert_eq!(
        render_screen(&session, 80, 24).title,
        file_name
    );

    session.insert_char('X').unwrap();

    assert!(render_screen(&session, 80, 24).title.ends_with(" *"));
}

#[test]
fn status_summary_reflects_active_area_and_cursor_position() {
    let mut session = EditorSession::new(EditBuffer::from_text("ONE\nTWO\n").unwrap());
    session.move_cursor_down();
    session.move_cursor_right();
    session.toggle_active_area();
    session.toggle_active_area();

    let screen = render_screen(&session, 80, 24);

    assert_eq!(screen.status_summary, "LINE CMD  Ln 2 Col 2");
}

#[test]
fn columns_and_footer_match_the_ispf_style_shell() {
    let mut session = EditorSession::new(EditBuffer::from_text("ONE\n").unwrap());
    session.execute_primary(PrimaryCommand::Right(8)).unwrap();

    let screen = render_screen(&session, 80, 24);

    assert_eq!(screen.header_right, "Columns 00009 00152");
    assert!(screen.footer_keys[0].contains("F3=Exit"));
    assert!(screen.footer_keys[1].contains("F12=Cancel"));
}

#[test]
fn cols_command_displays_a_columns_indicator_line() {
    let mut session = EditorSession::new(EditBuffer::from_text("ONE\n").unwrap());
    session.execute_primary(PrimaryCommand::Cols).unwrap();

    let screen = render_screen(&session, 80, 24);

    let cols_line = screen.cols_line.as_deref().expect("cols line missing");
    assert!(cols_line.starts_with("=COLS>"));
    assert!(cols_line.contains('+'));
    assert!(cols_line.len() > 20);
}

#[test]
fn bounds_command_displays_a_bounds_indicator_line() {
    let mut session = EditorSession::new(EditBuffer::from_text("ONE\n").unwrap());
    session
        .execute_primary(PrimaryCommand::Bounds(Some((7, 70))))
        .unwrap();

    let screen = render_screen(&session, 80, 24);

    let bounds_line = screen
        .bounds_line
        .as_deref()
        .expect("bounds line missing");
    assert!(bounds_line.starts_with("=BNDS>"));
    assert!(bounds_line.contains('|'));
}

#[test]
fn top_of_data_banner_disappears_when_scrolled_down() {
    let mut session = EditorSession::new(EditBuffer::from_text("ONE\nTWO\nTHREE\n").unwrap());
    session.execute_primary(PrimaryCommand::Down(1)).unwrap();

    let screen = render_screen(&session, 80, 24);

    assert_eq!(screen.data_banner, None);
}

#[test]
fn bottom_of_data_banner_disappears_when_more_rows_remain_below() {
    let session = EditorSession::new(
        EditBuffer::from_text("ONE\nTWO\nTHREE\nFOUR\nFIVE\nSIX\nSEVEN\nEIGHT\n").unwrap(),
    );

    let screen = render_screen(&session, 80, 10);

    assert_eq!(screen.bottom_banner, None);
}
