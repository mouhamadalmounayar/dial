use crate::persistence::{load_snippets, save_snippets};
use crate::view::{Component, ViewManager};
use anyhow::{Context, Result};
use log::error;
use ratatui::crossterm::style::Color;
use ratatui::style::Stylize;
use ratatui::text::Span;
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
    Popup,
}

pub struct AppState {
    pub snippet_list: Vec<Snippet>,
    pub selected_index: usize,
    pub mode: AppMode,
    pub should_exit: bool,
    pub current_area: Rect,
    pub focused_editor: bool,
    pub focused_search: bool,
    pub search_query: String,
}

impl AppState {
    pub fn get_content(&self) -> Option<String> {
        if let Some(actual_index) = self.get_selected_snippet_index() {
            self.snippet_list
                .get(actual_index)
                .map(|snippet| snippet.code.clone())
        } else {
            None
        }
    }

    pub fn filtered_snippets(&self) -> Vec<(usize, &Snippet)> {
        self.snippet_list
            .iter()
            .enumerate()
            .filter(|(_, snippet)| {
                snippet
                    .title
                    .to_lowercase()
                    .contains(&self.search_query.to_lowercase())
            })
            .collect()
    }

    pub fn get_selected_snippet_index(&self) -> Option<usize> {
        self.filtered_snippets()
            .get(self.selected_index)
            .map(|(i, _)| *i)
    }

    pub fn get_current_snippet(&self) -> Option<&Snippet> {
        if let Some(actual_index) = self.get_selected_snippet_index() {
            self.snippet_list.get(actual_index)
        } else {
            None
        }
    }

    pub fn focus_editor(&mut self) {
        self.focused_editor = true;
    }

    pub fn focus_search(&mut self) {
        self.focused_search = true;
    }

    pub fn blur_search(&mut self) {
        self.focused_search = false;
    }

    pub fn blur_editor(&mut self) {
        self.focused_editor = false;
    }

    pub fn blur(&mut self) {
        self.blur_search();
        self.blur_editor();
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
            snippet_list: snippet_list.clone(),
            search_query: String::new(),
            selected_index: 0,
            mode: AppMode::Command,
            should_exit: false,
            current_area: Rect::default(),
            focused_editor: false,
            focused_search: false,
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
                        KeyCode::Char('/') => self.app_state.mode = AppMode::Search,
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }

    fn render_outer_block(&self, f: &mut Frame) -> Rect {
        let mode_text = format!(" Mode: {:?} ", self.app_state.mode);
        let help_text = " 󰈆 [q] Quit   │   [s] Select Mode   │  [e] Edit Mode  |   [/] Search ";
        let block = Block::new()
            .borders(Borders::ALL)
            .border_type(ratatui::widgets::BorderType::Rounded)
            .title(" Dial ")
            .bold()
            .title_alignment(ratatui::layout::Alignment::Center)
            .title_bottom(mode_text.bg(Color::DarkBlue).black())
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
                    let vertical_chunks = Layout::new(
                        Direction::Vertical,
                        vec![Constraint::Percentage(10), Constraint::Percentage(90)],
                    )
                    .split(horizontal_chunks[0]);
                    self.view_manager.search_component.render(
                        vertical_chunks[0],
                        f,
                        &self.app_state,
                    );
                    self.view_manager.snippet_list_component.render(
                        vertical_chunks[1],
                        f,
                        &self.app_state,
                    );
                    self.view_manager.editor_component.render(
                        horizontal_chunks[1],
                        f,
                        &self.app_state,
                    );
                    // only render popup in popup mode
                    if self.app_state.mode == AppMode::Popup {
                        self.view_manager.add_snippet_popup_component.render(
                            f.area(),
                            f,
                            &self.app_state,
                        )
                    }
                    // update current area
                    match self.app_state.mode {
                        AppMode::Select => {
                            self.app_state.current_area = horizontal_chunks[0];
                        }
                        AppMode::Edit => {
                            self.app_state.current_area = horizontal_chunks[1];
                        }
                        AppMode::Search => {
                            self.app_state.current_area = vertical_chunks[0];
                        }
                        _ => {}
                    }
                })
                .with_context(|| "could not draw frame")?;
            let result = event::read();
            match result {
                Ok(event) => match event {
                    Event::Key(key) => match key.code {
                        KeyCode::Esc => {
                            // on command mode, unfocus and save
                            self.app_state.mode = AppMode::Command;
                            self.app_state.blur();
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
                            } else if self.app_state.mode == AppMode::Search {
                                self.view_manager
                                    .search_component
                                    .handle_event(&event, &mut self.app_state);
                            } else if self.app_state.mode == AppMode::Popup {
                                self.view_manager
                                    .add_snippet_popup_component
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
