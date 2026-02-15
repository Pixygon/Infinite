use std::sync::{Arc, RwLock};

use reqwest::Client;
use tracing::{info, warn};

use crate::error::IntegrationError;
use crate::types::{AuthResponse, UserInfo};

const BASE_URL: &str = "https://pixygon-server.onrender.com";

/// Manages authentication state and token refresh
pub struct AuthManager {
    client: Client,
    token: Arc<RwLock<Option<String>>>,
    refresh_token: Arc<RwLock<Option<String>>>,
    user: Arc<RwLock<Option<UserInfo>>>,
}

impl AuthManager {
    pub fn new(client: Client) -> Self {
        Self {
            client,
            token: Arc::new(RwLock::new(None)),
            refresh_token: Arc::new(RwLock::new(None)),
            user: Arc::new(RwLock::new(None)),
        }
    }

    /// Log in with username and password
    pub async fn login(&self, username: String, password: String) -> Result<AuthResponse, IntegrationError> {
        let url = format!("{}/v1/auth/login", BASE_URL);

        let body = serde_json::json!({
            "userName": username,
            "password": password,
        });

        let response = self.client
            .post(&url)
            .json(&body)
            .send()
            .await?;

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

        let auth: AuthResponse = response.json().await?;

        // Store tokens
        if let Ok(mut t) = self.token.write() {
            *t = Some(auth.token.clone());
        }
        if let Ok(mut rt) = self.refresh_token.write() {
            *rt = Some(auth.refresh_token.clone());
        }
        if let Ok(mut u) = self.user.write() {
            *u = Some(auth.user.clone());
        }

        info!("Logged in as {}", auth.user.user_name);
        Ok(auth)
    }

    /// Get the current JWT token, if authenticated
    pub fn token(&self) -> Option<String> {
        self.token.read().ok()?.clone()
    }

    /// Get the current user ID, if authenticated
    pub fn user_id(&self) -> Option<String> {
        self.user.read().ok()?.as_ref().map(|u| u.id.clone())
    }

    /// Whether we currently have a valid token
    pub fn is_authenticated(&self) -> bool {
        self.token.read().ok().map(|t| t.is_some()).unwrap_or(false)
    }

    /// Whether the current user is an admin or superadmin
    pub fn is_admin(&self) -> bool {
        self.user.read().ok()
            .and_then(|u| u.as_ref().map(|u| u.role == "admin" || u.role == "superadmin"))
            .unwrap_or(false)
    }

    /// Get the current user's display name
    pub fn user_name(&self) -> Option<String> {
        self.user.read().ok()?.as_ref().map(|u| u.user_name.clone())
    }

    /// Clear all auth state
    pub fn logout(&self) {
        if let Ok(mut t) = self.token.write() {
            *t = None;
        }
        if let Ok(mut rt) = self.refresh_token.write() {
            *rt = None;
        }
        if let Ok(mut u) = self.user.write() {
            *u = None;
        }
        warn!("Logged out");
    }
}
