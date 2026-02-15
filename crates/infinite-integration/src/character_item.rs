//! CharacterItem API client for server-backed item catalog

use reqwest::Client;

use crate::auth::AuthManager;
use crate::error::IntegrationError;
use crate::types::*;

const BASE_URL: &str = "https://pixygon-server.onrender.com";
const PROJECT_ID: &str = "6981e8eda259e89734bd007a";

/// API client for CharacterItem CRUD
pub struct CharacterItemApi {
    client: Client,
}

impl CharacterItemApi {
    pub fn new(client: Client) -> Self {
        Self { client }
    }

    /// List all items for the project
    pub async fn list_project_items(
        &self,
        auth: &AuthManager,
    ) -> Result<Vec<ServerCharacterItem>, IntegrationError> {
        let token = auth.token().ok_or_else(|| IntegrationError::AuthFailed("Not authenticated".into()))?;

        let url = format!("{}/v1/character-items/project/{}", BASE_URL, PROJECT_ID);
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

        // The endpoint returns { items: [...], total, ... }
        let body: ItemListResponse = response.json().await?;
        Ok(body.items)
    }

    /// Get a single item by itemId
    pub async fn get_item(
        &self,
        auth: &AuthManager,
        item_id: &str,
    ) -> Result<ServerCharacterItem, IntegrationError> {
        let token = auth.token().ok_or_else(|| IntegrationError::AuthFailed("Not authenticated".into()))?;

        let url = format!("{}/v1/character-items/{}", BASE_URL, item_id);
        let response = self.client
            .get(&url)
            .bearer_auth(&token)
            .send()
            .await?;

        handle_response(response).await
    }

    /// Create a new item
    pub async fn create_item(
        &self,
        auth: &AuthManager,
        item: ServerCharacterItem,
    ) -> Result<ServerCharacterItem, IntegrationError> {
        let token = auth.token().ok_or_else(|| IntegrationError::AuthFailed("Not authenticated".into()))?;

        let url = format!("{}/v1/character-items", BASE_URL);
        let response = self.client
            .post(&url)
            .bearer_auth(&token)
            .json(&item)
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let text = response.text().await.unwrap_or_default();
            return Err(IntegrationError::ServerError { status: status.as_u16(), message: text });
        }

        let body: ItemMutationResponse = response.json().await?;
        Ok(body.item)
    }

    /// Update an existing item
    pub async fn update_item(
        &self,
        auth: &AuthManager,
        item_id: &str,
        updates: ServerCharacterItem,
    ) -> Result<ServerCharacterItem, IntegrationError> {
        let token = auth.token().ok_or_else(|| IntegrationError::AuthFailed("Not authenticated".into()))?;

        let url = format!("{}/v1/character-items/{}", BASE_URL, item_id);
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

        let body: ItemMutationResponse = response.json().await?;
        Ok(body.item)
    }

    /// Delete an item by itemId
    pub async fn delete_item(
        &self,
        auth: &AuthManager,
        item_id: &str,
    ) -> Result<serde_json::Value, IntegrationError> {
        let token = auth.token().ok_or_else(|| IntegrationError::AuthFailed("Not authenticated".into()))?;

        let url = format!("{}/v1/character-items/{}", BASE_URL, item_id);
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
    if status == reqwest::StatusCode::UNAUTHORIZED || status == reqwest::StatusCode::FORBIDDEN {
        let text = response.text().await.unwrap_or_default();
        return Err(IntegrationError::AuthFailed(text));
    }
    if !status.is_success() {
        let text = response.text().await.unwrap_or_default();
        return Err(IntegrationError::ServerError { status: status.as_u16(), message: text });
    }
    Ok(response.json().await?)
}
