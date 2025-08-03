use crate::app::{AppState, Snippet};
use crate::editor::GapBuffer;
use log::info;
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
        let items = state
            .snippet_list
            .iter()
            .map(|snippet| ListItem::from(snippet));
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
        let length = state.snippet_list.len();
        if length == 0 {
            return;
        }
        let index = state.selected_index + 1;
        state.selected_index = index % length;
    }

    fn select_previous(&self, state: &mut AppState) {
        let length = state.snippet_list.len();
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
}

impl Component for EditorComponent {
    fn render(&mut self, area: Rect, frame: &mut Frame, state: &AppState) {
        if state.should_show_cursor {
            info!("Showing cursor at {:?}", state.cursor_coordinates);
            frame.set_cursor_position(state.cursor_coordinates);
        }
        // sync local state with global state by reinitializing the gap_buffer if the selected_index changes.
        if self.selected_index != Some(state.selected_index) {
            let content = state
                .get_content()
                .expect("Unexpected state, a snippet must be selected at all times");
            self.gap_buffer = Some(GapBuffer::from_str(&content[..], 1024));
            self.selected_index = Some(state.selected_index);
        }
        // render the gap buffer with syntax highlighting.
        let gap_buffer = self
            .gap_buffer
            .as_ref()
            .expect("Unexpected state, buffer must not be null at this point");
        let text: String = gap_buffer.buffer.iter().filter(|&&c| c != '\0').collect();
        let language = &state.snippet_list[state.selected_index].language;
        let syntax = self.syntax_set.find_syntax_by_extension(&language).unwrap();
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
    }

    fn handle_event(&mut self, event: &Event, state: &mut AppState) {
        let buffer = self
            .gap_buffer
            .as_mut()
            .expect("Unexpected state, buffer must not be null at this point");
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
                            buffer.insert_char(' ');
                            buffer.insert_char(' ');
                            buffer.insert_char(' ');
                            buffer.insert_char(' ');
                        }
                        _ => {}
                    }
                    // update the cursor coordinates after handling the event
                    let text_before_cursor = &buffer.buffer[..buffer.gap_start];
                    let line_count = text_before_cursor.iter().filter(|&&c| c == '\n').count() + 1;
                    let last_newline = text_before_cursor
                        .iter()
                        .rposition(|&c| c == '\n')
                        .map(|p| p + 1)
                        .unwrap_or(0);
                    let column = buffer.gap_start - last_newline;
                    state.cursor_coordinates = (
                        state.current_area.x + column as u16 + 1,
                        state.current_area.y + line_count as u16,
                    );
                    state.should_show_cursor = true;
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
            syntax_set,
            theme_set,
        }
    }
}

pub struct ViewManager {
    pub snippet_list_component: SnippetListComponent,
    pub editor_component: EditorComponent,
}

impl ViewManager {
    pub fn new() -> Self {
        ViewManager {
            snippet_list_component: SnippetListComponent::new(),
            editor_component: EditorComponent::new(),
        }
    }
}
