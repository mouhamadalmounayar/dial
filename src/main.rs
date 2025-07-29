use log::error;
use simplelog::{Config, WriteLogger};
use std::fs::File;
use tui_textarea::{Input, TextArea};

use ratatui::crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{
    DefaultTerminal, Frame,
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Style, Stylize},
    text::Line,
    widgets::{Block, Borders, List, ListItem, ListState, StatefulWidget, Widget},
};

fn main() {
    setup_logger();
    let mut terminal = ratatui::init();
    let mut app = App::new();
    app.run(&mut terminal);
    ratatui::restore();
}

fn setup_logger() {
    let log_file = File::create("dial.log").unwrap();
    let result = WriteLogger::init(log::LevelFilter::Debug, Config::default(), log_file);
    match result {
        Ok(()) => {}
        Err(_) => {
            error!("There was an error initializing the logger")
        }
    }
}

struct Snippet {
    language: String,
    code: String,
    title: String,
}

#[derive(PartialEq, Debug)]
enum AppMode {
    Select,
    Search,
    Edit,
    Command,
}

struct AppState {
    snippet_list: Vec<Snippet>,
    selected_index: usize,
    mode: AppMode,
    should_exit: bool,
}

struct App {
    app_state: AppState,
    view_manager: ViewManager,
}

impl App {
    fn new() -> Self {
        // I am mocking the snippet list values for now...
        let snippet_list = vec![
            Snippet {
                language: String::from("python"),
                code: String::from("print()"),
                title: String::from("Print function"),
            },
            Snippet {
                language: String::from("rust"),
                code: String::from("println!()"),
                title: String::from("Print macro"),
            },
        ];
        App {
            app_state: AppState {
                snippet_list,
                selected_index: 0,
                mode: AppMode::Command,
                should_exit: false,
            },
            view_manager: ViewManager::new(),
        }
    }

    fn switch_mode(&mut self, event: &Event) {
        match event {
            Event::Key(key) => match key.code {
                KeyCode::Char('q') => {
                    self.app_state.should_exit = true;
                }
                KeyCode::Char('e') => {
                    self.app_state.mode = AppMode::Edit;
                }
                KeyCode::Char('s') => {
                    self.app_state.mode = AppMode::Select;
                }
                _ => {}
            },
            _ => {}
        }
    }

    fn render_outer_block(&self, f: &mut Frame) -> Rect {
        let mode_text = format!(" Mode: {:?} ", self.app_state.mode);
        let help_text = " 🅆 [q] Quit   │   🛈 [s] Select Mode   │   ✎ [e] Edit Mode ";
        let block = Block::new()
            .borders(Borders::ALL)
            .title(" Dial ")
            .title_alignment(ratatui::layout::Alignment::Center)
            .title_bottom(mode_text)
            .title_bottom(help_text);
        let inner_area = block.inner(f.area());
        block.render(f.area(), f.buffer_mut());
        inner_area
    }

    fn run(&mut self, terminal: &mut DefaultTerminal) {
        while self.app_state.should_exit != true {
            let result = terminal.draw(|f: &mut Frame| {
                let inner_area = self.render_outer_block(f);
                let horizontal_chunks = Layout::new(
                    Direction::Horizontal,
                    vec![Constraint::Percentage(30), Constraint::Percentage(70)],
                )
                .split(inner_area);
                self.view_manager.snippet_list_component.render(
                    horizontal_chunks[0],
                    f.buffer_mut(),
                    &self.app_state,
                );
                self.view_manager.editor_component.render(
                    horizontal_chunks[1],
                    f.buffer_mut(),
                    &self.app_state,
                );
            });
            match result {
                Ok(_) => {}
                Err(e) => {
                    error!("There was an error while calling draw {})", e)
                }
            }

            let result = event::read();
            match result {
                Ok(event) => match event {
                    Event::Key(key) => match key.code {
                        KeyCode::Esc => self.app_state.mode = AppMode::Command,
                        _ => {
                            if self.app_state.mode == AppMode::Command {
                                self.switch_mode(&event);
                            }
                            if self.app_state.mode == AppMode::Select {
                                self.view_manager
                                    .snippet_list_component
                                    .handle_event(&event, &mut self.app_state)
                            }
                            if self.app_state.mode == AppMode::Edit {
                                self.view_manager
                                    .editor_component
                                    .handle_event(&event, &mut self.app_state);
                            }
                        }
                    },
                    _ => {}
                },
                Err(_) => {
                    error!("There was an error trying to read events")
                }
            }
        }
    }
}

trait Component {
    fn render(&mut self, area: Rect, buff: &mut Buffer, state: &AppState);
    fn handle_event(&mut self, event: &Event, state: &mut AppState);
}

struct SnippetListComponent {
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
        let block = Block::new()
            .borders(Borders::all())
            .title(" Snippets ")
            .title_alignment(ratatui::layout::Alignment::Center);
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

struct EditorComponent {
    text_area: TextArea<'static>,
}

impl Component for EditorComponent {
    fn render(&mut self, area: Rect, buff: &mut Buffer, state: &AppState) {
        let block = Block::default().borders(Borders::ALL).title(" Editor ");
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
        }
    }
}

struct ViewManager {
    snippet_list_component: SnippetListComponent,
    editor_component: EditorComponent,
}

impl ViewManager {
    fn new() -> Self {
        ViewManager {
            snippet_list_component: SnippetListComponent::new(),
            editor_component: EditorComponent::new(),
        }
    }
}
