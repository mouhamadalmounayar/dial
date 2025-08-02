mod app;
mod editor;
mod view;
use log::error;
use simplelog::{Config, WriteLogger};
use std::fs::File;

use crate::app::App;
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
