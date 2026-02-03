//! Character save/load persistence

use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};

use super::CharacterData;

/// Get the characters directory path
fn characters_dir() -> Result<PathBuf> {
    let data_dir = dirs::data_local_dir()
        .context("Could not determine local data directory")?
        .join("infinite")
        .join("characters");

    // Ensure directory exists
    fs::create_dir_all(&data_dir).context("Failed to create characters directory")?;

    Ok(data_dir)
}

/// Save a character to disk
pub fn save_character(character: &CharacterData) -> Result<PathBuf> {
    let dir = characters_dir()?;

    // Sanitize filename (remove invalid characters)
    let safe_name: String = character
        .name
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '_' || *c == '-')
        .take(64)
        .collect();

    let filename = if safe_name.is_empty() {
        character.id.clone()
    } else {
        format!("{}_{}", safe_name, &character.id[..8])
    };

    let path = dir.join(format!("{}.json", filename));

    let json = serde_json::to_string_pretty(character).context("Failed to serialize character")?;

    fs::write(&path, json).context("Failed to write character file")?;

    tracing::info!("Saved character '{}' to {:?}", character.name, path);

    Ok(path)
}

/// Load a character from disk by filename (without extension)
pub fn load_character(filename: &str) -> Result<CharacterData> {
    let dir = characters_dir()?;
    let path = dir.join(format!("{}.json", filename));

    let json = fs::read_to_string(&path).context("Failed to read character file")?;

    let character: CharacterData =
        serde_json::from_str(&json).context("Failed to parse character file")?;

    tracing::info!("Loaded character '{}' from {:?}", character.name, path);

    Ok(character)
}

/// Load a character by ID
pub fn load_character_by_id(id: &str) -> Result<CharacterData> {
    let characters = list_characters()?;

    for (filename, character) in characters {
        if character.id == id {
            return load_character(&filename);
        }
    }

    anyhow::bail!("Character with ID '{}' not found", id)
}

/// List all saved characters
pub fn list_characters() -> Result<Vec<(String, CharacterData)>> {
    let dir = characters_dir()?;

    let mut characters = Vec::new();

    for entry in fs::read_dir(&dir).context("Failed to read characters directory")? {
        let entry = entry.context("Failed to read directory entry")?;
        let path = entry.path();

        if path.extension().map_or(false, |ext| ext == "json") {
            if let Ok(json) = fs::read_to_string(&path) {
                if let Ok(character) = serde_json::from_str::<CharacterData>(&json) {
                    let filename = path
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("unknown")
                        .to_string();
                    characters.push((filename, character));
                }
            }
        }
    }

    // Sort by creation date (newest first)
    characters.sort_by(|a, b| b.1.created_at.cmp(&a.1.created_at));

    Ok(characters)
}

/// Delete a character by filename
pub fn delete_character(filename: &str) -> Result<()> {
    let dir = characters_dir()?;
    let path = dir.join(format!("{}.json", filename));

    if path.exists() {
        fs::remove_file(&path).context("Failed to delete character file")?;
        tracing::info!("Deleted character file {:?}", path);
    }

    Ok(())
}

/// Check if any characters exist
pub fn has_characters() -> bool {
    list_characters().map(|c| !c.is_empty()).unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::character::Archetype;

    #[test]
    fn test_save_and_load() {
        let character = CharacterData::new("TestCharacter".to_string(), Archetype::Vanguard);
        let original_id = character.id.clone();

        // Save
        let path = save_character(&character).expect("Failed to save");
        assert!(path.exists());

        // Get filename from path
        let filename = path.file_stem().unwrap().to_str().unwrap();

        // Load
        let loaded = load_character(filename).expect("Failed to load");
        assert_eq!(loaded.id, original_id);
        assert_eq!(loaded.name, "TestCharacter");
        assert_eq!(loaded.archetype, Archetype::Vanguard);

        // Cleanup
        delete_character(filename).expect("Failed to delete");
        assert!(!path.exists());
    }
}
