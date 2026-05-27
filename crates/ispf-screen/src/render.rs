use ispf_command::ScrollMode;
use ispf_core::{ActiveArea, EditorSession};

const DEFAULT_DATA_COLUMNS: usize = 144;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScreenRow {
    pub record_index: usize,
    pub line_number: String,
    pub prefix: String,
    pub text: String,
    pub is_excluded_placeholder: bool,
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
    let visible_entries = collect_visible_entries(session);
    let show_bottom_banner = visible_entries.len() + usize::from(show_top_banner) < body_rows;
    let visible_rows = body_rows
        .saturating_sub(usize::from(show_top_banner))
        .saturating_sub(usize::from(show_bottom_banner));
    let rows = visible_entries
        .into_iter()
        .take(visible_rows)
        .map(|entry| {
            let is_cursor_row = session.view().cursor_row >= entry.start_index
                && session.view().cursor_row <= entry.end_index;
            let prefix_selected =
                is_cursor_row && session.view().active_area == ActiveArea::LineCommandArea;
            let text_selected = is_cursor_row && session.view().active_area == ActiveArea::DataArea;

            ScreenRow {
                record_index: entry.start_index,
                line_number: if session.profile().number_mode {
                    format!("{:06}", entry.start_index + 1)
                } else {
                    String::new()
                },
                prefix: String::new(),
                text: entry.text,
                is_excluded_placeholder: entry.is_excluded_placeholder,
                prefix_selected,
                text_selected,
                text_cursor_col: text_selected.then_some(if entry.is_excluded_placeholder {
                    0
                } else {
                    session
                        .view()
                        .cursor_col
                        .saturating_sub(session.view().left_col)
                }),
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
        scroll_value: match session.profile().scroll_mode {
            ScrollMode::Page => "PAGE".into(),
            ScrollMode::Half => "HALF".into(),
            ScrollMode::Csr => "CSR".into(),
        },
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

#[derive(Clone, Debug, Eq, PartialEq)]
struct VisibleEntry {
    start_index: usize,
    end_index: usize,
    text: String,
    is_excluded_placeholder: bool,
}

fn collect_visible_entries(session: &EditorSession) -> Vec<VisibleEntry> {
    let records = session.buffer().records();
    let mut entries = Vec::new();
    let mut index = normalize_visible_start(records, session.view().top_row);

    while index < records.len() {
        let record = &records[index];
        if record.excluded {
            let mut end = index;
            while end + 1 < records.len() && records[end + 1].excluded {
                end += 1;
            }
            entries.push(VisibleEntry {
                start_index: index,
                end_index: end,
                text: match end - index + 1 {
                    1 => "1 line excluded".into(),
                    count => format!("{count} lines excluded"),
                },
                is_excluded_placeholder: true,
            });
            index = end + 1;
        } else {
            entries.push(VisibleEntry {
                start_index: index,
                end_index: index,
                text: record.text.chars().skip(session.view().left_col).collect(),
                is_excluded_placeholder: false,
            });
            index += 1;
        }
    }

    entries
}

fn normalize_visible_start(records: &[ispf_core::Record], start: usize) -> usize {
    if start >= records.len() {
        return records.len();
    }

    if !records[start].excluded {
        return start;
    }

    let mut normalized = start;
    while normalized > 0 && records[normalized - 1].excluded {
        normalized -= 1;
    }
    normalized
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
    let edit_mode = if session.profile().insert_mode {
        "INS"
    } else {
        "OVR"
    };
    let area = match session.view().active_area {
        ActiveArea::DataArea => "DATA",
        ActiveArea::LineCommandArea => "LINE CMD",
        ActiveArea::CommandLine => "COMMAND",
    };

    format!(
        "{edit_mode} {area}  Ln {} Col {}",
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
