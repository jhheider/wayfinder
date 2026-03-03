use std::path::PathBuf;

use anyhow::{Context, Result};

use crate::build::choices::BuildChoices;

/// Directory for saved characters.
pub fn characters_dir() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("waybuilder")
        .join("characters")
}

/// Save build choices to JSON file. Returns the path written.
pub fn save_choices(choices: &BuildChoices) -> Result<PathBuf> {
    let dir = characters_dir();
    std::fs::create_dir_all(&dir).context("Failed to create characters directory")?;

    let filename = sanitize_filename(&choices.name);
    let path = dir.join(format!("{filename}.json"));
    let json = serde_json::to_string_pretty(choices).context("Failed to serialize choices")?;
    std::fs::write(&path, json).context("Failed to write character file")?;
    Ok(path)
}

/// Load build choices from JSON file.
pub fn load_choices(path: &std::path::Path) -> Result<BuildChoices> {
    let json = std::fs::read_to_string(path).context("Failed to read character file")?;
    serde_json::from_str(&json).context("Failed to parse character file")
}

/// List saved character files.
pub fn list_characters() -> Result<Vec<PathBuf>> {
    let dir = characters_dir();
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut files: Vec<PathBuf> = std::fs::read_dir(&dir)?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|e| e == "json"))
        .collect();
    files.sort();
    Ok(files)
}

fn sanitize_filename(name: &str) -> String {
    let s: String = name
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect();
    if s.is_empty() { "unnamed".into() } else { s }
}
