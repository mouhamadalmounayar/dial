use crate::persistence::{load_snippets, save_snippets};
use crate::view::{Component, ViewManager};
use anyhow::{Context, Result};
use log::error;
use ratatui::{
    DefaultTerminal, Frame,
    crossterm::event::{self, Event, KeyCode, KeyEventKind},
    layout::{Constraint, Direction, Layout, Rect},
    widgets::{Block, Borders, Widget},
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
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
    pub cursor_coordinates: (u16, u16),
    pub current_area: Rect,
    pub should_show_cursor: bool,
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
        let snippet_list = load_snippets().expect("snippet_list should not be empty");
        let app_state = AppState {
            snippet_list,
            selected_index: 0,
            mode: AppMode::Command,
            should_exit: false,
            cursor_coordinates: (0, 0),
            current_area: Rect::default(),
            should_show_cursor: false,
        };

        App {
            app_state,
            view_manager: ViewManager::new(),
        }
    }

    fn switch_mode(&mut self, event: &Event) {
        match event {
            Event::Key(key) => {
                if key.kind == KeyEventKind::Press {
                    match key.code {
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
                    }
                }
            }
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

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<(), anyhow::Error> {
        while self.app_state.should_exit != true {
            terminal
                .draw(|f: &mut Frame| {
                    let inner_area = self.render_outer_block(f);

                    let horizontal_chunks = Layout::new(
                        Direction::Horizontal,
                        vec![Constraint::Percentage(30), Constraint::Percentage(70)],
                    )
                    .split(inner_area);
                    self.view_manager.snippet_list_component.render(
                        horizontal_chunks[0],
                        f,
                        &self.app_state,
                    );
                    self.view_manager.editor_component.render(
                        horizontal_chunks[1],
                        f,
                        &self.app_state,
                    );
                    // update current area
                    match self.app_state.mode {
                        AppMode::Select => {
                            self.app_state.current_area = horizontal_chunks[0];
                        }
                        AppMode::Edit => {
                            self.app_state.current_area = horizontal_chunks[1];
                        }
                        _ => {}
                    }
                    if self.app_state.should_show_cursor {
                        f.set_cursor_position(self.app_state.cursor_coordinates);
                    }
                })
                .with_context(|| "could not draw frame")?;
            let result = event::read();
            match result {
                Ok(event) => match event {
                    Event::Key(key) => match key.code {
                        KeyCode::Esc => {
                            self.app_state.mode = AppMode::Command;
                            self.view_manager
                                .editor_component
                                .sync_buffer_to_state(&mut self.app_state);
                            save_snippets(&self.app_state.snippet_list[..])?;
                        }
                        _ => {
                            if self.app_state.mode == AppMode::Command {
                                self.switch_mode(&event);
                            } else if self.app_state.mode == AppMode::Select {
                                self.view_manager
                                    .snippet_list_component
                                    .handle_event(&event, &mut self.app_state);
                            } else if self.app_state.mode == AppMode::Edit {
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
        Ok(())
    }
}
