//! Save/load system with named save slots, quicksave, and auto-save
//!
//! Persists player position, rotation, era, time of day, collected items,
//! and interaction states to JSON files.

use anyhow::{Context, Result};
use infinite_game::InteractionSaveData;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Top-level save data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveData {
    /// Save format version (for future migration)
    pub version: u32,
    /// Player state
    pub player: PlayerSaveData,
    /// World state
    pub world: WorldSaveData,
    /// Human-readable timestamp
    pub timestamp: String,
    /// Name of the save slot (empty for quicksave/autosave)
    #[serde(default)]
    pub slot_name: String,
    /// Items the player has collected
    #[serde(default)]
    pub collected_items: Vec<String>,
    /// Total play time in seconds
    #[serde(default)]
    pub play_time_seconds: f64,
    /// Interaction world state (doors, levers, etc.)
    #[serde(default)]
    pub interactions: InteractionSaveData,
}

/// Saved player state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerSaveData {
    /// Player position [x, y, z]
    pub position: [f32; 3],
    /// Camera yaw in radians
    pub rotation_yaw: f32,
    /// Camera pitch in radians
    pub rotation_pitch: f32,
    /// Character name
    pub character_name: String,
}

/// Saved world state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldSaveData {
    /// Active era index in the timeline
    pub era_index: usize,
    /// Time of day in hours (0.0 - 24.0)
    pub time_of_day: f32,
}

/// Summary info for a save slot (for listing in UI)
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct SaveSlotInfo {
    /// Filename (without extension)
    pub filename: String,
    /// Display name of the slot
    pub slot_name: String,
    /// Timestamp when saved
    pub timestamp: String,
    /// Character name
    pub character_name: String,
    /// Era index
    pub era_index: usize,
    /// Play time in seconds
    pub play_time_seconds: f64,
}

/// Get the save directory path, creating it if it doesn't exist
fn save_dir() -> Result<PathBuf> {
    let dir = dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("infinite")
        .join("saves");
    fs::create_dir_all(&dir).context("Failed to create save directory")?;
    Ok(dir)
}

/// Get the quicksave file path
fn quicksave_path() -> Result<PathBuf> {
    Ok(save_dir()?.join("quicksave.json"))
}

/// Get the autosave file path
fn autosave_path() -> Result<PathBuf> {
    Ok(save_dir()?.join("autosave.json"))
}

/// Get the path for a named save slot
fn slot_path(filename: &str) -> Result<PathBuf> {
    Ok(save_dir()?.join(format!("{}.json", filename)))
}

/// Sanitize a slot name into a valid filename
fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
        .collect::<String>()
        .to_lowercase()
}

/// Save the game to the quicksave slot
pub fn save_game(data: &SaveData) -> Result<()> {
    let path = quicksave_path()?;
    write_save(&path, data)
}

/// Load the game from the quicksave slot
pub fn load_game() -> Result<SaveData> {
    let path = quicksave_path()?;
    read_save(&path)
}

/// Check if a quicksave file exists
pub fn has_quicksave() -> bool {
    quicksave_path().map(|p| p.exists()).unwrap_or(false)
}

/// Save to the autosave slot
pub fn autosave(data: &SaveData) -> Result<()> {
    let path = autosave_path()?;
    write_save(&path, data)
}

/// Save to a named slot
pub fn save_to_slot(slot_name: &str, data: &SaveData) -> Result<()> {
    let filename = sanitize_filename(slot_name);
    let path = slot_path(&filename)?;
    write_save(&path, data)
}

/// Load from a named slot (by filename, not display name)
pub fn load_from_slot(filename: &str) -> Result<SaveData> {
    let path = slot_path(filename)?;
    read_save(&path)
}

/// Delete a save slot
pub fn delete_slot(filename: &str) -> Result<()> {
    let path = slot_path(filename)?;
    if path.exists() {
        fs::remove_file(&path).context("Failed to delete save file")?;
    }
    Ok(())
}

/// List all save slots (excludes quicksave and autosave)
pub fn list_save_slots() -> Result<Vec<SaveSlotInfo>> {
    let dir = save_dir()?;
    let mut slots = Vec::new();

    for entry in fs::read_dir(&dir).context("Failed to read save directory")? {
        let entry = entry?;
        let path = entry.path();

        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }

        let filename = path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_string();

        // Skip quicksave and autosave from the slot listing
        if filename == "quicksave" || filename == "autosave" {
            continue;
        }

        if let Ok(data) = read_save(&path) {
            slots.push(SaveSlotInfo {
                filename,
                slot_name: data.slot_name,
                timestamp: data.timestamp,
                character_name: data.player.character_name,
                era_index: data.world.era_index,
                play_time_seconds: data.play_time_seconds,
            });
        }
    }

    // Sort by timestamp (newest first)
    slots.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
    Ok(slots)
}

// --- Internal helpers ---

fn write_save(path: &PathBuf, data: &SaveData) -> Result<()> {
    let json = serde_json::to_string_pretty(data).context("Failed to serialize save data")?;
    fs::write(path, json).context("Failed to write save file")?;
    Ok(())
}

fn read_save(path: &PathBuf) -> Result<SaveData> {
    let json = fs::read_to_string(path).context("Failed to read save file")?;
    let data: SaveData = serde_json::from_str(&json).context("Failed to deserialize save data")?;
    Ok(data)
}

/// Format play time as "Xh Ym" or "Ym Zs"
pub fn format_play_time(seconds: f64) -> String {
    let total_secs = seconds as u64;
    let hours = total_secs / 3600;
    let minutes = (total_secs % 3600) / 60;
    let secs = total_secs % 60;

    if hours > 0 {
        format!("{}h {}m", hours, minutes)
    } else {
        format!("{}m {}s", minutes, secs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_save_data() -> SaveData {
        SaveData {
            version: 1,
            player: PlayerSaveData {
                position: [10.0, 5.0, -3.0],
                rotation_yaw: 1.5,
                rotation_pitch: -0.3,
                character_name: "TestPlayer".to_string(),
            },
            world: WorldSaveData {
                era_index: 3,
                time_of_day: 14.5,
            },
            timestamp: "2025-01-01 12:00:00".to_string(),
            slot_name: String::new(),
            collected_items: vec!["Gem".to_string()],
            play_time_seconds: 3661.0,
            interactions: InteractionSaveData::default(),
        }
    }

    #[test]
    fn test_round_trip_serialize() {
        let data = test_save_data();
        let json = serde_json::to_string(&data).unwrap();
        let loaded: SaveData = serde_json::from_str(&json).unwrap();

        assert_eq!(loaded.version, 1);
        assert_eq!(loaded.player.position, [10.0, 5.0, -3.0]);
        assert_eq!(loaded.player.rotation_yaw, 1.5);
        assert_eq!(loaded.player.character_name, "TestPlayer");
        assert_eq!(loaded.world.era_index, 3);
        assert_eq!(loaded.world.time_of_day, 14.5);
        assert_eq!(loaded.collected_items, vec!["Gem"]);
        assert_eq!(loaded.play_time_seconds, 3661.0);
    }

    #[test]
    fn test_save_and_load() {
        let data = test_save_data();

        // Save
        save_game(&data).unwrap();

        // Verify file exists
        assert!(has_quicksave());

        // Load
        let loaded = load_game().unwrap();
        assert_eq!(loaded.player.position, data.player.position);
        assert_eq!(loaded.world.era_index, data.world.era_index);
    }

    #[test]
    fn test_sanitize_filename() {
        assert_eq!(sanitize_filename("My Save!"), "my_save_");
        assert_eq!(sanitize_filename("save-01_test"), "save-01_test");
        assert_eq!(sanitize_filename("a b c"), "a_b_c");
    }

    #[test]
    fn test_format_play_time() {
        assert_eq!(format_play_time(0.0), "0m 0s");
        assert_eq!(format_play_time(65.0), "1m 5s");
        assert_eq!(format_play_time(3661.0), "1h 1m");
    }
}
