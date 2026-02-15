use serde::{Deserialize, Serialize};

/// Authentication response from `/v1/auth/login`
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthResponse {
    pub token: String,
    pub refresh_token: String,
    pub expires_in: u64,
    pub user: UserInfo,
}

/// User info returned with auth
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserInfo {
    #[serde(rename = "_id")]
    pub id: String,
    pub user_name: String,
    #[serde(default)]
    pub role: String,
}

/// A character stored on PixygonServer
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerCharacter {
    #[serde(rename = "_id", default)]
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub system_prompt: String,
    #[serde(default)]
    pub lore: Option<CharacterLore>,
    #[serde(default)]
    pub appearance: Option<CharacterAppearanceServer>,
    #[serde(default)]
    pub project_id: String,
    #[serde(default)]
    pub user_id: String,
}

/// Lore/backstory for a character
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CharacterLore {
    #[serde(default)]
    pub backstory: String,
    #[serde(default)]
    pub personality: String,
    #[serde(default)]
    pub occupation: String,
    #[serde(default)]
    pub era: String,
}

/// Server-side appearance data
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CharacterAppearanceServer {
    #[serde(default)]
    pub description: String,
}

/// Request to create a character on the server
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateCharacterRequest {
    pub name: String,
    pub system_prompt: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lore: Option<CharacterLore>,
}

/// A single chat message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

/// Request body for `/v1/ai/chat`
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatRequest {
    pub messages: Vec<ChatMessage>,
    pub system_prompt: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
}

/// Response from `/v1/ai/chat`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatResponse {
    pub content: String,
}

// ============================================
// CharacterItem types (server item catalog)
// ============================================

/// A character item from the server catalog
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerCharacterItem {
    #[serde(rename = "_id", default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub item_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project_id: Option<String>,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default = "default_icon")]
    pub icon: String,
    #[serde(default)]
    pub category: String,
    #[serde(default = "default_subcategory")]
    pub subcategory: String,
    #[serde(default = "default_rarity")]
    pub rarity: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub price: f64,
    #[serde(default)]
    pub stackable: bool,
    #[serde(default = "default_max_stack")]
    pub max_stack: u32,
    #[serde(default = "default_true")]
    pub is_available: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub equip_slot: Option<String>,
    #[serde(default)]
    pub stats: ServerItemStats,
    #[serde(default)]
    pub effects: Vec<ServerItemEffect>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub requirements: Option<ServerItemRequirements>,
}

fn default_icon() -> String { "📦".to_string() }
fn default_subcategory() -> String { "other".to_string() }
fn default_rarity() -> String { "common".to_string() }
fn default_max_stack() -> u32 { 1 }
fn default_true() -> bool { true }

/// Stats block on a server item
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ServerItemStats {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub custom: Option<GameItemCustomStats>,
}

/// Game-specific stats stored in `stats.custom`
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameItemCustomStats {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stat_modifiers: Option<CustomStatModifiers>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub element: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub weapon_data: Option<CustomWeaponData>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gem_sockets: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub item_level: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub required_level: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub game_category: Option<String>,
}

/// Stat modifiers in the custom blob
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CustomStatModifiers {
    #[serde(default)]
    pub max_hp: f32,
    #[serde(default)]
    pub attack: f32,
    #[serde(default)]
    pub defense: f32,
    #[serde(default)]
    pub speed: f32,
    #[serde(default)]
    pub crit_chance: f32,
    #[serde(default)]
    pub crit_multiplier: f32,
}

/// Weapon data in the custom blob
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CustomWeaponData {
    #[serde(default)]
    pub weapon_type: String,
    #[serde(default)]
    pub base_damage: f32,
    #[serde(default)]
    pub weapon_grip: String,
}

/// An effect on a server item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerItemEffect {
    #[serde(rename = "type", default)]
    pub effect_type: String,
    #[serde(default)]
    pub target: String,
    #[serde(default)]
    pub value: f64,
    #[serde(default)]
    pub duration: f64,
    #[serde(default)]
    pub description: String,
}

/// Requirements to use/equip an item
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerItemRequirements {
    #[serde(default)]
    pub level: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub achievement: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quest: Option<String>,
}

/// Response from list items endpoint
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemListResponse {
    pub items: Vec<ServerCharacterItem>,
    #[serde(default)]
    pub total: u64,
}

/// Response from create/update item endpoint
#[derive(Debug, Clone, Deserialize)]
pub struct ItemMutationResponse {
    #[serde(default)]
    pub success: bool,
    pub item: ServerCharacterItem,
}

/// Response from delete endpoint
#[derive(Debug, Clone, Deserialize)]
pub struct DeleteResponse {
    #[serde(default)]
    pub success: bool,
    #[serde(default)]
    pub deleted: bool,
}

// ============================================
// GameStory types (story system)
// ============================================

/// A game story from the server
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerGameStory {
    #[serde(rename = "_id", default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub story_id: String,
    #[serde(default)]
    pub project_id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub start_location: Option<StoryLocation>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub start_year: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub start_time_of_day: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub difficulty: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub estimated_minutes: Option<u32>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub is_published: bool,
    #[serde(default)]
    pub events: Vec<StoryEvent>,
}

/// A 3D location for story start
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoryLocation {
    #[serde(default)]
    pub x: f32,
    #[serde(default)]
    pub y: f32,
    #[serde(default)]
    pub z: f32,
}

/// A story event with trigger and actions
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoryEvent {
    pub event_id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub description: String,
    pub trigger: StoryTrigger,
    #[serde(default)]
    pub actions: Vec<StoryAction>,
    #[serde(default)]
    pub next_events: Vec<String>,
}

/// Trigger definition for a story event
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoryTrigger {
    #[serde(rename = "type")]
    pub trigger_type: String,
    #[serde(default)]
    pub params: serde_json::Value,
}

/// An action to perform when a story event triggers
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoryAction {
    #[serde(rename = "type")]
    pub action_type: String,
    #[serde(default)]
    pub params: serde_json::Value,
}

/// Response from list stories endpoint
#[derive(Debug, Clone, Deserialize)]
pub struct StoryListResponse {
    pub stories: Vec<ServerGameStory>,
}

/// Response from create/update story endpoint
#[derive(Debug, Clone, Deserialize)]
pub struct StoryMutationResponse {
    #[serde(default)]
    pub success: bool,
    pub story: ServerGameStory,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_character_serde_roundtrip() {
        let json = r#"{
            "_id": "abc123",
            "name": "Elder Morvyn",
            "systemPrompt": "You are a wise elder.",
            "lore": {
                "backstory": "Born in the mountains.",
                "personality": "Wise and kind.",
                "occupation": "Village elder",
                "era": "Medieval"
            },
            "appearance": {
                "description": "Tall with grey beard."
            },
            "projectId": "proj1",
            "userId": "user1"
        }"#;

        let character: ServerCharacter = serde_json::from_str(json).unwrap();
        assert_eq!(character.id, "abc123");
        assert_eq!(character.name, "Elder Morvyn");
        assert_eq!(character.system_prompt, "You are a wise elder.");
        assert!(character.lore.is_some());
        let lore = character.lore.as_ref().unwrap();
        assert_eq!(lore.backstory, "Born in the mountains.");

        // Roundtrip
        let serialized = serde_json::to_string(&character).unwrap();
        let deserialized: ServerCharacter = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized.name, "Elder Morvyn");
    }

    #[test]
    fn test_server_character_minimal_json() {
        let json = r#"{"_id": "x"}"#;
        let character: ServerCharacter = serde_json::from_str(json).unwrap();
        assert_eq!(character.id, "x");
        assert_eq!(character.name, "");
        assert!(character.lore.is_none());
    }

    #[test]
    fn test_chat_request_serde() {
        let req = ChatRequest {
            messages: vec![ChatMessage {
                role: "user".into(),
                content: "Hello".into(),
            }],
            system_prompt: "You are an NPC.".into(),
            model: Some("grok".into()),
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("systemPrompt"));
        assert!(json.contains("grok"));
    }

    #[test]
    fn test_auth_response_serde() {
        let json = r#"{
            "token": "jwt123",
            "refreshToken": "ref456",
            "expiresIn": 3600,
            "user": { "_id": "u1", "userName": "testuser" }
        }"#;
        let resp: AuthResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.token, "jwt123");
        assert_eq!(resp.user.id, "u1");
        assert_eq!(resp.user.user_name, "testuser");
    }
}
