use std::sync::mpsc;
use std::sync::Arc;
use std::time::Duration;

use reqwest::Client;

use crate::auth::AuthManager;
use crate::ai_chat::AiChatApi;
use crate::character::CharacterApi;
use crate::error::IntegrationError;
use crate::types::*;

/// A non-blocking handle to an in-flight async request.
/// Call `try_recv()` each frame to check for results without blocking the game loop.
pub struct PendingRequest<T> {
    receiver: mpsc::Receiver<Result<T, IntegrationError>>,
}

impl<T> PendingRequest<T> {
    /// Non-blocking check for the result. Returns `None` if still pending.
    pub fn try_recv(&self) -> Option<Result<T, IntegrationError>> {
        self.receiver.try_recv().ok()
    }

    /// Blocking wait for the result. Only use during loading screens.
    pub fn wait(self) -> Result<T, IntegrationError> {
        self.receiver.recv().map_err(|_| IntegrationError::Network("Channel closed".into()))?
    }
}

/// Facade for all PixygonServer interactions.
/// Owns a background tokio runtime and dispatches async work via channels.
pub struct IntegrationClient {
    runtime: tokio::runtime::Runtime,
    auth: Arc<AuthManager>,
    character_api: Arc<CharacterApi>,
    ai_chat_api: Arc<AiChatApi>,
    online: Arc<std::sync::atomic::AtomicBool>,
}

impl IntegrationClient {
    /// Create a new integration client with a background tokio runtime.
    pub fn new() -> Result<Self, IntegrationError> {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .enable_all()
            .build()
            .map_err(|e| IntegrationError::Network(format!("Failed to create runtime: {}", e)))?;

        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(|e| IntegrationError::Network(format!("Failed to create HTTP client: {}", e)))?;

        let auth = Arc::new(AuthManager::new(client.clone()));
        let character_api = Arc::new(CharacterApi::new(client.clone()));
        let ai_chat_api = Arc::new(AiChatApi::new(client));

        Ok(Self {
            runtime,
            auth,
            character_api,
            ai_chat_api,
            online: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        })
    }

    /// Log in with username and password. Returns a pending request with the auth response.
    pub fn login(&self, username: String, password: String) -> PendingRequest<AuthResponse> {
        let (tx, rx) = mpsc::channel();
        let auth = Arc::clone(&self.auth);
        let online = Arc::clone(&self.online);

        self.runtime.spawn(async move {
            let result = auth.login(username, password).await;
            match &result {
                Ok(_) => online.store(true, std::sync::atomic::Ordering::Relaxed),
                Err(IntegrationError::Offline) => online.store(false, std::sync::atomic::Ordering::Relaxed),
                _ => {}
            }
            let _ = tx.send(result);
        });

        PendingRequest { receiver: rx }
    }

    /// Fetch a specific character by ID.
    pub fn fetch_character(&self, character_id: String) -> PendingRequest<ServerCharacter> {
        let (tx, rx) = mpsc::channel();
        let auth = Arc::clone(&self.auth);
        let api = Arc::clone(&self.character_api);

        self.runtime.spawn(async move {
            let result = api.get(&auth, &character_id).await;
            let _ = tx.send(result);
        });

        PendingRequest { receiver: rx }
    }

    /// List all characters for the authenticated user.
    pub fn list_characters(&self) -> PendingRequest<Vec<ServerCharacter>> {
        let (tx, rx) = mpsc::channel();
        let auth = Arc::clone(&self.auth);
        let api = Arc::clone(&self.character_api);

        self.runtime.spawn(async move {
            let result = api.list(&auth).await;
            let _ = tx.send(result);
        });

        PendingRequest { receiver: rx }
    }

    /// Create a new character on the server.
    pub fn create_character(&self, req: CreateCharacterRequest) -> PendingRequest<ServerCharacter> {
        let (tx, rx) = mpsc::channel();
        let auth = Arc::clone(&self.auth);
        let api = Arc::clone(&self.character_api);

        self.runtime.spawn(async move {
            let result = api.create(&auth, req).await;
            let _ = tx.send(result);
        });

        PendingRequest { receiver: rx }
    }

    /// Send a chat request to the AI endpoint (no auth required).
    pub fn send_chat(&self, request: ChatRequest) -> PendingRequest<ChatResponse> {
        let (tx, rx) = mpsc::channel();
        let api = Arc::clone(&self.ai_chat_api);

        self.runtime.spawn(async move {
            let result = api.chat(&request).await;
            let _ = tx.send(result);
        });

        PendingRequest { receiver: rx }
    }

    /// Whether the server appears to be online (based on last request result).
    pub fn is_online(&self) -> bool {
        self.online.load(std::sync::atomic::Ordering::Relaxed)
    }

    /// Whether the client has valid authentication.
    pub fn is_authenticated(&self) -> bool {
        self.auth.is_authenticated()
    }
}

impl Default for IntegrationClient {
    fn default() -> Self {
        Self::new().expect("Failed to create IntegrationClient")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pending_request_try_recv_none_then_result() {
        let (tx, rx) = mpsc::channel();
        let pending: PendingRequest<String> = PendingRequest { receiver: rx };

        // Before sending, should return None
        assert!(pending.try_recv().is_none());

        // Send a result
        tx.send(Ok("hello".to_string())).unwrap();

        // Now should return Some
        let result = pending.try_recv();
        assert!(result.is_some());
        assert_eq!(result.unwrap().unwrap(), "hello");
    }

    #[test]
    fn test_pending_request_wait() {
        let (tx, rx) = mpsc::channel();
        let pending: PendingRequest<u32> = PendingRequest { receiver: rx };

        tx.send(Ok(42)).unwrap();
        assert_eq!(pending.wait().unwrap(), 42);
    }

    #[test]
    fn test_pending_request_error() {
        let (tx, rx) = mpsc::channel();
        let pending: PendingRequest<String> = PendingRequest { receiver: rx };

        tx.send(Err(IntegrationError::Offline)).unwrap();

        let result = pending.try_recv();
        assert!(result.is_some());
        assert!(result.unwrap().is_err());
    }

    #[test]
    fn test_error_type_mapping() {
        // Test that our error variants exist and display correctly
        let offline = IntegrationError::Offline;
        assert!(offline.to_string().contains("offline"));

        let auth = IntegrationError::AuthFailed("bad credentials".into());
        assert!(auth.to_string().contains("Authentication failed"));

        let server = IntegrationError::ServerError { status: 500, message: "Internal".into() };
        assert!(server.to_string().contains("500"));

        let timeout = IntegrationError::Timeout;
        assert!(timeout.to_string().contains("timed out"));
    }
}
