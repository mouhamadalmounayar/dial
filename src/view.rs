use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Style, Stylize},
    text::Line,
    widgets::{Block, Borders, List, ListItem, ListState, StatefulWidget, Widget},
};
use ratatui::crossterm::event::{Event, KeyCode, KeyEventKind};
use tui_textarea::{Input, TextArea};
use crate::app::{AppState, Snippet};

pub trait Component {
    fn render(&mut self, area: Rect, buff: &mut Buffer, state: &AppState);
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
    fn render(&mut self, area: Rect, buff: &mut Buffer, state: &AppState) {
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
        StatefulWidget::render(list, area, buff, &mut self.local_state);
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
    text_area: TextArea<'static>,
    selected_index: Option<usize>,
}

impl Component for EditorComponent {
    fn render(&mut self, area: Rect, buff: &mut Buffer, state: &AppState) {
        let block = Block::default().borders(Borders::ALL).title(" Editor ");
        let content = &state.snippet_list[state.selected_index].code;
        if self.selected_index != Some(state.selected_index) {
            self.text_area = TextArea::default();
            self.text_area.insert_str(content);
            self.selected_index = Some(state.selected_index);
        }
        self.text_area.set_block(block);
        self.text_area
            .set_line_number_style(Style::default().fg(ratatui::style::Color::LightBlue));
        self.text_area.render(area, buff);
    }

    fn handle_event(&mut self, event: &Event, _state: &mut AppState) {
        let owned_event = event.clone();
        let input: Input = owned_event.into();
        self.text_area.input(input);
    }
}

impl EditorComponent {
    fn new() -> Self {
        EditorComponent {
            text_area: TextArea::default(),
            selected_index: None,
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
