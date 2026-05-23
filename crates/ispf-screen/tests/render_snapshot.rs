use ispf_core::{EditBuffer, EditorSession};
use ispf_screen::render_screen;

#[test]
fn renders_command_line_prefix_area_and_data_rows() {
    let session = EditorSession::new(EditBuffer::from_text("ONE\nTWO\n").unwrap());
    let screen = render_screen(&session, 80, 24);

    assert_eq!(screen.command_prompt, "Command ===>");
    assert_eq!(screen.scroll_label, "Scroll ===>");
    assert_eq!(screen.rows[0].text.trim_end(), "ONE");
}
