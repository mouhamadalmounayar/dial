use log::error;
use simplelog::{Config, WriteLogger};
use std::fs::File;

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{
    DefaultTerminal, Frame,
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Style, Stylize},
    text::Line,
    widgets::{Block, Borders, List, ListItem, ListState, StatefulWidget},
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

#[derive(PartialEq)]
enum AppMode {
    Select,
    Search,
    Edit,
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
                mode: AppMode::Select,
                should_exit: false,
            },
            view_manager: ViewManager::new(),
        }
    }

    fn run(&mut self, terminal: &mut DefaultTerminal) {
        while self.app_state.should_exit != true {
            let result = terminal.draw(|f: &mut Frame| {
                let chunks = Layout::new(
                    Direction::Horizontal,
                    vec![Constraint::Percentage(30), Constraint::Percentage(70)],
                )
                .split(f.area());
                self.view_manager.snippet_list_component.render(
                    chunks[0],
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
                        KeyCode::Esc => {
                            self.app_state.should_exit = true;
                        }
                        _ => {
                            if self.app_state.mode == AppMode::Select {
                                self.view_manager
                                    .snippet_list_component
                                    .handle_event(&event, &mut self.app_state)
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
            .highlight_style(Style::default().green());
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

struct ViewManager {
    snippet_list_component: SnippetListComponent,
}

impl ViewManager {
    fn new() -> Self {
        ViewManager {
            snippet_list_component: SnippetListComponent::new(),
        }
    }
}
