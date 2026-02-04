use thiserror::Error;

#[derive(Debug, Error)]
pub enum IntegrationError {
    #[error("Network error: {0}")]
    Network(String),

    #[error("Authentication failed: {0}")]
    AuthFailed(String),

    #[error("Server error ({status}): {message}")]
    ServerError { status: u16, message: String },

    #[error("Server is offline or unreachable")]
    Offline,

    #[error("Request timed out")]
    Timeout,

    #[error("Serialization error: {0}")]
    Serialization(String),
}

impl From<reqwest::Error> for IntegrationError {
    fn from(err: reqwest::Error) -> Self {
        if err.is_timeout() {
            IntegrationError::Timeout
        } else if err.is_connect() {
            IntegrationError::Offline
        } else {
            IntegrationError::Network(err.to_string())
        }
    }
}

impl From<serde_json::Error> for IntegrationError {
    fn from(err: serde_json::Error) -> Self {
        IntegrationError::Serialization(err.to_string())
    }
}
