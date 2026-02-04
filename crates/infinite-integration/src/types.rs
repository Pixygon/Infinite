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
