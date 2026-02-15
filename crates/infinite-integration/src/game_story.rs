//! GameStory API client for story system

use reqwest::Client;

use crate::auth::AuthManager;
use crate::error::IntegrationError;
use crate::types::*;

const BASE_URL: &str = "https://pixygon-server.onrender.com";
const PROJECT_ID: &str = "6981e8eda259e89734bd007a";

/// API client for GameStory CRUD
pub struct GameStoryApi {
    client: Client,
}

impl GameStoryApi {
    pub fn new(client: Client) -> Self {
        Self { client }
    }

    /// List all stories for the project
    pub async fn list_stories(
        &self,
        auth: &AuthManager,
    ) -> Result<Vec<ServerGameStory>, IntegrationError> {
        let token = auth.token().ok_or_else(|| IntegrationError::AuthFailed("Not authenticated".into()))?;

        let url = format!("{}/v1/game-stories?projectId={}", BASE_URL, PROJECT_ID);
        let response = self.client
            .get(&url)
            .bearer_auth(&token)
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let text = response.text().await.unwrap_or_default();
            return Err(IntegrationError::ServerError { status: status.as_u16(), message: text });
        }

        // Try to parse as array first (simple response), then as wrapped object
        let text = response.text().await?;
        if let Ok(stories) = serde_json::from_str::<Vec<ServerGameStory>>(&text) {
            return Ok(stories);
        }
        if let Ok(resp) = serde_json::from_str::<StoryListResponse>(&text) {
            return Ok(resp.stories);
        }
        Err(IntegrationError::Serialization("Failed to parse story list response".into()))
    }

    /// Get a single story
    pub async fn get_story(
        &self,
        auth: &AuthManager,
        story_id: &str,
    ) -> Result<ServerGameStory, IntegrationError> {
        let token = auth.token().ok_or_else(|| IntegrationError::AuthFailed("Not authenticated".into()))?;

        let url = format!("{}/v1/game-stories/{}", BASE_URL, story_id);
        let response = self.client
            .get(&url)
            .bearer_auth(&token)
            .send()
            .await?;

        handle_response(response).await
    }

    /// Create a new story
    pub async fn create_story(
        &self,
        auth: &AuthManager,
        story: ServerGameStory,
    ) -> Result<ServerGameStory, IntegrationError> {
        let token = auth.token().ok_or_else(|| IntegrationError::AuthFailed("Not authenticated".into()))?;

        let url = format!("{}/v1/game-stories", BASE_URL);
        let response = self.client
            .post(&url)
            .bearer_auth(&token)
            .json(&story)
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let text = response.text().await.unwrap_or_default();
            return Err(IntegrationError::ServerError { status: status.as_u16(), message: text });
        }

        // Try parsing as { success, story } or as raw story
        let text = response.text().await?;
        if let Ok(resp) = serde_json::from_str::<StoryMutationResponse>(&text) {
            return Ok(resp.story);
        }
        if let Ok(story) = serde_json::from_str::<ServerGameStory>(&text) {
            return Ok(story);
        }
        Err(IntegrationError::Serialization("Failed to parse create story response".into()))
    }

    /// Update a story
    pub async fn update_story(
        &self,
        auth: &AuthManager,
        story_id: &str,
        updates: ServerGameStory,
    ) -> Result<ServerGameStory, IntegrationError> {
        let token = auth.token().ok_or_else(|| IntegrationError::AuthFailed("Not authenticated".into()))?;

        let url = format!("{}/v1/game-stories/{}", BASE_URL, story_id);
        let response = self.client
            .patch(&url)
            .bearer_auth(&token)
            .json(&updates)
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let text = response.text().await.unwrap_or_default();
            return Err(IntegrationError::ServerError { status: status.as_u16(), message: text });
        }

        let text = response.text().await?;
        if let Ok(resp) = serde_json::from_str::<StoryMutationResponse>(&text) {
            return Ok(resp.story);
        }
        if let Ok(story) = serde_json::from_str::<ServerGameStory>(&text) {
            return Ok(story);
        }
        Err(IntegrationError::Serialization("Failed to parse update story response".into()))
    }

    /// Delete a story
    pub async fn delete_story(
        &self,
        auth: &AuthManager,
        story_id: &str,
    ) -> Result<serde_json::Value, IntegrationError> {
        let token = auth.token().ok_or_else(|| IntegrationError::AuthFailed("Not authenticated".into()))?;

        let url = format!("{}/v1/game-stories/{}", BASE_URL, story_id);
        let response = self.client
            .delete(&url)
            .bearer_auth(&token)
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let text = response.text().await.unwrap_or_default();
            return Err(IntegrationError::ServerError { status: status.as_u16(), message: text });
        }

        Ok(response.json().await?)
    }
}

async fn handle_response<T: serde::de::DeserializeOwned>(response: reqwest::Response) -> Result<T, IntegrationError> {
    let status = response.status();
    if !status.is_success() {
        let text = response.text().await.unwrap_or_default();
        return Err(IntegrationError::ServerError { status: status.as_u16(), message: text });
    }
    Ok(response.json().await?)
}
