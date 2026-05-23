use anyhow::Result;
use ispf_core::{EditBuffer, EditorSession};
use ispf_screen::render_screen;

pub fn run() -> Result<()> {
    let buffer = EditBuffer::from_text("ISPF EDITOR\n").unwrap();
    let session = EditorSession::new(buffer);
    let _screen = render_screen(&session, 80, 24);

    Ok(())
}
