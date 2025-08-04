use crate::app::{AppState, Snippet};
use crate::editor::GapBuffer;
use ratatui::crossterm::event::{Event, KeyCode, KeyEventKind};
use ratatui::text::Span;
use ratatui::widgets::Paragraph;
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
        let block = Block::new().borders(Borders::all()).title(" Snippets ");
        let list = List::new(items)
            .block(block)
            .highlight_symbol(" > ")
            .highlight_style(Style::default().light_blue());
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
        let block = Block::default().borders(Borders::ALL).title(" Editor ");
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
                        state.current_area.x + column as u16 + 1,
                        state.current_area.y + line_count as u16,
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
        let block = Block::default().title_top(" Search ").borders(Borders::ALL);
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

pub struct ViewManager {
    pub snippet_list_component: SnippetListComponent,
    pub editor_component: EditorComponent,
    pub search_component: SearchComponent,
}

impl ViewManager {
    pub fn new() -> Self {
        ViewManager {
            snippet_list_component: SnippetListComponent::new(),
            editor_component: EditorComponent::new(),
            search_component: SearchComponent::new(),
        }
    }
}
