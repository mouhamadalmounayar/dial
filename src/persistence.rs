use anyhow::{Context, Result, bail};
use directories::ProjectDirs;
use log::info;
use std::{fs, io::Write, path::PathBuf};

use crate::app::Snippet;

fn get_path() -> Result<PathBuf> {
    let optional_project_dirs = ProjectDirs::from("com", "mouhamadalmounayar", "dial");
    match optional_project_dirs {
        Some(project_dirs) => {
            let data_dir = project_dirs.data_dir();
            fs::create_dir_all(data_dir)?;
            Ok(data_dir.join("snippets.json"))
        }
        None => bail!("could not get the path to the data directory"),
    }
}

pub fn save_snippets(snippets: &[Snippet]) -> Result<()> {
    let path = get_path()?;

    let mut file =
        fs::File::create(&path).with_context(|| format!("could not create file {:?}", &path))?;

    let json_string = serde_json::to_string_pretty(snippets)
        .with_context(|| format!("could not serialize json string"))?;

    file.write_all(json_string.as_bytes())
        .with_context(|| format!("could not write to file {:?}", &path))?;

    info!("writing to file was successful");
    Ok(())
}

pub fn load_snippets() -> Result<Vec<Snippet>> {
    let default_snippets = vec![Snippet {
        language: String::from("txt"),
        title: String::from("Welcome to Dial"),
        code: String::from("Dial is a code snippet manager built with rust and ratatui."),
    }];
    let path = get_path()?;
    if !path.exists() {
        info!("{:?} does not exist", path);
        return Ok(default_snippets);
    }

    let json_data =
        fs::read_to_string(&path).with_context(|| format!("could not read file {:?}", &path))?;

    if json_data.is_empty() {
        return Ok(default_snippets);
    }

    let snippets = serde_json::from_str(&json_data)
        .with_context(|| format!("could not serialize json data"))?;

    Ok(snippets)
}
