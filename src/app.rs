use log::error;
use ratatui::{
    DefaultTerminal, Frame,
    crossterm::event::{self, Event, KeyCode},
    layout::{Constraint, Direction, Layout, Rect},
    widgets::{Block, Borders, Widget},
};

use crate::view::{Component, ViewManager};

pub struct Snippet {
    pub language: String,
    pub code: String,
    pub title: String,
}

#[derive(PartialEq, Debug)]
pub enum AppMode {
    Select,
    Search,
    Edit,
    Command,
}

pub struct AppState {
    pub snippet_list: Vec<Snippet>,
    pub selected_index: usize,
    pub mode: AppMode,
    pub should_exit: bool,
}

impl AppState {
    pub fn get_content(&self) -> Option<String> {
        self.snippet_list
            .get(self.selected_index)
            .map(|snippet| snippet.code.clone())
    }
}

pub struct App {
    pub app_state: AppState,
    pub view_manager: ViewManager,
}

impl App {
    pub fn new() -> Self {
        // I am mocking the snippet list values for now...
        let snippet_list = vec![
            Snippet {
                language: String::from("py"),
                code: String::from("print() \n for i in range () "),
                title: String::from("Print function"),
            },
            Snippet {
                language: String::from("rs"),
                code: String::from("println!()"),
                title: String::from("Print macro"),
            },
        ];
        let app_state = AppState {
            snippet_list,
            selected_index: 0,
            mode: AppMode::Command,
            should_exit: false,
        };

        App {
            app_state,
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
        let help_text = " [q] Quit   │  [s] Select Mode   │  [e] Edit Mode ";
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

    pub fn run(&mut self, terminal: &mut DefaultTerminal) {
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
                    error!("There was an error while calling draw {})", e);
                }
            }
            let result = event::read();
            match result {
                Ok(event) => match event {
                    Event::Key(key) => match key.code {
                        KeyCode::Esc => {
                            self.app_state.mode = AppMode::Command;
                        }
                        _ => {
                            if self.app_state.mode == AppMode::Command {
                                self.switch_mode(&event);
                            }
                            if self.app_state.mode == AppMode::Select {
                                self.view_manager
                                    .snippet_list_component
                                    .handle_event(&event, &mut self.app_state);
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
                    error!("There was an error trying to read events");
                }
            }
        }
    }
}