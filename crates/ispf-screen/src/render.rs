use ispf_core::EditorSession;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScreenRow {
    pub prefix: String,
    pub text: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScreenModel {
    pub title: String,
    pub command_prompt: String,
    pub scroll_label: String,
    pub rows: Vec<ScreenRow>,
    pub message: String,
}

pub fn render_screen(session: &EditorSession, _width: u16, height: u16) -> ScreenModel {
    let visible_rows = height.saturating_sub(4) as usize;
    let rows = session
        .buffer()
        .records()
        .iter()
        .skip(session.view().top_row)
        .filter(|record| !record.excluded)
        .take(visible_rows)
        .map(|record| ScreenRow {
            prefix: String::new(),
            text: record.text.clone(),
        })
        .collect();

    ScreenModel {
        title: "ISPF Editor".into(),
        command_prompt: "Command ===>".into(),
        scroll_label: "Scroll ===>".into(),
        rows,
        message: String::new(),
    }
}
