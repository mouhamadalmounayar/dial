use std::u16;

use crate::app::{AppMode, AppState, Snippet};
use crate::editor::GapBuffer;
use ratatui::crossterm::event::{Event, KeyCode, KeyEventKind};
use ratatui::layout::{Constraint, Layout};
use ratatui::text::Span;
use ratatui::widgets::{BorderType, Padding, Paragraph};
use ratatui::{
    Frame,
    layout::Rect,
    style::{Style, Stylize},
    text::Line,
    widgets::{Block, Borders, List, ListItem, ListState},
};
use syntect::{
    easy::HighlightLines, highlighting::ThemeSet, parsing::SyntaxSet, util::LinesWithEndings,
};

use syntect_tui::into_span;

const EDITOR_BUFFER_SIZE: usize = 1024;
const SEARCH_BUFFER_SIZE: usize = 256;
const TAB_SIZE: usize = 4;
const PADDING_SIZE: u16 = 1;

pub trait Component {
    fn render(&mut self, area: Rect, frame: &mut Frame, state: &AppState);
    fn handle_event(&mut self, event: &Event, state: &mut AppState);
}

pub struct SnippetListComponent {
    local_state: ListState,
}

impl SnippetListComponent {
    fn new() -> Self {
        SnippetListComponent {
            local_state: ListState::default(),
        }
    }
}

impl Component for SnippetListComponent {
    fn render(&mut self, area: Rect, frame: &mut Frame, state: &AppState) {
        let index = state.selected_index;
        self.local_state.select(Some(index));
        let items: Vec<ListItem> = state
            .filtered_snippets()
            .iter()
            .map(|(_, snippet)| ListItem::from(*snippet))
            .collect();
        let block = Block::new()
            .borders(Borders::all())
            .title(" 󰅩 Snippets ".blue())
            .title_bottom(" [a]: Add Snippet  ")
            .title_alignment(ratatui::layout::Alignment::Center);
        let list = List::new(items)
            .block(block)
            .highlight_style(Style::default().bg(ratatui::style::Color::Black).white());
        frame.render_stateful_widget(list, area, &mut self.local_state);
    }

    fn handle_event(&mut self, event: &Event, state: &mut AppState) {
        match event {
            Event::Key(key) => match key.code {
                KeyCode::Char('j') => {
                    if key.kind == KeyEventKind::Press {
                        self.select_next(state);
                    }
                }
                KeyCode::Char('k') => {
                    if key.kind == KeyEventKind::Press {
                        self.select_previous(state);
                    }
                }
                KeyCode::Char('a') => {
                    if key.kind == KeyEventKind::Press {
                        state.mode = AppMode::Popup
                    }
                }
                _ => {}
            },
            _ => {}
        }
    }
}

impl SnippetListComponent {
    fn select_next(&self, state: &mut AppState) {
        let length = state.filtered_snippets().len();
        if length == 0 {
            return;
        }
        let index = state.selected_index + 1;
        state.selected_index = index % length;
    }

    fn select_previous(&self, state: &mut AppState) {
        let length = state.filtered_snippets().len();
        if length == 0 {
            return;
        }
        if state.selected_index == 0 {
            state.selected_index = length - 1;
        } else {
            state.selected_index -= 1;
        }
    }
}

impl From<&Snippet> for ListItem<'_> {
    fn from(value: &Snippet) -> Self {
        let title = Line::from(value.title.clone()).bold();
        let language = Line::from(value.language.clone()).italic();
        ListItem::new(vec![title, language, Line::from("")])
    }
}

pub struct EditorComponent {
    pub gap_buffer: Option<GapBuffer>,
    selected_index: Option<usize>,
    pub syntax_set: SyntaxSet,
    pub theme_set: ThemeSet,
    pub cursor_coordinates: (u16, u16),
}

impl Component for EditorComponent {
    fn render(&mut self, area: Rect, frame: &mut Frame, state: &AppState) {
        // sync local state with global state by reinitializing the gap_buffer if the selected_index changes.
        if self.selected_index != state.get_selected_snippet_index() {
            let content = state
                .get_content()
                .expect("unexpected state a snippet must be selected at all times");
            self.gap_buffer = Some(GapBuffer::from_str(&content[..], EDITOR_BUFFER_SIZE));
            self.selected_index = state.get_selected_snippet_index();
        }
        // render the gap buffer with syntax highlighting.
        let gap_buffer = self
            .gap_buffer
            .as_ref()
            .expect("unexpected state buffer must not be null at this point");
        let text: String = gap_buffer.to_string();
        let language = state
            .get_current_snippet()
            .map(|snippet| &snippet.language)
            .unwrap();
        let syntax = self
            .syntax_set
            .find_syntax_by_extension(&language)
            .or_else(|| self.syntax_set.find_syntax_by_extension("txt"))
            .unwrap();
        let mut highlighter =
            HighlightLines::new(syntax, &self.theme_set.themes["base16-eighties.dark"]);
        let buffer_widget: Vec<Line> = LinesWithEndings::from(&text)
            .map(|line| {
                let spans: Vec<Span> = highlighter
                    .highlight_line(line, &self.syntax_set)
                    .unwrap()
                    .into_iter()
                    .filter_map(|segment| into_span(segment).ok())
                    // override underline color style and background
                    .map(|span| {
                        let style = span
                            .style
                            .underline_color(ratatui::style::Color::Reset)
                            .bg(ratatui::style::Color::Reset);
                        Span::styled(span.content, style)
                    })
                    .collect();
                Line::from(spans)
            })
            .collect();
        let block = Block::default()
            .borders(Borders::ALL)
            .title("  Editor ".blue())
            .padding(Padding::uniform(PADDING_SIZE));
        let paragraph = Paragraph::new(buffer_widget).block(block);
        frame.render_widget(paragraph, area);
        if state.focused_editor {
            frame.set_cursor_position(self.cursor_coordinates);
        }
    }

    fn handle_event(&mut self, event: &Event, state: &mut AppState) {
        let buffer = self
            .gap_buffer
            .as_mut()
            .expect("unexpected state buffer must not be null at this point");
        match event {
            Event::Key(key) => {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char(c) => {
                            buffer.insert_char(c);
                        }
                        KeyCode::Enter => {
                            buffer.insert_char('\n');
                        }
                        KeyCode::Backspace => {
                            buffer.delete_char();
                        }
                        KeyCode::Left => {
                            buffer.move_gap(buffer.gap_start.saturating_sub(1));
                        }
                        KeyCode::Right => {
                            buffer.move_gap(buffer.gap_start + 1);
                        }
                        KeyCode::Tab => {
                            for _ in 0..TAB_SIZE {
                                buffer.insert_char(' ');
                            }
                        }
                        _ => {}
                    }
                    let text_before_cursor = &buffer.buffer[..buffer.gap_start];
                    let line_count = text_before_cursor.iter().filter(|&&c| c == '\n').count() + 1;
                    let last_newline = text_before_cursor
                        .iter()
                        .rposition(|&c| c == '\n')
                        .map(|p| p + 1)
                        .unwrap_or(0);
                    let column = buffer.gap_start - last_newline;
                    self.cursor_coordinates = (
                        state.current_area.x + PADDING_SIZE + column as u16 + 1,
                        state.current_area.y + PADDING_SIZE + line_count as u16,
                    );
                    state.focus_editor();
                }
            }
            _ => {}
        }
    }
}

impl EditorComponent {
    fn new() -> Self {
        let syntax_set = SyntaxSet::load_defaults_nonewlines();
        let theme_set = ThemeSet::load_defaults();
        EditorComponent {
            gap_buffer: None,
            selected_index: None,
            cursor_coordinates: (0, 0),
            syntax_set,
            theme_set,
        }
    }

    pub fn sync_buffer_to_state(&mut self, state: &mut AppState) {
        let gap_buffer = self
            .gap_buffer
            .as_ref()
            .expect("buffer should be defined at this point");
        let new_code: String = gap_buffer.to_string();

        // Update the actual snippet in the original list
        if let Some(actual_index) = state.get_selected_snippet_index() {
            if let Some(snippet) = state.snippet_list.get_mut(actual_index) {
                snippet.code = new_code;
            }
        }
    }
}

pub struct SearchComponent {
    gap_buffer: GapBuffer,
    coordinates: (u16, u16),
}

impl SearchComponent {
    fn new() -> Self {
        SearchComponent {
            gap_buffer: GapBuffer::from_str("", SEARCH_BUFFER_SIZE),
            coordinates: (0, 0),
        }
    }
}

impl Component for SearchComponent {
    fn render(&mut self, area: Rect, frame: &mut Frame, state: &AppState) {
        let block = Block::default()
            .title_top("  Search ".blue())
            .borders(Borders::ALL);
        let text: String = self.gap_buffer.to_string();
        let line = Paragraph::new(text).block(block);
        frame.render_widget(line, area);
        if state.focused_search {
            frame.set_cursor_position(self.coordinates);
        }
    }

    fn handle_event(&mut self, event: &Event, state: &mut AppState) {
        match event {
            Event::Key(key) => {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char(c) => {
                            self.gap_buffer.insert_char(c);
                        }
                        KeyCode::Backspace => {
                            self.gap_buffer.delete_char();
                        }
                        KeyCode::Enter => {
                            state.search_query = self.gap_buffer.to_string();
                            state.selected_index = 0;
                        }
                        _ => {}
                    }
                    // set cursor position
                    let x: u16 = state.current_area.x + self.gap_buffer.gap_start as u16 + 1;
                    let y: u16 = state.current_area.y + 1;
                    self.coordinates = (x, y);
                    // focused_search
                    state.focus_search();
                }
            }
            _ => {}
        }
    }
}

#[derive(PartialEq)]
enum Input {
    Title,
    Language,
}

pub struct AddSnippetPopupComponent {
    title_input: GapBuffer,
    language_input: GapBuffer,
    cursor_position: (u16, u16),
    focused_input: Input,
    current_area: Rect,
    should_show_cursor: bool,
    title_area: Rect,
    language_area: Rect,
}

impl Component for AddSnippetPopupComponent {
    fn render(&mut self, _area: Rect, frame: &mut Frame, _state: &AppState) {
        let width = frame.area().width / 3;
        let height = frame.area().height / 3;
        let area = Rect::new(
            frame.area().width / 2 - width / 2,
            frame.area().height / 2 - height / 2,
            width,
            height,
        );
        let layout = Layout::new(
            ratatui::layout::Direction::Vertical,
            vec![
                Constraint::Percentage(35),
                Constraint::Percentage(35),
                Constraint::Percentage(30),
            ],
        )
        // position in the center of the screen
        .split(area);

        frame.render_widget(ratatui::widgets::Clear, area);

        // block definitions
        let title_block = Block::default()
            .title(" Snippet Title ")
            .title_alignment(ratatui::layout::Alignment::Left)
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded);

        let language_block = Block::default()
            .title(" Language Extension ")
            .title_alignment(ratatui::layout::Alignment::Left)
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded);

        let title = Paragraph::new(self.title_input.to_string()).block(title_block);
        let language = Paragraph::new(self.language_input.to_string()).block(language_block);

        let help_text = Block::default()
            .title("[A]: Add | [Esc]: Close")
            .title_alignment(ratatui::layout::Alignment::Center);

        frame.render_widget(language, layout[0]);
        frame.render_widget(title, layout[1]);
        frame.render_widget(help_text, layout[2]);

        //initialize areas
        self.title_area = layout[1];
        self.language_area = layout[0];

        match self.focused_input {
            Input::Title => {
                self.current_area = layout[1];
            }
            Input::Language => {
                self.current_area = layout[0];
            }
        }

        if self.should_show_cursor {
            assert!(self.cursor_position != (0, 0));
            frame.set_cursor_position(self.cursor_position);
        }
    }

    fn handle_event(&mut self, event: &Event, state: &mut AppState) {
        match event {
            Event::Key(key) => {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('A') => {
                            let snippet = Snippet {
                                title: self.title_input.to_string(),
                                language: self.language_input.to_string(),
                                code: String::new(),
                            };
                            state.snippet_list.push(snippet);
                            state.mode = AppMode::Command;
                        }
                        KeyCode::Char(c) => {
                            self.insert_char(c);
                            self.should_show_cursor = true;
                        }
                        KeyCode::Backspace => {
                            self.delete_char();
                            self.should_show_cursor = true;
                        }
                        KeyCode::Enter => self.toggle_focused_input(),
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }
}

impl AddSnippetPopupComponent {
    fn new() -> Self {
        AddSnippetPopupComponent {
            title_input: GapBuffer::from_str("", SEARCH_BUFFER_SIZE),
            language_input: GapBuffer::from_str("", SEARCH_BUFFER_SIZE),
            cursor_position: (0, 0),
            focused_input: Input::Language,
            current_area: Rect::default(),
            should_show_cursor: false,
            title_area: Rect::default(),
            language_area: Rect::default(),
        }
    }

    fn active_buffer(&mut self) -> &mut GapBuffer {
        match self.focused_input {
            Input::Title => &mut self.title_input,
            Input::Language => &mut self.language_input,
        }
    }

    fn insert_char(&mut self, c: char) {
        let buffer = self.active_buffer();
        buffer.insert_char(c);
        let gap_start = buffer.gap_start;
        self.update_cursor_position(gap_start as u16);
    }

    fn delete_char(&mut self) {
        let buffer = self.active_buffer();
        buffer.delete_char();
        let gap_start = buffer.gap_start;
        self.update_cursor_position(gap_start as u16);
    }

    fn update_cursor_position(&mut self, gap_start: u16) {
        let x = self.current_area.x + gap_start as u16 + 1;
        let y = self.current_area.y + 1;
        self.cursor_position = (x, y);
    }

    fn toggle_focused_input(&mut self) {
        if self.focused_input == Input::Title {
            self.focused_input = Input::Language;
            self.current_area = self.language_area;
            self.update_cursor_position(self.language_input.gap_start as u16);
        } else {
            self.focused_input = Input::Title;
            self.current_area = self.title_area;
            self.update_cursor_position(self.title_input.gap_start as u16);
        }
    }
}

pub struct ViewManager {
    pub snippet_list_component: SnippetListComponent,
    pub editor_component: EditorComponent,
    pub search_component: SearchComponent,
    pub add_snippet_popup_component: AddSnippetPopupComponent,
}

impl ViewManager {
    pub fn new() -> Self {
        ViewManager {
            snippet_list_component: SnippetListComponent::new(),
            editor_component: EditorComponent::new(),
            search_component: SearchComponent::new(),
            add_snippet_popup_component: AddSnippetPopupComponent::new(),
        }
    }
}
