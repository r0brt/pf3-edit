use ispf_core::{ActiveArea, EditorSession};

const DEFAULT_DATA_COLUMNS: usize = 144;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScreenRow {
    pub record_index: usize,
    pub line_number: String,
    pub prefix: String,
    pub text: String,
    pub prefix_selected: bool,
    pub text_selected: bool,
    pub text_cursor_col: Option<usize>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScreenModel {
    pub menu_bar: String,
    pub title: String,
    pub header_left: String,
    pub header_right: String,
    pub command_prompt: String,
    pub command_value: String,
    pub command_selected: bool,
    pub scroll_label: String,
    pub scroll_value: String,
    pub data_banner: Option<String>,
    pub cols_line: Option<String>,
    pub bounds_line: Option<String>,
    pub bottom_banner: Option<String>,
    pub rows: Vec<ScreenRow>,
    pub message: String,
    pub status_summary: String,
    pub footer_keys: [String; 2],
}

pub fn render_screen(session: &EditorSession, _width: u16, height: u16) -> ScreenModel {
    let show_top_banner = session.view().top_row == 0;
    let body_rows = height.saturating_sub(6) as usize;
    let visible_records: Vec<(usize, _)> = session
        .buffer()
        .records()
        .iter()
        .enumerate()
        .skip(session.view().top_row)
        .filter(|(_, record)| !record.excluded)
        .collect();
    let show_bottom_banner = visible_records.len() + usize::from(show_top_banner) < body_rows;
    let visible_rows = body_rows
        .saturating_sub(usize::from(show_top_banner))
        .saturating_sub(usize::from(show_bottom_banner));
    let rows = visible_records
        .into_iter()
        .take(visible_rows)
        .map(|(index, record)| {
            let is_cursor_row = index == session.view().cursor_row;
            let prefix_selected =
                is_cursor_row && session.view().active_area == ActiveArea::LineCommandArea;
            let text_selected = is_cursor_row && session.view().active_area == ActiveArea::DataArea;

            ScreenRow {
                record_index: index,
                line_number: if session.profile().number_mode {
                    format!("{:06}", index + 1)
                } else {
                    String::new()
                },
                prefix: String::new(),
                text: record
                    .text
                    .chars()
                    .skip(session.view().left_col)
                    .collect(),
                prefix_selected,
                text_selected,
                text_cursor_col: text_selected.then_some(
                    session
                        .view()
                        .cursor_col
                        .saturating_sub(session.view().left_col),
                ),
            }
        })
        .collect();

    ScreenModel {
        menu_bar: "File  Edit  Edit_Settings  Menu  Utilities  Compilers  Test  Help".into(),
        title: title_for_session(session),
        header_left: "EDIT".into(),
        header_right: columns_summary(session),
        command_prompt: "Command ===>".into(),
        command_value: String::new(),
        command_selected: false,
        scroll_label: "Scroll ===>".into(),
        scroll_value: "PAGE".into(),
        data_banner: show_top_banner.then(|| top_of_data_banner(DEFAULT_DATA_COLUMNS)),
        cols_line: session
            .profile()
            .cols_mode
            .then(|| cols_line_for(session.view().left_col)),
        bounds_line: session
            .profile()
            .bounds
            .map(|bounds| bounds_line_for(session.view().left_col, bounds)),
        bottom_banner: show_bottom_banner.then(|| bottom_of_data_banner(DEFAULT_DATA_COLUMNS)),
        rows,
        message: session
            .message()
            .map(|message| message.text.clone())
            .unwrap_or_default(),
        status_summary: status_summary(session),
        footer_keys: [
            "F1=Help  F2=Split  F3=Exit  F5=Rfind  F6=Rchange  F7=Up".into(),
            "F8=Down  F9=Swap  F10=Left  F11=Right  F12=Cancel".into(),
        ],
    }
}

fn title_for_session(session: &EditorSession) -> String {
    let mut title = String::new();
    if let Some(path) = session.buffer().file_path()
        && let Some(name) = path.file_name().and_then(|value| value.to_str())
    {
        title.push_str(name);
    } else {
        title.push_str("UNTITLED");
    }
    if session.buffer().is_dirty() {
        title.push_str(" *");
    }
    title
}

fn columns_summary(session: &EditorSession) -> String {
    let start = session.view().left_col + 1;
    let end = session.view().left_col + DEFAULT_DATA_COLUMNS;
    format!("Columns {:05} {:05}", start, end)
}

fn status_summary(session: &EditorSession) -> String {
    let area = match session.view().active_area {
        ActiveArea::DataArea => "DATA",
        ActiveArea::LineCommandArea => "LINE CMD",
        ActiveArea::CommandLine => "COMMAND",
    };

    format!(
        "{area}  Ln {} Col {}",
        session.view().cursor_row + 1,
        session.view().cursor_col + 1
    )
}

fn top_of_data_banner(width: usize) -> String {
    data_banner("Top of Data", width)
}

fn bottom_of_data_banner(width: usize) -> String {
    data_banner("Bottom of Data", width)
}

fn cols_line_for(left_col: usize) -> String {
    let mut line = String::from("=COLS> ");
    let start = left_col + 1;
    let end = left_col + DEFAULT_DATA_COLUMNS;
    for col in start..=end {
        if col == start {
            line.push('-');
        } else if col % 10 == 0 {
            line.push('+');
        } else if col % 10 == 1 {
            let digit = char::from(b'0' + (((col / 10) % 10) as u8));
            line.push(digit);
        } else {
            line.push('-');
        }
    }
    line
}

fn bounds_line_for(left_col: usize, bounds: (usize, usize)) -> String {
    let mut line = String::from("=BNDS> ");
    let start = left_col + 1;
    let end = left_col + DEFAULT_DATA_COLUMNS;
    for col in start..=end {
        if col == bounds.0 || col == bounds.1 {
            line.push('|');
        } else if col > bounds.0 && col < bounds.1 {
            line.push('-');
        } else {
            line.push('.');
        }
    }
    line
}

fn data_banner(label: &str, width: usize) -> String {
    let minimum = label.len() + 2;
    let width = width.max(minimum);
    let interior = width.saturating_sub(label.len() + 2);
    let left = interior / 2;
    let right = interior.saturating_sub(left);

    format!("{} {} {}", "*".repeat(left), label, "*".repeat(right))
}
