use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ispf_command::{parse_prefix, parse_primary, PrimaryCommand};
use ispf_core::{ActiveArea, EditBuffer, EditorSession};
use ispf_screen::{render_screen, ScreenModel};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Terminal,
};
use std::collections::BTreeMap;
use std::io::{stdout, IsTerminal};

use crate::input::AppAction;

pub struct App {
    session: EditorSession,
    command_buffer: String,
    prefix_buffers: BTreeMap<usize, String>,
    line_command_markers: BTreeMap<usize, String>,
    ui_message: Option<String>,
    should_quit: bool,
}

impl App {
    pub fn new(buffer: EditBuffer) -> Self {
        let mut session = EditorSession::new(buffer);
        session
            .execute_primary(PrimaryCommand::Number(true))
            .expect("enabling line numbers should not fail");

        Self {
            session,
            command_buffer: String::new(),
            prefix_buffers: BTreeMap::new(),
            line_command_markers: BTreeMap::new(),
            ui_message: None,
            should_quit: false,
        }
    }

    #[cfg(test)]
    pub fn session(&self) -> &EditorSession {
        &self.session
    }

    pub fn should_quit(&self) -> bool {
        self.should_quit
    }

    pub fn handle_action(&mut self, action: AppAction) -> Result<()> {
        match action {
            AppAction::Help => {
                self.ui_message = Some(
                    "Line cmds: D/Dn/DD Delete, I/In Insert, R/Rn/RR Repeat, C/CC/M/MM Copy/Move, A/B/O/OO Destination, LC/UC/LCC/UCC Case, X/XX Exclude, S Show | PF3 Save+Exit | PF5 RFind | PF6 RChange | PF12 Cancel".into(),
                );
            }
            AppAction::ExitSave => {
                if self.session.buffer().is_dirty() {
                    self.session
                        .execute_primary(PrimaryCommand::Save)
                        .map_err(anyhow::Error::msg)?;
                }
                self.session
                    .execute_primary(PrimaryCommand::End)
                    .map_err(anyhow::Error::msg)?;
            }
            AppAction::CursorUp => self.session.move_cursor_up(),
            AppAction::CursorDown => self.session.move_cursor_down(),
            AppAction::CursorLeft => match self.session.view().active_area {
                ActiveArea::DataArea if self.session.view().cursor_col == 0 => {
                    self.session.activate_line_command_area();
                }
                ActiveArea::DataArea => self.session.move_cursor_left(),
                ActiveArea::LineCommandArea | ActiveArea::CommandLine => {}
            },
            AppAction::CursorRight => match self.session.view().active_area {
                ActiveArea::LineCommandArea => self.session.activate_data_area(),
                ActiveArea::DataArea => self.session.move_cursor_right(),
                ActiveArea::CommandLine => {}
            },
            AppAction::CursorLineStart => {
                if self.session.view().active_area == ActiveArea::DataArea {
                    self.session.move_cursor_to_line_start();
                }
            }
            AppAction::CursorLineEnd => {
                if self.session.view().active_area == ActiveArea::DataArea {
                    self.session.move_cursor_to_line_end();
                }
            }
            AppAction::ScrollUp => self
                .session
                .execute_primary(PrimaryCommand::Up(1))
                .map_err(anyhow::Error::msg)?,
            AppAction::ScrollDown => self
                .session
                .execute_primary(PrimaryCommand::Down(1))
                .map_err(anyhow::Error::msg)?,
            AppAction::ScrollLeft => self
                .session
                .execute_primary(PrimaryCommand::Left(8))
                .map_err(anyhow::Error::msg)?,
            AppAction::ScrollRight => self
                .session
                .execute_primary(PrimaryCommand::Right(8))
                .map_err(anyhow::Error::msg)?,
            AppAction::RepeatFind => self.execute_primary_command(PrimaryCommand::RFind)?,
            AppAction::RepeatChange => self.execute_primary_command(PrimaryCommand::RChange)?,
            AppAction::ToggleFocus => self.session.toggle_active_area(),
            AppAction::Backspace => {
                match self.session.view().active_area {
                    ActiveArea::CommandLine => {
                        self.command_buffer.pop();
                    }
                    ActiveArea::LineCommandArea => {
                        let row = self.session.view().cursor_row;
                        if let Some(buffer) = self.prefix_buffers.get_mut(&row) {
                            buffer.pop();
                            if buffer.is_empty() {
                                self.prefix_buffers.remove(&row);
                            }
                        }
                    }
                    ActiveArea::DataArea => {}
                }
            }
            AppAction::Delete => {
                if self.session.view().active_area == ActiveArea::DataArea {
                    match self.session.delete_char() {
                        Ok(()) => self.ui_message = None,
                        Err(err) => self.ui_message = Some(err),
                    }
                }
            }
            AppAction::LineFeed => {
                if self.session.view().active_area == ActiveArea::DataArea {
                    let row = self.session.view().cursor_row;
                    if self.session.row_is_excluded(row) {
                        self.ui_message = Some("Cannot edit excluded lines".into());
                    } else {
                        self.session.insert_blank_line_after(row);
                        self.ui_message = None;
                    }
                }
            }
            AppAction::Cancel => self.execute_primary_command(PrimaryCommand::Cancel)?,
            AppAction::End => self.execute_primary_command(PrimaryCommand::End)?,
            AppAction::Execute => {
                match self.session.view().active_area {
                    ActiveArea::CommandLine => self.execute_command_line()?,
                    ActiveArea::LineCommandArea => self.execute_prefix_line()?,
                    ActiveArea::DataArea => {}
                }
            }
            AppAction::None => {}
        }
        self.sync_session_state();
        Ok(())
    }

    fn screen_model(&self, width: u16, height: u16) -> ScreenModel {
        let mut screen = render_screen(&self.session, width, height);
        screen.command_value = self.command_buffer.clone();
        screen.command_selected = self.session.view().active_area == ActiveArea::CommandLine;
        for row in &mut screen.rows {
            row.prefix = self
                .prefix_buffers
                .get(&row.record_index)
                .or_else(|| self.line_command_markers.get(&row.record_index))
                .cloned()
                .unwrap_or_default();
        }
        if let Some(message) = &self.ui_message {
            screen.message = message.clone();
        }
        screen
    }

    fn handle_key(&mut self, key: KeyEvent) -> Result<()> {
        let mapped_action = crate::input::map_key(key);
        let should_preempt_text_input = matches!(mapped_action, AppAction::LineFeed)
            || (mapped_action == AppAction::CursorDown && !key.modifiers.is_empty());
        if should_preempt_text_input {
            return self.handle_action(mapped_action);
        }

        match self.session.view().active_area {
            ActiveArea::CommandLine => match key.code {
                KeyCode::Char(ch) => {
                    self.command_buffer.push(ch);
                    self.ui_message = None;
                    Ok(())
                }
                _ => self.handle_action(crate::input::map_key(key)),
            },
            ActiveArea::LineCommandArea => match key.code {
                KeyCode::Char(ch) => {
                    let row = self.session.view().cursor_row;
                    self.prefix_buffers
                        .entry(row)
                        .or_default()
                        .push(ch.to_ascii_uppercase());
                    self.ui_message = None;
                    Ok(())
                }
                _ => self.handle_action(crate::input::map_key(key)),
            },
            ActiveArea::DataArea => {
                match key.code {
                    KeyCode::Char(ch) => {
                        match self.session.insert_char(ch) {
                            Ok(()) => self.ui_message = None,
                            Err(err) => self.ui_message = Some(err),
                        }
                        self.sync_session_state();
                        Ok(())
                    }
                    KeyCode::Backspace => {
                        match self.session.backspace_char() {
                            Ok(()) => self.ui_message = None,
                            Err(err) => self.ui_message = Some(err),
                        }
                        self.sync_session_state();
                        Ok(())
                    }
                    KeyCode::Enter => {
                        match self.session.split_line_at_cursor() {
                            Ok(()) => self.ui_message = None,
                            Err(err) => self.ui_message = Some(err),
                        }
                        self.sync_session_state();
                        Ok(())
                    }
                    _ => self.handle_action(crate::input::map_key(key)),
                }
            }
        }
    }

    fn execute_command_line(&mut self) -> Result<()> {
        let command_text = self.command_buffer.trim().to_string();
        if command_text.is_empty() {
            return Ok(());
        }

        if let Some(prefix_text) = command_text.strip_prefix(':') {
            let prefix_text = prefix_text.trim();
            let command = match parse_prefix(prefix_text) {
                Ok(command) => command,
                Err(err) => {
                    self.ui_message = Some(err);
                    return Ok(());
                }
            };
            let row = self.session.view().cursor_row;
            if let Err(err) = self.session.execute_prefix(row, command) {
                self.ui_message = Some(err);
                self.sync_line_command_markers();
                return Ok(());
            }
            self.command_buffer.clear();
            self.ui_message = None;
            self.session.activate_data_area();
            self.sync_line_command_markers();
            return Ok(());
        }

        match parse_primary(&command_text) {
            Ok(command) => {
                self.execute_primary_command(command)?;
                self.command_buffer.clear();
                self.ui_message = None;
            }
            Err(err) => {
                self.ui_message = Some(err);
            }
        }

        Ok(())
    }

    fn execute_primary_command(&mut self, command: PrimaryCommand) -> Result<()> {
        let return_to_data_area = returns_focus_to_data_area(&command);
        self.session
            .execute_primary(command)
            .map_err(anyhow::Error::msg)?;
        if return_to_data_area {
            self.session.activate_data_area();
        }
        Ok(())
    }

    fn execute_prefix_line(&mut self) -> Result<()> {
        let mut commands = Vec::new();
        for (row, value) in &self.prefix_buffers {
            let prefix_text = value.trim().to_string();
            if prefix_text.is_empty() {
                continue;
            }

            let command = match parse_prefix(&prefix_text) {
                Ok(command) => command,
                Err(err) => {
                    self.ui_message = Some(err);
                    return Ok(());
                }
            };
            commands.push((*row, prefix_text, command));
        }

        if commands.is_empty() {
            return Ok(());
        }

        commands.sort_by_key(|(row, _, _)| *row);

        self.prefix_buffers.clear();

        let mut row_offset = 0isize;
        for (row, _, command) in commands {
            let buffer_len = self.session.buffer().records().len();
            let effective_row = if buffer_len == 0 {
                0
            } else {
                let adjusted = row as isize + row_offset;
                adjusted.clamp(0, buffer_len.saturating_sub(1) as isize) as usize
            };
            let before_len = self.session.buffer().records().len() as isize;
            if let Err(err) = self.session.execute_prefix(effective_row, command) {
                self.ui_message = Some(err);
                self.sync_line_command_markers();
                return Ok(());
            }
            let after_len = self.session.buffer().records().len() as isize;
            row_offset += after_len - before_len;
        }
        self.sync_line_command_markers();
        self.ui_message = None;

        Ok(())
    }

    fn sync_session_state(&mut self) {
        self.should_quit = self.should_quit || self.session.should_exit();
    }

    fn sync_line_command_markers(&mut self) {
        self.line_command_markers.clear();
        if let Some(row) = self.session.pending_exclude_block() {
            self.line_command_markers.insert(row, "XX".into());
        }
        if let Some(row) = self.session.pending_delete_block() {
            self.line_command_markers.insert(row, "DD".into());
        }
        if let Some(row) = self.session.pending_repeat_block() {
            self.line_command_markers.insert(row, "RR".into());
        }
        if let Some(row) = self.session.pending_copy_block() {
            self.line_command_markers.insert(row, "CC".into());
        }
        if let Some((start, end)) = self.session.pending_copy_range() {
            let display = self.session.pending_copy_display().unwrap_or("CC");
            self.line_command_markers.insert(start, display.into());
            self.line_command_markers.insert(end, display.into());
        }
        if let Some(row) = self.session.pending_move_block() {
            self.line_command_markers.insert(row, "MM".into());
        }
        if let Some((start, end)) = self.session.pending_move_range() {
            let display = self.session.pending_move_display().unwrap_or("MM");
            self.line_command_markers.insert(start, display.into());
            self.line_command_markers.insert(end, display.into());
        }
        if let Some(row) = self.session.pending_overlay_block() {
            self.line_command_markers.insert(row, "OO".into());
        }
        if let Some((start, end)) = self.session.pending_overlay_range() {
            let display = self.session.pending_overlay_display().unwrap_or("OO");
            self.line_command_markers.insert(start, display.into());
            self.line_command_markers.insert(end, display.into());
        }
        if let Some(row) = self.session.pending_lowercase_block() {
            self.line_command_markers.insert(row, "LCC".into());
        }
        if let Some(row) = self.session.pending_uppercase_block() {
            self.line_command_markers.insert(row, "UCC".into());
        }
        if let Some(destination) = self.session.pending_destination() {
            match destination {
                ispf_core::Destination::After(row) => {
                    self.line_command_markers.insert(row, "A".into());
                }
                ispf_core::Destination::Before(row) => {
                    self.line_command_markers.insert(row, "B".into());
                }
                ispf_core::Destination::Overlay(row) => {
                    self.line_command_markers.insert(row, "O".into());
                }
            }
        }
    }
}

fn returns_focus_to_data_area(command: &PrimaryCommand) -> bool {
    matches!(
        command,
        PrimaryCommand::Find { .. }
            | PrimaryCommand::RFind
            | PrimaryCommand::Change { .. }
            | PrimaryCommand::RChange
            | PrimaryCommand::Locate { .. }
            | PrimaryCommand::Reset
    )
}

fn visible_command_value(screen: &ScreenModel) -> String {
    if screen.command_selected && screen.command_value.is_empty() {
        String::from(" ")
    } else {
        screen.command_value.clone()
    }
}

fn ispf_black() -> Color {
    Color::Rgb(0, 0, 0)
}

fn ispf_white() -> Color {
    Color::Rgb(240, 244, 248)
}

fn ispf_cyan() -> Color {
    Color::Rgb(110, 235, 255)
}

fn ispf_green() -> Color {
    Color::Rgb(122, 214, 120)
}

fn data_text_style() -> Style {
    Style::default().fg(ispf_green()).bg(ispf_black())
}

fn menu_style() -> Style {
    Style::default()
        .fg(ispf_white())
        .bg(ispf_black())
        .add_modifier(Modifier::BOLD)
}

fn header_style() -> Style {
    Style::default()
        .fg(ispf_cyan())
        .bg(ispf_black())
        .add_modifier(Modifier::BOLD)
}

fn metadata_style() -> Style {
    Style::default().fg(ispf_green()).bg(ispf_black())
}

fn active_field_style() -> Style {
    Style::default()
        .fg(ispf_black())
        .bg(ispf_white())
        .add_modifier(Modifier::BOLD)
}

fn data_spans(
    text: &str,
    selected: bool,
    cursor_col: Option<usize>,
) -> Vec<Span<'static>> {
    if !selected {
        return vec![Span::styled(text.to_string(), data_text_style())];
    }

    let chars: Vec<char> = text.chars().collect();
    let cursor_col = cursor_col.unwrap_or(0).min(chars.len());
    let before: String = chars[..cursor_col].iter().collect();
    let active = chars
        .get(cursor_col)
        .map(|ch| ch.to_string())
        .unwrap_or_else(|| " ".to_string());
    let after: String = if cursor_col < chars.len() {
        chars[cursor_col + 1..].iter().collect()
    } else {
        String::new()
    };

    let mut spans = Vec::new();
    if !before.is_empty() {
        spans.push(Span::styled(before, data_text_style()));
    }
    spans.push(Span::styled(active, active_field_style()));
    if !after.is_empty() {
        spans.push(Span::styled(after, data_text_style()));
    }
    spans
}

fn line_command_field_spans(row: &ispf_screen::ScreenRow) -> Vec<Span<'static>> {
    let base = if row.line_number.is_empty() {
        String::from("      ")
    } else {
        row.line_number.clone()
    };

    let display = if row.prefix.is_empty() {
        if row.line_number.is_empty() {
            String::from("      ")
        } else {
            row.line_number.clone()
        }
    } else {
        let mut chars: Vec<char> = base.chars().collect();
        for (index, ch) in row.prefix.chars().take(chars.len()).enumerate() {
            chars[index] = ch;
        }
        chars.into_iter().collect()
    };

    if row.prefix_selected {
        vec![Span::styled(display, active_field_style())]
    } else {
        vec![Span::styled(display, metadata_style())]
    }
}

fn data_row_line(row: &ispf_screen::ScreenRow) -> Line<'static> {
    let mut spans = line_command_field_spans(row);
    spans.push(Span::styled(" ".to_string(), data_text_style()));
    if row.is_excluded_placeholder {
        spans.push(Span::styled(
            row.text.clone(),
            if row.text_selected {
                active_field_style()
            } else {
                menu_style()
            },
        ));
    } else {
        spans.extend(data_spans(&row.text, row.text_selected, row.text_cursor_col));
    }

    Line::from(spans)
}

fn status_line_text(message: &str, status_summary: &str, width: u16) -> String {
    let width = usize::from(width);
    if message.is_empty() {
        return status_summary.to_string();
    }

    let total_len = message.len() + status_summary.len();
    if total_len + 1 > width {
        format!("{message} {status_summary}")
    } else {
        format!(
            "{message}{}{status_summary}",
            " ".repeat(width.saturating_sub(total_len))
        )
    }
}

#[cfg(test)]
fn right_aligned_text(left: &str, right: &str, width: u16) -> String {
    let width = usize::from(width);
    let total_len = left.len() + right.len();
    if total_len + 1 > width {
        format!("{left} {right}")
    } else {
        format!(
            "{left}{}{right}",
            " ".repeat(width.saturating_sub(total_len))
        )
    }
}

fn initialize_startup_focus(app: &mut App) {
    app.session.reset_view_to_top();
    app.session.activate_command_line();
}

pub fn run() -> Result<()> {
    let buffer = if let Some(path) = std::env::args().nth(1) {
        EditBuffer::from_path(std::path::Path::new(&path))?
    } else {
        EditBuffer::from_text("ISPF EDITOR\n").unwrap()
    };

    let mut app = App::new(buffer);
    initialize_startup_focus(&mut app);

    if !stdout().is_terminal() {
        let _screen = app.screen_model(80, 24);
        return Ok(());
    }

    enable_raw_mode()?;
    let mut out = stdout();
    execute!(out, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(out);
    let mut terminal = Terminal::new(backend)?;

    let run_result = run_loop(&mut terminal, &mut app);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    run_result
}

fn run_loop(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    app: &mut App,
) -> Result<()> {
    while !app.should_quit() {
        terminal.draw(|frame| {
            let area = frame.area();
            frame.render_widget(Clear, area);
            frame.render_widget(Block::default().style(data_text_style()), area);
            let screen = app.screen_model(area.width, area.height);
            let command_value = visible_command_value(&screen);
            let sections = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(1),
                    Constraint::Length(1),
                    Constraint::Length(1),
                    Constraint::Min(1),
                    Constraint::Length(1),
                    Constraint::Length(1),
                    Constraint::Length(1),
                ])
                .split(area);

            frame.render_widget(
                Paragraph::new(screen.menu_bar).style(menu_style()),
                sections[0],
            );

            let header_sections = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Min(0),
                    Constraint::Length(screen.header_right.len() as u16),
                ])
                .split(sections[1]);

            frame.render_widget(
                Paragraph::new(format!("{}    {}", screen.header_left, screen.title))
                    .style(header_style()),
                header_sections[0],
            );
            frame.render_widget(
                Paragraph::new(screen.header_right.clone()).style(metadata_style()),
                header_sections[1],
            );

            let command_style = if screen.command_selected {
                active_field_style()
            } else {
                data_text_style()
            };
            let command_line = Line::from(vec![
                Span::styled(screen.command_prompt, metadata_style()),
                Span::raw(" "),
                Span::styled(command_value, command_style),
                Span::raw("    "),
                Span::styled(screen.scroll_label, metadata_style()),
                Span::raw(" "),
                Span::styled(screen.scroll_value, header_style()),
            ]);
            frame.render_widget(
                Paragraph::new(command_line).style(data_text_style()),
                sections[2],
            );

            let rows = screen.rows.into_iter().map(|row| data_row_line(&row));
            let mut body_lines = Vec::new();
            if let Some(banner) = screen.data_banner.clone() {
                body_lines.push(Line::from(vec![Span::styled(
                    banner,
                    menu_style(),
                )]));
            }
            if let Some(cols_line) = screen.cols_line.clone() {
                body_lines.push(Line::from(vec![Span::styled(
                    cols_line,
                    header_style(),
                )]));
            }
            if let Some(bounds_line) = screen.bounds_line.clone() {
                body_lines.push(Line::from(vec![Span::styled(
                    bounds_line,
                    metadata_style(),
                )]));
            }
            body_lines.extend(rows);
            if let Some(banner) = screen.bottom_banner.clone() {
                body_lines.push(Line::from(vec![Span::styled(
                    banner,
                    menu_style(),
                )]));
            }

            frame.render_widget(
                Paragraph::new(body_lines)
                    .style(data_text_style())
                    .block(Block::default().borders(Borders::ALL).style(metadata_style())),
                sections[3],
            );

            frame.render_widget(
                Paragraph::new(status_line_text(
                    &screen.message,
                    &screen.status_summary,
                    sections[4].width,
                ))
                .style(metadata_style()),
                sections[4],
            );

            frame.render_widget(
                Paragraph::new(screen.footer_keys[0].clone()).style(menu_style()),
                sections[5],
            );

            frame.render_widget(
                Paragraph::new(screen.footer_keys[1].clone()).style(menu_style()),
                sections[6],
            );
        })?;

        if event::poll(std::time::Duration::from_millis(250))?
            && let Event::Key(key) = event::read()?
        {
            app.handle_key(key)?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::App;
    use crate::input::AppAction;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use ispf_core::{ActiveArea, EditBuffer};
    use ispf_screen::ScreenRow;
    use ratatui::style::Modifier;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static NEXT_TEMP_ID: AtomicUsize = AtomicUsize::new(1);

    fn unique_temp_path(name: &str) -> PathBuf {
        let id = NEXT_TEMP_ID.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!(
            "ispf-tui-{name}-{}-{id}.txt",
            std::process::id()
        ))
    }

    #[test]
    fn cursor_actions_move_the_active_row() {
        let mut app = App::new(EditBuffer::from_text("A\nB\nC\n").unwrap());

        app.handle_action(AppAction::CursorDown).unwrap();
        app.handle_action(AppAction::CursorDown).unwrap();
        app.handle_action(AppAction::CursorUp).unwrap();

        assert_eq!(app.session().view().cursor_row, 1);
    }

    #[test]
    fn f1_displays_a_compact_help_message() {
        let mut app = App::new(EditBuffer::from_text("A\n").unwrap());

        app.handle_key(KeyEvent::from(KeyCode::F(1))).unwrap();

        assert!(app.screen_model(120, 24).message.contains("Line cmds:"));
        assert!(app.screen_model(120, 24).message.contains("PF12 Cancel"));
    }

    #[test]
    fn data_area_collects_text_and_backspace_edits_current_record() {
        let mut app = App::new(EditBuffer::from_text("AB\n").unwrap());

        app.handle_key(KeyEvent::from(KeyCode::Right)).unwrap();
        app.handle_key(KeyEvent::from(KeyCode::Char('x'))).unwrap();
        app.handle_key(KeyEvent::from(KeyCode::Backspace)).unwrap();

        assert_eq!(app.session().buffer().records()[0].text(), "A");
        assert_eq!(app.session().view().cursor_col, 1);
    }

    #[test]
    fn data_area_delete_removes_the_character_under_the_cursor() {
        let mut app = App::new(EditBuffer::from_text("AB\n").unwrap());

        app.handle_key(KeyEvent::from(KeyCode::Delete)).unwrap();

        assert_eq!(app.session().buffer().to_text(), "B\n");
        assert_eq!(app.session().view().cursor_col, 0);
    }

    #[test]
    fn data_area_backspace_at_column_zero_joins_the_previous_line() {
        let mut app = App::new(EditBuffer::from_text("AB\nCD\n").unwrap());

        app.handle_key(KeyEvent::from(KeyCode::Down)).unwrap();
        app.handle_key(KeyEvent::from(KeyCode::Backspace)).unwrap();

        assert_eq!(app.session().buffer().to_text(), "ABCD\n");
        assert_eq!(app.session().view().cursor_row, 0);
        assert_eq!(app.session().view().cursor_col, 2);
    }

    #[test]
    fn data_area_delete_at_end_of_line_joins_the_next_line() {
        let mut app = App::new(EditBuffer::from_text("AB\nCD\n").unwrap());

        app.handle_key(KeyEvent::from(KeyCode::Right)).unwrap();
        app.handle_key(KeyEvent::from(KeyCode::Right)).unwrap();
        app.handle_key(KeyEvent::from(KeyCode::Delete)).unwrap();

        assert_eq!(app.session().buffer().to_text(), "ABCD\n");
        assert_eq!(app.session().view().cursor_row, 0);
        assert_eq!(app.session().view().cursor_col, 2);
    }

    #[test]
    fn data_area_enter_splits_the_line_at_the_cursor() {
        let mut app = App::new(EditBuffer::from_text("ABCD\n").unwrap());

        app.handle_key(KeyEvent::from(KeyCode::Right)).unwrap();
        app.handle_key(KeyEvent::from(KeyCode::Right)).unwrap();
        app.handle_key(KeyEvent::from(KeyCode::Enter)).unwrap();

        assert_eq!(app.session().buffer().to_text(), "AB\nCD\n");
        assert_eq!(app.session().view().cursor_row, 1);
        assert_eq!(app.session().view().cursor_col, 0);
    }

    #[test]
    fn undo_after_split_restores_the_original_line_via_command_line() {
        let mut app = App::new(EditBuffer::from_text("ABCD\n").unwrap());

        app.handle_key(KeyEvent::from(KeyCode::Right)).unwrap();
        app.handle_key(KeyEvent::from(KeyCode::Right)).unwrap();
        app.handle_key(KeyEvent::from(KeyCode::Enter)).unwrap();
        app.handle_action(AppAction::ToggleFocus).unwrap();
        for ch in "UNDO".chars() {
            app.handle_key(KeyEvent::from(KeyCode::Char(ch))).unwrap();
        }
        app.handle_action(AppAction::Execute).unwrap();

        assert_eq!(app.session().buffer().to_text(), "ABCD\n");
    }

    #[test]
    fn undo_after_join_restores_the_original_lines_via_command_line() {
        let mut app = App::new(EditBuffer::from_text("AB\nCD\n").unwrap());

        app.handle_key(KeyEvent::from(KeyCode::Down)).unwrap();
        app.handle_key(KeyEvent::from(KeyCode::Backspace)).unwrap();
        app.handle_action(AppAction::ToggleFocus).unwrap();
        for ch in "UNDO".chars() {
            app.handle_key(KeyEvent::from(KeyCode::Char(ch))).unwrap();
        }
        app.handle_action(AppAction::Execute).unwrap();

        assert_eq!(app.session().buffer().to_text(), "AB\nCD\n");
    }

    #[test]
    fn dirty_buffer_marks_the_title() {
        let mut app = App::new(EditBuffer::from_text("AB\n").unwrap());

        app.handle_key(KeyEvent::from(KeyCode::Char('x'))).unwrap();

        assert!(app.screen_model(80, 24).title.contains('*'));
    }

    #[test]
    fn data_area_arrow_keys_move_the_horizontal_cursor() {
        let mut app = App::new(EditBuffer::from_text("AB\n").unwrap());

        app.handle_key(KeyEvent::from(KeyCode::Right)).unwrap();
        app.handle_key(KeyEvent::from(KeyCode::Right)).unwrap();
        app.handle_key(KeyEvent::from(KeyCode::Left)).unwrap();

        assert_eq!(app.session().view().cursor_col, 1);
    }

    #[test]
    fn data_area_home_and_end_move_to_line_start_and_end() {
        let mut app = App::new(EditBuffer::from_text("ABCD\n").unwrap());

        app.handle_key(KeyEvent::from(KeyCode::Right)).unwrap();
        app.handle_key(KeyEvent::from(KeyCode::Right)).unwrap();
        app.handle_key(KeyEvent::from(KeyCode::End)).unwrap();
        assert_eq!(app.session().view().cursor_col, 4);

        app.handle_key(KeyEvent::from(KeyCode::Home)).unwrap();
        assert_eq!(app.session().view().cursor_col, 0);
    }

    #[test]
    fn left_from_data_column_zero_moves_focus_to_line_command_area() {
        let mut app = App::new(EditBuffer::from_text("AB\n").unwrap());

        app.handle_key(KeyEvent::from(KeyCode::Left)).unwrap();

        assert_eq!(app.session().view().active_area, ActiveArea::LineCommandArea);
        assert_eq!(app.session().view().cursor_col, 0);
    }

    #[test]
    fn right_from_line_command_area_returns_focus_to_data_area() {
        let mut app = App::new(EditBuffer::from_text("AB\n").unwrap());

        app.handle_action(AppAction::ToggleFocus).unwrap();
        app.handle_action(AppAction::ToggleFocus).unwrap();
        app.handle_key(KeyEvent::from(KeyCode::Right)).unwrap();

        assert_eq!(app.session().view().active_area, ActiveArea::DataArea);
        assert_eq!(app.session().view().cursor_col, 0);
    }

    #[test]
    fn selected_data_row_renders_a_visible_cursor_cell() {
        let app = App::new(EditBuffer::from_text("AB\n").unwrap());
        let screen = app.screen_model(80, 24);

        assert_eq!(screen.rows[0].text_cursor_col, Some(0));
    }

    #[test]
    fn toggle_focus_cycles_command_line_then_line_command_then_data_area() {
        let mut app = App::new(EditBuffer::from_text("A\n").unwrap());

        app.handle_action(AppAction::ToggleFocus).unwrap();
        assert_eq!(app.session().view().active_area, ActiveArea::CommandLine);

        app.handle_action(AppAction::ToggleFocus).unwrap();
        assert_eq!(app.session().view().active_area, ActiveArea::LineCommandArea);

        app.handle_action(AppAction::ToggleFocus).unwrap();
        assert_eq!(app.session().view().active_area, ActiveArea::DataArea);
    }

    #[test]
    fn scroll_actions_shift_the_viewport() {
        let mut app = App::new(EditBuffer::from_text("A\nB\nC\nD\n").unwrap());

        app.handle_action(AppAction::ScrollDown).unwrap();
        app.handle_action(AppAction::ScrollDown).unwrap();
        app.handle_action(AppAction::ScrollUp).unwrap();

        assert_eq!(app.session().view().top_row, 1);
    }

    #[test]
    fn end_action_marks_the_app_for_exit() {
        let mut app = App::new(EditBuffer::from_text("A\n").unwrap());

        app.handle_action(AppAction::End).unwrap();

        assert!(app.should_quit());
    }

    #[test]
    fn f3_saves_dirty_buffer_and_quits() {
        let path = unique_temp_path("f3-save-exit");
        std::fs::write(&path, "A\n").unwrap();
        let mut app = App::new(EditBuffer::from_path(&path).unwrap());

        app.handle_key(KeyEvent::from(KeyCode::Char('x'))).unwrap();
        app.handle_key(KeyEvent::from(KeyCode::F(3))).unwrap();

        assert!(app.should_quit());
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "x\n");
    }

    #[test]
    fn end_action_uses_end_semantics_for_dirty_buffers() {
        let mut app = App::new(EditBuffer::from_text("A\n").unwrap());

        app.handle_key(KeyEvent::from(KeyCode::Char('x'))).unwrap();
        app.handle_action(AppAction::End).unwrap();

        assert!(!app.should_quit());
        assert_eq!(app.screen_model(80, 24).message, "Use SAVE or CANCEL before END");
    }

    #[test]
    fn command_line_end_quits_when_buffer_is_clean() {
        let mut app = App::new(EditBuffer::from_text("A\n").unwrap());

        app.handle_action(AppAction::ToggleFocus).unwrap();
        for ch in "END".chars() {
            app.handle_key(KeyEvent::from(KeyCode::Char(ch))).unwrap();
        }
        app.handle_action(AppAction::Execute).unwrap();

        assert!(app.should_quit());
    }

    #[test]
    fn command_line_end_stays_open_when_buffer_is_dirty() {
        let mut app = App::new(EditBuffer::from_text("A\n").unwrap());

        app.handle_key(KeyEvent::from(KeyCode::Char('x'))).unwrap();
        app.handle_action(AppAction::ToggleFocus).unwrap();
        for ch in "END".chars() {
            app.handle_key(KeyEvent::from(KeyCode::Char(ch))).unwrap();
        }
        app.handle_action(AppAction::Execute).unwrap();

        assert!(!app.should_quit());
        assert_eq!(app.screen_model(80, 24).message, "Use SAVE or CANCEL before END");
    }

    #[test]
    fn command_line_cancel_restores_buffer_and_quits() {
        let mut app = App::new(EditBuffer::from_text("A\n").unwrap());

        app.handle_key(KeyEvent::from(KeyCode::Char('x'))).unwrap();
        app.handle_action(AppAction::ToggleFocus).unwrap();
        for ch in "CANCEL".chars() {
            app.handle_key(KeyEvent::from(KeyCode::Char(ch))).unwrap();
        }
        app.handle_action(AppAction::Execute).unwrap();

        assert!(app.should_quit());
        assert_eq!(app.session().buffer().to_text(), "A\n");
    }

    #[test]
    fn f12_cancels_and_restores_the_original_buffer() {
        let mut app = App::new(EditBuffer::from_text("A\n").unwrap());

        app.handle_key(KeyEvent::from(KeyCode::Char('x'))).unwrap();
        app.handle_key(KeyEvent::from(KeyCode::F(12))).unwrap();

        assert!(app.should_quit());
        assert_eq!(app.session().buffer().to_text(), "A\n");
    }

    #[test]
    fn command_line_save_clears_dirty_title_and_writes_the_file() {
        let path = unique_temp_path("save");
        std::fs::write(&path, "A\n").unwrap();
        let file_name = path.file_name().unwrap().to_string_lossy().to_string();
        let mut app = App::new(EditBuffer::from_path(&path).unwrap());

        app.handle_key(KeyEvent::from(KeyCode::Char('x'))).unwrap();
        assert_eq!(app.screen_model(80, 24).title, format!("{file_name} *"));

        app.handle_action(AppAction::ToggleFocus).unwrap();
        for ch in "SAVE".chars() {
            app.handle_key(KeyEvent::from(KeyCode::Char(ch))).unwrap();
        }
        app.handle_action(AppAction::Execute).unwrap();

        assert!(!app.should_quit());
        assert_eq!(app.screen_model(80, 24).title, file_name);
        assert_eq!(app.screen_model(80, 24).message, "Save completed");
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "x\n");
    }

    #[test]
    fn command_line_find_returns_focus_to_data_area_on_the_match() {
        let mut app = App::new(EditBuffer::from_text("ONE\nTWO\nTHREE\n").unwrap());

        app.handle_action(AppAction::ToggleFocus).unwrap();
        for ch in "FIND THREE".chars() {
            app.handle_key(KeyEvent::from(KeyCode::Char(ch))).unwrap();
        }
        app.handle_action(AppAction::Execute).unwrap();

        assert_eq!(app.session().view().cursor_row, 2);
        assert_eq!(app.session().view().active_area, ActiveArea::DataArea);
        assert!(app.screen_model(80, 24).rows[0].text_selected);
    }

    #[test]
    fn command_line_locate_returns_focus_to_data_area_on_the_requested_line() {
        let mut app = App::new(EditBuffer::from_text("ONE\nTWO\nTHREE\nFOUR\n").unwrap());

        app.handle_action(AppAction::ToggleFocus).unwrap();
        for ch in "LOCATE 3".chars() {
            app.handle_key(KeyEvent::from(KeyCode::Char(ch))).unwrap();
        }
        app.handle_action(AppAction::Execute).unwrap();

        assert_eq!(app.session().view().cursor_row, 2);
        assert_eq!(app.session().view().top_row, 2);
        assert_eq!(app.session().view().active_area, ActiveArea::DataArea);
    }

    #[test]
    fn f5_repeats_the_last_find_and_returns_focus_to_data_area() {
        let mut app = App::new(EditBuffer::from_text("ONE\nTWO\nTHREE TWO\n").unwrap());

        app.handle_action(AppAction::ToggleFocus).unwrap();
        for ch in "FIND TWO".chars() {
            app.handle_key(KeyEvent::from(KeyCode::Char(ch))).unwrap();
        }
        app.handle_action(AppAction::Execute).unwrap();
        app.handle_action(AppAction::CursorDown).unwrap();
        app.handle_key(KeyEvent::from(KeyCode::F(5))).unwrap();

        assert_eq!(app.session().view().cursor_row, 1);
        assert_eq!(app.session().view().active_area, ActiveArea::DataArea);
    }

    #[test]
    fn command_line_change_returns_focus_to_data_area_on_the_changed_row() {
        let mut app = App::new(EditBuffer::from_text("OLD\nKEEP\n").unwrap());

        app.handle_action(AppAction::ToggleFocus).unwrap();
        for ch in "CHANGE OLD NEW".chars() {
            app.handle_key(KeyEvent::from(KeyCode::Char(ch))).unwrap();
        }
        app.handle_action(AppAction::Execute).unwrap();

        assert_eq!(app.session().buffer().records()[0].text(), "NEW");
        assert_eq!(app.session().view().cursor_row, 0);
        assert_eq!(app.session().view().active_area, ActiveArea::DataArea);
    }

    #[test]
    fn command_line_colon_prefix_executes_at_the_current_row() {
        let mut app = App::new(EditBuffer::from_text("A\nB\nC\nD\n").unwrap());

        app.handle_action(AppAction::CursorDown).unwrap();
        app.handle_action(AppAction::ToggleFocus).unwrap();
        for ch in ":D2".chars() {
            app.handle_key(KeyEvent::from(KeyCode::Char(ch))).unwrap();
        }
        app.handle_action(AppAction::Execute).unwrap();

        assert_eq!(app.session().buffer().to_text(), "A\nD\n");
        assert_eq!(app.session().view().cursor_row, 1);
        assert_eq!(app.session().view().active_area, ActiveArea::DataArea);
    }

    #[test]
    fn f6_repeats_the_last_change() {
        let mut app = App::new(EditBuffer::from_text("OLD\nOLD\n").unwrap());

        app.handle_action(AppAction::ToggleFocus).unwrap();
        for ch in "CHANGE OLD NEW".chars() {
            app.handle_key(KeyEvent::from(KeyCode::Char(ch))).unwrap();
        }
        app.handle_action(AppAction::Execute).unwrap();
        app.handle_action(AppAction::CursorDown).unwrap();
        app.handle_key(KeyEvent::from(KeyCode::F(6))).unwrap();

        assert_eq!(app.session().buffer().records()[0].text(), "NEW");
        assert_eq!(app.session().buffer().records()[1].text(), "NEW");
        assert_eq!(app.session().view().active_area, ActiveArea::DataArea);
    }

    #[test]
    fn prefix_exclude_hides_the_row_and_moves_selection_to_the_next_visible_line() {
        let mut app = App::new(EditBuffer::from_text("A\nB\nC\n").unwrap());

        app.handle_action(AppAction::ToggleFocus).unwrap();
        app.handle_action(AppAction::ToggleFocus).unwrap();
        app.handle_key(KeyEvent::from(KeyCode::Char('x'))).unwrap();
        app.handle_action(AppAction::Execute).unwrap();

        let screen = app.screen_model(80, 24);
        assert_eq!(app.session().view().cursor_row, 1);
        assert_eq!(screen.rows[0].text, "1 line excluded");
        assert_eq!(screen.rows[1].text, "B");
        assert!(screen.rows[1].prefix_selected);
        assert_eq!(screen.message, "Line excluded");
    }

    #[test]
    fn prefix_input_stays_on_the_selected_visible_row_after_hidden_lines() {
        let mut app = App::new(EditBuffer::from_text("A\nB\nC\n").unwrap());

        app.handle_action(AppAction::ToggleFocus).unwrap();
        app.handle_action(AppAction::ToggleFocus).unwrap();
        app.handle_key(KeyEvent::from(KeyCode::Char('x'))).unwrap();
        app.handle_action(AppAction::Execute).unwrap();
        app.handle_key(KeyEvent::from(KeyCode::Char('d'))).unwrap();

        let screen = app.screen_model(80, 24);
        assert_eq!(screen.rows[0].text, "1 line excluded");
        assert_eq!(screen.rows[0].prefix.trim(), "");
        assert_eq!(screen.rows[1].text, "B");
        assert_eq!(screen.rows[1].prefix.trim(), "D");
        assert_eq!(screen.rows[2].text, "C");
        assert_eq!(screen.rows[2].prefix.trim(), "");
    }

    #[test]
    fn exclude_block_start_marker_stays_visible_until_the_range_is_closed() {
        let mut app = App::new(EditBuffer::from_text("A\nB\nC\nD\n").unwrap());

        app.handle_action(AppAction::ToggleFocus).unwrap();
        app.handle_action(AppAction::ToggleFocus).unwrap();
        app.handle_key(KeyEvent::from(KeyCode::Char('x'))).unwrap();
        app.handle_key(KeyEvent::from(KeyCode::Char('x'))).unwrap();
        app.handle_action(AppAction::Execute).unwrap();

        let screen = app.screen_model(80, 24);
        assert_eq!(screen.rows[0].prefix.trim(), "XX");
        assert_eq!(screen.message, "Exclude block start set");

        app.handle_action(AppAction::CursorDown).unwrap();
        app.handle_action(AppAction::CursorDown).unwrap();
        app.handle_key(KeyEvent::from(KeyCode::Char('x'))).unwrap();
        app.handle_key(KeyEvent::from(KeyCode::Char('x'))).unwrap();
        app.handle_action(AppAction::Execute).unwrap();

        let screen = app.screen_model(80, 24);
        assert_eq!(screen.rows[0].text, "3 lines excluded");
        assert_eq!(screen.rows[0].prefix.trim(), "");
        assert_eq!(screen.rows[1].text, "D");
        assert_eq!(screen.message, "3 lines excluded");
    }

    #[test]
    fn delete_block_start_marker_stays_visible_until_the_range_is_closed() {
        let mut app = App::new(EditBuffer::from_text("A\nB\nC\nD\n").unwrap());

        app.handle_action(AppAction::ToggleFocus).unwrap();
        app.handle_action(AppAction::ToggleFocus).unwrap();
        app.handle_key(KeyEvent::from(KeyCode::Char('d'))).unwrap();
        app.handle_key(KeyEvent::from(KeyCode::Char('d'))).unwrap();
        app.handle_action(AppAction::Execute).unwrap();

        let screen = app.screen_model(80, 24);
        assert_eq!(screen.rows[0].prefix.trim(), "DD");
        assert_eq!(screen.message, "Delete block start set");
    }

    #[test]
    fn delete_block_completes_when_second_dd_is_entered() {
        let mut app = App::new(EditBuffer::from_text("A\nB\nC\nD\nE\n").unwrap());

        app.handle_action(AppAction::ToggleFocus).unwrap();
        app.handle_action(AppAction::ToggleFocus).unwrap();
        app.handle_key(KeyEvent::from(KeyCode::Char('d'))).unwrap();
        app.handle_key(KeyEvent::from(KeyCode::Char('d'))).unwrap();
        app.handle_action(AppAction::Execute).unwrap();
        app.handle_action(AppAction::CursorDown).unwrap();
        app.handle_action(AppAction::CursorDown).unwrap();
        app.handle_key(KeyEvent::from(KeyCode::Char('d'))).unwrap();
        app.handle_key(KeyEvent::from(KeyCode::Char('d'))).unwrap();
        app.handle_action(AppAction::Execute).unwrap();

        let screen = app.screen_model(80, 24);
        assert_eq!(app.session().buffer().to_text(), "D\nE\n");
        assert_eq!(screen.rows[0].text, "D");
        assert_eq!(screen.rows[1].text, "E");
        assert_eq!(screen.message, "3 lines deleted");
    }

    #[test]
    fn repeat_block_start_marker_stays_visible_until_the_range_is_closed() {
        let mut app = App::new(EditBuffer::from_text("A\nB\nC\nD\n").unwrap());

        app.handle_action(AppAction::ToggleFocus).unwrap();
        app.handle_action(AppAction::ToggleFocus).unwrap();
        app.handle_key(KeyEvent::from(KeyCode::Char('r'))).unwrap();
        app.handle_key(KeyEvent::from(KeyCode::Char('r'))).unwrap();
        app.handle_action(AppAction::Execute).unwrap();

        let screen = app.screen_model(80, 24);
        assert_eq!(screen.rows[0].prefix.trim(), "RR");
        assert_eq!(screen.message, "Repeat block start set");
    }

    #[test]
    fn repeat_block_completes_when_second_rr_is_entered() {
        let mut app = App::new(EditBuffer::from_text("A\nB\nC\nD\n").unwrap());

        app.handle_action(AppAction::ToggleFocus).unwrap();
        app.handle_action(AppAction::ToggleFocus).unwrap();
        app.handle_key(KeyEvent::from(KeyCode::Char('r'))).unwrap();
        app.handle_key(KeyEvent::from(KeyCode::Char('r'))).unwrap();
        app.handle_action(AppAction::Execute).unwrap();
        app.handle_action(AppAction::CursorDown).unwrap();
        app.handle_key(KeyEvent::from(KeyCode::Char('r'))).unwrap();
        app.handle_key(KeyEvent::from(KeyCode::Char('r'))).unwrap();
        app.handle_action(AppAction::Execute).unwrap();

        let screen = app.screen_model(80, 24);
        assert_eq!(app.session().buffer().to_text(), "A\nB\nA\nB\nC\nD\n");
        assert_eq!(screen.message, "2 lines repeated");
    }

    #[test]
    fn delete_block_can_complete_for_a_middle_range() {
        let mut app = App::new(EditBuffer::from_text("A\nB\nC\nD\nE\nF\n").unwrap());

        app.handle_action(AppAction::ToggleFocus).unwrap();
        app.handle_action(AppAction::ToggleFocus).unwrap();
        app.handle_action(AppAction::CursorDown).unwrap();
        app.handle_action(AppAction::CursorDown).unwrap();
        app.handle_action(AppAction::CursorDown).unwrap();
        app.handle_key(KeyEvent::from(KeyCode::Char('d'))).unwrap();
        app.handle_key(KeyEvent::from(KeyCode::Char('d'))).unwrap();
        app.handle_action(AppAction::Execute).unwrap();
        app.handle_action(AppAction::CursorDown).unwrap();
        app.handle_action(AppAction::CursorDown).unwrap();
        app.handle_key(KeyEvent::from(KeyCode::Char('d'))).unwrap();
        app.handle_key(KeyEvent::from(KeyCode::Char('d'))).unwrap();
        app.handle_action(AppAction::Execute).unwrap();

        assert_eq!(app.session().buffer().to_text(), "A\nB\nC\n");
        assert_eq!(app.screen_model(80, 24).message, "3 lines deleted");
    }

    #[test]
    fn delete_block_completes_when_both_markers_are_entered_before_enter() {
        let mut app = App::new(EditBuffer::from_text("A\nB\nC\nD\nE\nF\n").unwrap());

        app.handle_action(AppAction::ToggleFocus).unwrap();
        app.handle_action(AppAction::ToggleFocus).unwrap();
        app.handle_action(AppAction::CursorDown).unwrap();
        app.handle_action(AppAction::CursorDown).unwrap();
        app.handle_action(AppAction::CursorDown).unwrap();
        app.handle_key(KeyEvent::from(KeyCode::Char('d'))).unwrap();
        app.handle_key(KeyEvent::from(KeyCode::Char('d'))).unwrap();
        app.handle_action(AppAction::CursorDown).unwrap();
        app.handle_action(AppAction::CursorDown).unwrap();
        app.handle_key(KeyEvent::from(KeyCode::Char('d'))).unwrap();
        app.handle_key(KeyEvent::from(KeyCode::Char('d'))).unwrap();
        app.handle_action(AppAction::Execute).unwrap();

        assert_eq!(app.session().buffer().to_text(), "A\nB\nC\n");
        assert_eq!(app.screen_model(80, 24).message, "3 lines deleted");
    }

    #[test]
    fn exclude_block_completes_when_both_markers_are_entered_before_enter() {
        let mut app = App::new(EditBuffer::from_text("A\nB\nC\nD\nE\nF\n").unwrap());

        app.handle_action(AppAction::ToggleFocus).unwrap();
        app.handle_action(AppAction::ToggleFocus).unwrap();
        app.handle_action(AppAction::CursorDown).unwrap();
        app.handle_action(AppAction::CursorDown).unwrap();
        app.handle_key(KeyEvent::from(KeyCode::Char('x'))).unwrap();
        app.handle_key(KeyEvent::from(KeyCode::Char('x'))).unwrap();
        app.handle_action(AppAction::CursorDown).unwrap();
        app.handle_action(AppAction::CursorDown).unwrap();
        app.handle_key(KeyEvent::from(KeyCode::Char('x'))).unwrap();
        app.handle_key(KeyEvent::from(KeyCode::Char('x'))).unwrap();
        app.handle_action(AppAction::Execute).unwrap();

        let screen = app.screen_model(80, 24);
        assert_eq!(screen.rows[0].text, "A");
        assert_eq!(screen.rows[1].text, "B");
        assert_eq!(screen.rows[2].text, "3 lines excluded");
        assert_eq!(screen.rows[3].text, "F");
        assert_eq!(screen.message, "3 lines excluded");
    }

    #[test]
    fn repeat_block_completes_when_both_markers_are_entered_before_enter() {
        let mut app = App::new(EditBuffer::from_text("A\nB\nC\nD\nE\n").unwrap());

        app.handle_action(AppAction::ToggleFocus).unwrap();
        app.handle_action(AppAction::ToggleFocus).unwrap();
        app.handle_action(AppAction::CursorDown).unwrap();
        app.handle_key(KeyEvent::from(KeyCode::Char('r'))).unwrap();
        app.handle_key(KeyEvent::from(KeyCode::Char('r'))).unwrap();
        app.handle_action(AppAction::CursorDown).unwrap();
        app.handle_action(AppAction::CursorDown).unwrap();
        app.handle_key(KeyEvent::from(KeyCode::Char('r'))).unwrap();
        app.handle_key(KeyEvent::from(KeyCode::Char('r'))).unwrap();
        app.handle_action(AppAction::Execute).unwrap();

        assert_eq!(app.session().buffer().to_text(), "A\nB\nC\nD\nB\nC\nD\nE\n");
        assert_eq!(app.screen_model(80, 24).message, "3 lines repeated");
    }

    #[test]
    fn command_line_reset_restores_hidden_rows_and_returns_focus_to_data_area() {
        let mut app = App::new(EditBuffer::from_text("A\nB\nC\n").unwrap());

        app.handle_action(AppAction::ToggleFocus).unwrap();
        app.handle_action(AppAction::ToggleFocus).unwrap();
        app.handle_key(KeyEvent::from(KeyCode::Char('x'))).unwrap();
        app.handle_action(AppAction::Execute).unwrap();
        app.handle_action(AppAction::ToggleFocus).unwrap();
        app.handle_action(AppAction::ToggleFocus).unwrap();
        for ch in "RESET".chars() {
            app.handle_key(KeyEvent::from(KeyCode::Char(ch))).unwrap();
        }
        app.handle_action(AppAction::Execute).unwrap();

        let screen = app.screen_model(80, 24);
        assert_eq!(app.session().view().active_area, ActiveArea::DataArea);
        assert_eq!(screen.rows[0].text, "A");
        assert_eq!(screen.message, "RESET completed");
    }

    #[test]
    fn excluded_block_renders_a_placeholder_row_and_s_reveals_only_that_block() {
        let mut app = App::new(EditBuffer::from_text("A\nB\nC\nD\nE\nF\n").unwrap());

        app.handle_action(AppAction::ToggleFocus).unwrap();
        app.handle_action(AppAction::ToggleFocus).unwrap();
        app.handle_key(KeyEvent::from(KeyCode::Char('x'))).unwrap();
        app.handle_key(KeyEvent::from(KeyCode::Char('x'))).unwrap();
        app.handle_action(AppAction::CursorDown).unwrap();
        app.handle_action(AppAction::CursorDown).unwrap();
        app.handle_key(KeyEvent::from(KeyCode::Char('x'))).unwrap();
        app.handle_key(KeyEvent::from(KeyCode::Char('x'))).unwrap();
        app.handle_action(AppAction::Execute).unwrap();

        let hidden = app.screen_model(80, 24);
        assert_eq!(hidden.rows[0].line_number, "000001");
        assert_eq!(hidden.rows[0].text, "3 lines excluded");

        app.handle_action(AppAction::CursorUp).unwrap();
        app.handle_key(KeyEvent::from(KeyCode::Char('s'))).unwrap();
        app.handle_action(AppAction::Execute).unwrap();

        let shown = app.screen_model(80, 24);
        assert_eq!(shown.rows[0].text, "A");
        assert_eq!(shown.rows[1].text, "B");
        assert_eq!(shown.rows[2].text, "C");
        assert_eq!(shown.rows[3].text, "D");
        assert_eq!(shown.message, "3 lines shown");
    }

    #[test]
    fn command_line_collects_text_and_backspace_edits_it() {
        let mut app = App::new(EditBuffer::from_text("A\n").unwrap());

        app.handle_action(AppAction::ToggleFocus).unwrap();
        app.handle_key(KeyEvent::from(KeyCode::Char('N'))).unwrap();
        app.handle_key(KeyEvent::from(KeyCode::Char('U'))).unwrap();
        app.handle_action(AppAction::Backspace).unwrap();

        let screen = app.screen_model(80, 24);
        assert_eq!(screen.command_value, "N");
        assert!(screen.command_selected);
    }

    #[test]
    fn enter_executes_primary_command_from_command_line() {
        let mut app = App::new(EditBuffer::from_text("A\n").unwrap());

        app.handle_action(AppAction::ToggleFocus).unwrap();
        for ch in "NUMBER".chars() {
            app.handle_key(KeyEvent::from(KeyCode::Char(ch))).unwrap();
        }
        app.handle_action(AppAction::Execute).unwrap();

        assert!(app.session().profile().number_mode);
        assert_eq!(app.screen_model(80, 24).command_value, "");
    }

    #[test]
    fn q_types_into_data_area_and_command_line() {
        let mut app = App::new(EditBuffer::from_text("A\n").unwrap());
        app.handle_key(KeyEvent::from(KeyCode::Char('q'))).unwrap();
        assert!(!app.should_quit());
        assert_eq!(app.session().buffer().records()[0].text(), "q");

        let mut app = App::new(EditBuffer::from_text("A\n").unwrap());
        app.handle_action(AppAction::ToggleFocus).unwrap();
        app.handle_key(KeyEvent::from(KeyCode::Char('q'))).unwrap();

        assert!(!app.should_quit());
        assert_eq!(app.screen_model(80, 24).command_value, "q");
    }

    #[test]
    fn end_action_still_works_from_escape() {
        let mut app = App::new(EditBuffer::from_text("A\n").unwrap());

        app.handle_key(KeyEvent::from(KeyCode::Esc)).unwrap();

        assert!(app.should_quit());
    }

    #[test]
    fn escape_uses_end_semantics_for_dirty_buffers() {
        let mut app = App::new(EditBuffer::from_text("A\n").unwrap());

        app.handle_key(KeyEvent::from(KeyCode::Char('x'))).unwrap();
        app.handle_key(KeyEvent::from(KeyCode::Esc)).unwrap();

        assert!(!app.should_quit());
        assert_eq!(app.screen_model(80, 24).message, "Use SAVE or CANCEL before END");
    }

    #[test]
    fn empty_selected_command_line_renders_a_visible_slot() {
        let mut app = App::new(EditBuffer::from_text("A\n").unwrap());
        app.handle_action(AppAction::ToggleFocus).unwrap();

        let screen = app.screen_model(80, 24);

        assert!(screen.command_selected);
        assert_eq!(super::visible_command_value(&screen), " ");
    }

    #[test]
    fn active_field_style_uses_reverse_video() {
        let style = super::active_field_style();

        assert_eq!(style.fg, Some(super::ispf_black()));
        assert_eq!(style.bg, Some(super::ispf_white()));
        assert!(style.add_modifier.contains(Modifier::BOLD));
    }

    #[test]
    fn startup_focus_selects_the_primary_command_field() {
        let mut app = App::new(EditBuffer::from_text("A\n").unwrap());

        super::initialize_startup_focus(&mut app);

        let screen = app.screen_model(80, 24);
        assert_eq!(app.session().view().active_area, ActiveArea::CommandLine);
        assert!(screen.command_selected);
        assert_eq!(super::visible_command_value(&screen), " ");
    }

    #[test]
    fn startup_tab_moves_to_the_first_line_command_row() {
        let mut app = App::new(EditBuffer::from_text("A\nB\n").unwrap());

        super::initialize_startup_focus(&mut app);
        app.handle_action(AppAction::ToggleFocus).unwrap();

        let screen = app.screen_model(80, 24);
        assert_eq!(app.session().view().active_area, ActiveArea::LineCommandArea);
        assert!(screen.rows[0].prefix_selected);
    }

    #[test]
    fn data_text_style_uses_green_on_black() {
        let style = super::data_text_style();

        assert_eq!(style.fg, Some(super::ispf_green()));
        assert_eq!(style.bg, Some(super::ispf_black()));
    }

    #[test]
    fn menu_style_uses_white_on_black() {
        let style = super::menu_style();

        assert_eq!(style.fg, Some(super::ispf_white()));
        assert_eq!(style.bg, Some(super::ispf_black()));
        assert!(style.add_modifier.contains(Modifier::BOLD));
    }

    #[test]
    fn header_style_uses_cyan_on_black() {
        let style = super::header_style();

        assert_eq!(style.fg, Some(super::ispf_cyan()));
        assert_eq!(style.bg, Some(super::ispf_black()));
        assert!(style.add_modifier.contains(Modifier::BOLD));
    }

    #[test]
    fn metadata_style_uses_green_on_black() {
        let style = super::metadata_style();

        assert_eq!(style.fg, Some(super::ispf_green()));
        assert_eq!(style.bg, Some(super::ispf_black()));
    }

    #[test]
    fn data_spans_highlight_the_active_character() {
        let spans = super::data_spans("AB", true, Some(1));

        assert_eq!(spans.len(), 2);
        assert_eq!(spans[0].content.as_ref(), "A");
        assert_eq!(spans[1].content.as_ref(), "B");
        assert_eq!(spans[1].style.fg, Some(super::ispf_black()));
        assert_eq!(spans[1].style.bg, Some(super::ispf_white()));
    }

    #[test]
    fn status_line_text_right_aligns_the_status_summary_when_space_allows() {
        let line = super::status_line_text("Save completed", "DATA  Ln 1 Col 1", 40);

        assert!(line.starts_with("Save completed"));
        assert!(line.ends_with("DATA  Ln 1 Col 1"));
        assert_eq!(line.len(), 40);
    }

    #[test]
    fn right_aligned_text_places_header_status_on_the_right() {
        let line = super::right_aligned_text("EDIT    sample.txt", "Columns 00001 00144", 50);

        assert!(line.starts_with("EDIT    sample.txt"));
        assert!(line.ends_with("Columns 00001 00144"));
        assert_eq!(line.len(), 50);
    }

    #[test]
    fn data_row_renders_a_gap_between_line_command_field_and_data_area() {
        let row = ScreenRow {
            record_index: 0,
            line_number: "000001".into(),
            prefix: String::new(),
            text: "TEXT".into(),
            is_excluded_placeholder: false,
            prefix_selected: true,
            text_selected: false,
            text_cursor_col: None,
        };

        let line = super::data_row_line(&row);
        let text: String = line.spans.iter().map(|span| span.content.as_ref()).collect();

        assert_eq!(text, "000001 TEXT");
    }

    #[test]
    fn typed_line_command_overlays_the_left_six_column_field() {
        let row = ScreenRow {
            record_index: 0,
            line_number: "000001".into(),
            prefix: "D".into(),
            text: "TEXT".into(),
            is_excluded_placeholder: false,
            prefix_selected: false,
            text_selected: false,
            text_cursor_col: None,
        };

        let line = super::data_row_line(&row);
        let text: String = line.spans.iter().map(|span| span.content.as_ref()).collect();

        assert_eq!(text, "D00001 TEXT");
    }

    #[test]
    fn selected_empty_line_command_field_keeps_the_visible_line_number() {
        let row = ScreenRow {
            record_index: 0,
            line_number: "000001".into(),
            prefix: String::new(),
            text: "TEXT".into(),
            is_excluded_placeholder: false,
            prefix_selected: true,
            text_selected: false,
            text_cursor_col: None,
        };

        let line = super::data_row_line(&row);
        let text: String = line.spans.iter().map(|span| span.content.as_ref()).collect();

        assert_eq!(text, "000001 TEXT");
    }

    #[test]
    fn selected_line_command_field_places_cursor_after_typed_command() {
        let row = ScreenRow {
            record_index: 0,
            line_number: "000001".into(),
            prefix: "XX".into(),
            text: "TEXT".into(),
            is_excluded_placeholder: false,
            prefix_selected: true,
            text_selected: false,
            text_cursor_col: None,
        };

        let spans = super::line_command_field_spans(&row);

        assert_eq!(spans.len(), 1);
        assert_eq!(spans[0].content.as_ref(), "XX0001");
        assert_eq!(spans[0].style.fg, Some(super::ispf_black()));
        assert_eq!(spans[0].style.bg, Some(super::ispf_white()));
    }

    #[test]
    fn line_command_area_collects_text_and_backspace_edits_it() {
        let mut app = App::new(EditBuffer::from_text("A\nB\n").unwrap());

        app.handle_action(AppAction::ToggleFocus).unwrap();
        app.handle_action(AppAction::ToggleFocus).unwrap();
        app.handle_key(KeyEvent::from(KeyCode::Char('x'))).unwrap();
        app.handle_key(KeyEvent::from(KeyCode::Char('x'))).unwrap();
        app.handle_action(AppAction::Backspace).unwrap();

        let screen = app.screen_model(80, 24);
        assert_eq!(screen.rows[0].prefix.trim(), "X");
        assert!(screen.rows[0].prefix_selected);
    }

    #[test]
    fn enter_executes_prefix_command_from_active_row() {
        let mut app = App::new(EditBuffer::from_text("A\nB\nC\n").unwrap());

        app.handle_action(AppAction::ToggleFocus).unwrap();
        app.handle_action(AppAction::ToggleFocus).unwrap();
        app.handle_key(KeyEvent::from(KeyCode::Char('d'))).unwrap();
        app.handle_action(AppAction::Execute).unwrap();

        assert_eq!(app.session().buffer().to_text(), "B\nC\n");
        assert_eq!(app.screen_model(80, 24).rows[0].prefix.trim(), "");
    }

    #[test]
    fn counted_insert_adds_blank_lines_before_the_current_line() {
        let mut app = App::new(EditBuffer::from_text("A\nB\nC\n").unwrap());

        app.handle_action(AppAction::ToggleFocus).unwrap();
        app.handle_action(AppAction::ToggleFocus).unwrap();
        app.handle_action(AppAction::CursorDown).unwrap();
        app.handle_key(KeyEvent::from(KeyCode::Char('i'))).unwrap();
        app.handle_key(KeyEvent::from(KeyCode::Char('3'))).unwrap();
        app.handle_action(AppAction::Execute).unwrap();

        assert_eq!(app.session().buffer().to_text(), "A\n\n\n\nB\nC\n");
    }

    #[test]
    fn shift_enter_in_data_area_inserts_a_blank_line_below_the_current_line() {
        let mut app = App::new(EditBuffer::from_text("A\nB\nC\n").unwrap());

        app.handle_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::SHIFT))
            .unwrap();

        assert_eq!(app.session().buffer().to_text(), "A\n\nB\nC\n");
        assert_eq!(app.session().view().cursor_row, 1);
    }

    #[test]
    fn copy_block_markers_stay_visible_until_a_destination_is_entered() {
        let mut app = App::new(EditBuffer::from_text("A\nB\nC\nD\nE\n").unwrap());

        app.handle_action(AppAction::ToggleFocus).unwrap();
        app.handle_action(AppAction::ToggleFocus).unwrap();
        app.handle_key(KeyEvent::from(KeyCode::Char('c'))).unwrap();
        app.handle_key(KeyEvent::from(KeyCode::Char('c'))).unwrap();
        app.handle_action(AppAction::CursorDown).unwrap();
        app.handle_key(KeyEvent::from(KeyCode::Char('c'))).unwrap();
        app.handle_key(KeyEvent::from(KeyCode::Char('c'))).unwrap();
        app.handle_action(AppAction::Execute).unwrap();

        let screen = app.screen_model(80, 24);
        assert_eq!(screen.rows[0].prefix.trim(), "CC");
        assert_eq!(screen.rows[1].prefix.trim(), "CC");
        assert_eq!(screen.message, "Copy block set");
    }

    #[test]
    fn copy_block_completes_when_destination_is_entered_after_the_range() {
        let mut app = App::new(EditBuffer::from_text("A\nB\nC\nD\nE\n").unwrap());

        app.handle_action(AppAction::ToggleFocus).unwrap();
        app.handle_action(AppAction::ToggleFocus).unwrap();
        app.handle_key(KeyEvent::from(KeyCode::Char('c'))).unwrap();
        app.handle_key(KeyEvent::from(KeyCode::Char('c'))).unwrap();
        app.handle_action(AppAction::CursorDown).unwrap();
        app.handle_key(KeyEvent::from(KeyCode::Char('c'))).unwrap();
        app.handle_key(KeyEvent::from(KeyCode::Char('c'))).unwrap();
        app.handle_action(AppAction::Execute).unwrap();
        app.handle_action(AppAction::CursorDown).unwrap();
        app.handle_action(AppAction::CursorDown).unwrap();
        app.handle_action(AppAction::CursorDown).unwrap();
        app.handle_key(KeyEvent::from(KeyCode::Char('a'))).unwrap();
        app.handle_action(AppAction::Execute).unwrap();

        assert_eq!(app.session().buffer().to_text(), "A\nB\nC\nD\nE\nA\nB\n");
        assert_eq!(app.screen_model(80, 24).message, "2 lines copied");
    }

    #[test]
    fn copy_block_completes_when_range_and_destination_are_entered_before_enter() {
        let mut app = App::new(EditBuffer::from_text("A\nB\nC\nD\nE\n").unwrap());

        app.handle_action(AppAction::ToggleFocus).unwrap();
        app.handle_action(AppAction::ToggleFocus).unwrap();
        app.handle_key(KeyEvent::from(KeyCode::Char('c'))).unwrap();
        app.handle_key(KeyEvent::from(KeyCode::Char('c'))).unwrap();
        app.handle_action(AppAction::CursorDown).unwrap();
        app.handle_key(KeyEvent::from(KeyCode::Char('c'))).unwrap();
        app.handle_key(KeyEvent::from(KeyCode::Char('c'))).unwrap();
        app.handle_action(AppAction::CursorDown).unwrap();
        app.handle_action(AppAction::CursorDown).unwrap();
        app.handle_action(AppAction::CursorDown).unwrap();
        app.handle_key(KeyEvent::from(KeyCode::Char('a'))).unwrap();
        app.handle_action(AppAction::Execute).unwrap();

        assert_eq!(app.session().buffer().to_text(), "A\nB\nC\nD\nE\nA\nB\n");
        assert_eq!(app.screen_model(80, 24).message, "2 lines copied");
    }

    #[test]
    fn move_block_completes_when_destination_is_entered_after_the_range() {
        let mut app = App::new(EditBuffer::from_text("A\nB\nC\nD\nE\n").unwrap());

        app.handle_action(AppAction::ToggleFocus).unwrap();
        app.handle_action(AppAction::ToggleFocus).unwrap();
        app.handle_key(KeyEvent::from(KeyCode::Char('m'))).unwrap();
        app.handle_key(KeyEvent::from(KeyCode::Char('m'))).unwrap();
        app.handle_action(AppAction::CursorDown).unwrap();
        app.handle_key(KeyEvent::from(KeyCode::Char('m'))).unwrap();
        app.handle_key(KeyEvent::from(KeyCode::Char('m'))).unwrap();
        app.handle_action(AppAction::Execute).unwrap();
        app.handle_action(AppAction::CursorDown).unwrap();
        app.handle_action(AppAction::CursorDown).unwrap();
        app.handle_action(AppAction::CursorDown).unwrap();
        app.handle_key(KeyEvent::from(KeyCode::Char('b'))).unwrap();
        app.handle_action(AppAction::Execute).unwrap();

        assert_eq!(app.session().buffer().to_text(), "C\nD\nA\nB\nE\n");
        assert_eq!(app.screen_model(80, 24).message, "2 lines moved");
    }

    #[test]
    fn invalid_line_command_sets_a_message_instead_of_exiting() {
        let mut app = App::new(EditBuffer::from_text("A\nB\n").unwrap());

        app.handle_action(AppAction::ToggleFocus).unwrap();
        app.handle_action(AppAction::ToggleFocus).unwrap();
        app.handle_key(KeyEvent::from(KeyCode::Char('z'))).unwrap();
        app.handle_key(KeyEvent::from(KeyCode::Char('z'))).unwrap();
        app.handle_action(AppAction::Execute).unwrap();

        assert!(!app.should_quit());
        assert_eq!(app.screen_model(80, 24).message, "unknown line command: ZZ");
    }

    #[test]
    fn ctrl_j_in_data_area_moves_to_the_next_line_without_inserting() {
        let mut app = App::new(EditBuffer::from_text("A\nB\nC\n").unwrap());

        app.handle_key(KeyEvent::new(
            KeyCode::Char('j'),
            KeyModifiers::CONTROL,
        ))
            .unwrap();

        assert_eq!(app.session().buffer().to_text(), "A\nB\nC\n");
        assert_eq!(app.session().view().cursor_row, 1);
    }

    #[test]
    fn q_types_into_line_command_area_instead_of_quitting() {
        let mut app = App::new(EditBuffer::from_text("A\n").unwrap());

        app.handle_action(AppAction::ToggleFocus).unwrap();
        app.handle_action(AppAction::ToggleFocus).unwrap();
        app.handle_key(KeyEvent::from(KeyCode::Char('q'))).unwrap();

        assert!(!app.should_quit());
        assert_eq!(app.screen_model(80, 24).rows[0].prefix.trim(), "Q");
    }
}
