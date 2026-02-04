use reqwest::Client;

use crate::error::IntegrationError;
use crate::types::{ChatRequest, ChatResponse};

const BASE_URL: &str = "https://pixygon-server.onrender.com";

/// API client for AI chat via PixygonServer (no auth required)
pub struct AiChatApi {
    client: Client,
}

impl AiChatApi {
    pub fn new(client: Client) -> Self {
        Self { client }
    }

    /// Send a chat request and get an AI response
    pub async fn chat(&self, request: &ChatRequest) -> Result<ChatResponse, IntegrationError> {
        let url = format!("{}/v1/ai/chat", BASE_URL);

        let response = self.client
            .post(&url)
            .json(request)
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let text = response.text().await.unwrap_or_default();
            return Err(IntegrationError::ServerError {
                status: status.as_u16(),
                message: text,
            });
        }

        Ok(response.json().await?)
    }
}
