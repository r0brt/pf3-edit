use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ispf_core::{EditBuffer, EditorSession};
use ispf_screen::render_screen;

pub fn run() -> Result<()> {
    let buffer = if let Some(path) = std::env::args().nth(1) {
        EditBuffer::from_path(std::path::Path::new(&path))?
    } else {
        EditBuffer::from_text("ISPF EDITOR\n").unwrap()
    };
    let session = EditorSession::new(buffer);
    let _screen = render_screen(&session, 80, 24);
    let _ = crate::input::map_key(KeyEvent::from(KeyCode::Enter));

    Ok(())
}
