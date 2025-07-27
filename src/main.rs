use ratatui::{
    DefaultTerminal, Frame,
    text::Line,
    widgets::{ListState, Widget},
};

fn main() {
    let mut terminal = ratatui::init();
    let mut app = App::new();
    app.run(&mut terminal);
    ratatui::restore();
}

struct Snippet {
    language: String,
    code: String,
    title: String,
}

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
                self.view_manager
                    .snippet_list_component
                    .render(f, &self.app_state);
            });
            match result {
                Ok(_) => {}
                Err(e) => {
                    eprintln!("There was an error while calling draw {})", e)
                }
            }
        }
    }
}

trait Component {
    fn render(&mut self, frame: &mut Frame, state: &AppState);
}

struct SnippetListComponent {
    state: ListState,
}

impl SnippetListComponent {
    fn new() -> Self {
        SnippetListComponent {
            state: ListState::default(),
        }
    }
}

impl Component for SnippetListComponent {
    // placeholder render method
    fn render(&mut self, frame: &mut Frame, state: &AppState) {
        let widget = Line::from("Hello World");
        widget.render(frame.area(), frame.buffer_mut());
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
