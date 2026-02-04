use reqwest::Client;

use crate::auth::AuthManager;
use crate::error::IntegrationError;
use crate::types::{CreateCharacterRequest, ServerCharacter};

const BASE_URL: &str = "https://pixygon-server.onrender.com";
const PROJECT_ID: &str = "6981e8eda259e89734bd007a";

/// API client for character CRUD operations on PixygonServer
pub struct CharacterApi {
    client: Client,
}

impl CharacterApi {
    pub fn new(client: Client) -> Self {
        Self { client }
    }

    /// List all characters for the authenticated user
    pub async fn list(&self, auth: &AuthManager) -> Result<Vec<ServerCharacter>, IntegrationError> {
        let token = auth.token().ok_or_else(|| IntegrationError::AuthFailed("Not authenticated".into()))?;
        let user_id = auth.user_id().ok_or_else(|| IntegrationError::AuthFailed("No user ID".into()))?;

        let url = format!("{}/v1/characters/{}/{}", BASE_URL, PROJECT_ID, user_id);
        let response = self.client
            .get(&url)
            .bearer_auth(&token)
            .send()
            .await?;

        handle_response(response).await
    }

    /// Get a specific character by ID
    pub async fn get(&self, auth: &AuthManager, character_id: &str) -> Result<ServerCharacter, IntegrationError> {
        let token = auth.token().ok_or_else(|| IntegrationError::AuthFailed("Not authenticated".into()))?;
        let user_id = auth.user_id().ok_or_else(|| IntegrationError::AuthFailed("No user ID".into()))?;

        let url = format!("{}/v1/characters/{}/{}/{}", BASE_URL, PROJECT_ID, user_id, character_id);
        let response = self.client
            .get(&url)
            .bearer_auth(&token)
            .send()
            .await?;

        handle_response(response).await
    }

    /// Create a new character
    pub async fn create(&self, auth: &AuthManager, req: CreateCharacterRequest) -> Result<ServerCharacter, IntegrationError> {
        let token = auth.token().ok_or_else(|| IntegrationError::AuthFailed("Not authenticated".into()))?;
        let user_id = auth.user_id().ok_or_else(|| IntegrationError::AuthFailed("No user ID".into()))?;

        let url = format!("{}/v1/characters/{}/{}", BASE_URL, PROJECT_ID, user_id);
        let response = self.client
            .post(&url)
            .bearer_auth(&token)
            .json(&req)
            .send()
            .await?;

        handle_response(response).await
    }

    /// Update a character
    pub async fn update(&self, auth: &AuthManager, character_id: &str, updates: serde_json::Value) -> Result<ServerCharacter, IntegrationError> {
        let token = auth.token().ok_or_else(|| IntegrationError::AuthFailed("Not authenticated".into()))?;
        let user_id = auth.user_id().ok_or_else(|| IntegrationError::AuthFailed("No user ID".into()))?;

        let url = format!("{}/v1/characters/{}/{}/{}", BASE_URL, PROJECT_ID, user_id, character_id);
        let response = self.client
            .patch(&url)
            .bearer_auth(&token)
            .json(&updates)
            .send()
            .await?;

        handle_response(response).await
    }
}

async fn handle_response<T: serde::de::DeserializeOwned>(response: reqwest::Response) -> Result<T, IntegrationError> {
    let status = response.status();
    if status == reqwest::StatusCode::UNAUTHORIZED || status == reqwest::StatusCode::FORBIDDEN {
        let text = response.text().await.unwrap_or_default();
        return Err(IntegrationError::AuthFailed(text));
    }
    if !status.is_success() {
        let text = response.text().await.unwrap_or_default();
        return Err(IntegrationError::ServerError {
            status: status.as_u16(),
            message: text,
        });
    }
    Ok(response.json().await?)
}
