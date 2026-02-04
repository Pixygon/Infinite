//! AI-powered dialogue system for NPCs with server character data

use std::collections::HashMap;

use infinite_integration::{
    ChatMessage, ChatRequest, ChatResponse, IntegrationClient, PendingRequest, ServerCharacter,
};

use super::game_context::GameContext;

/// A message displayed in the dialogue UI
#[derive(Debug, Clone)]
pub struct DisplayMessage {
    pub speaker: String,
    pub text: String,
    pub is_player: bool,
}

/// Current state of an AI dialogue
pub enum AiDialogueState {
    /// Waiting for player input
    WaitingForInput {
        messages: Vec<DisplayMessage>,
    },
    /// Waiting for AI response
    WaitingForResponse {
        pending: PendingRequest<ChatResponse>,
        messages: Vec<DisplayMessage>,
    },
    /// An error occurred
    Error(String),
}

/// An active AI conversation
struct ActiveAiDialogue {
    npc_id: super::NpcId,
    persistent_key: u64,
    npc_name: String,
    system_prompt: String,
    chat_history: Vec<ChatMessage>,
    state: AiDialogueState,
}

/// Manages AI-powered NPC conversations
pub struct AiDialogueManager {
    active: Option<ActiveAiDialogue>,
    /// Stored conversation histories keyed by persistent_key (survives across dialogue sessions)
    conversation_histories: HashMap<u64, Vec<ChatMessage>>,
}

impl AiDialogueManager {
    pub fn new() -> Self {
        Self {
            active: None,
            conversation_histories: HashMap::new(),
        }
    }

    /// Start a new AI dialogue with an NPC
    pub fn start_dialogue(
        &mut self,
        npc_id: super::NpcId,
        persistent_key: u64,
        npc_name: String,
        character: &ServerCharacter,
        context: GameContext,
        client: &IntegrationClient,
    ) {
        let game_context = context.to_system_context();
        let system_prompt = if character.system_prompt.is_empty() {
            format!(
                "You are {}, an NPC in a time-travel game. Stay in character. Keep responses under 3 sentences.\n\n{}",
                npc_name, game_context
            )
        } else {
            format!("{}\n\n{}", character.system_prompt, game_context)
        };

        // Restore previous conversation history if any
        let previous_history = self.conversation_histories.get(&persistent_key).cloned().unwrap_or_default();

        // Build greeting request
        let mut chat_history = previous_history;
        let greeting_msg = if chat_history.is_empty() {
            ChatMessage {
                role: "user".into(),
                content: format!("*A traveler approaches you.* Greet them as {}, staying in character.", npc_name),
            }
        } else {
            ChatMessage {
                role: "user".into(),
                content: format!("*The traveler returns.* Greet them again as {}, acknowledging you've met before.", npc_name),
            }
        };
        chat_history.push(greeting_msg);

        let request = ChatRequest {
            messages: chat_history.clone(),
            system_prompt: system_prompt.clone(),
            model: Some("grok".into()),
        };

        let pending = client.send_chat(request);

        let display_messages = Vec::new();

        self.active = Some(ActiveAiDialogue {
            npc_id,
            persistent_key,
            npc_name,
            system_prompt,
            chat_history,
            state: AiDialogueState::WaitingForResponse {
                pending,
                messages: display_messages,
            },
        });
    }

    /// Send a player message and request AI response
    pub fn send_player_message(&mut self, text: String, client: &IntegrationClient) {
        let active = match &mut self.active {
            Some(a) => a,
            None => return,
        };

        // Add player message to chat history
        active.chat_history.push(ChatMessage {
            role: "user".into(),
            content: text.clone(),
        });

        // Add to display messages
        let mut messages = match &active.state {
            AiDialogueState::WaitingForInput { messages } => messages.clone(),
            _ => Vec::new(),
        };
        messages.push(DisplayMessage {
            speaker: "You".into(),
            text,
            is_player: true,
        });

        // Fire off AI request
        let request = ChatRequest {
            messages: active.chat_history.clone(),
            system_prompt: active.system_prompt.clone(),
            model: Some("grok".into()),
        };
        let pending = client.send_chat(request);

        active.state = AiDialogueState::WaitingForResponse { pending, messages };
    }

    /// Poll for AI response. Returns true when a new message arrives.
    pub fn update(&mut self) -> bool {
        let active = match &mut self.active {
            Some(a) => a,
            None => return false,
        };

        let new_state = match &active.state {
            AiDialogueState::WaitingForResponse { pending, messages } => {
                match pending.try_recv() {
                    Some(Ok(response)) => {
                        let mut messages = messages.clone();
                        messages.push(DisplayMessage {
                            speaker: active.npc_name.clone(),
                            text: response.content.clone(),
                            is_player: false,
                        });

                        // Record in chat history
                        active.chat_history.push(ChatMessage {
                            role: "assistant".into(),
                            content: response.content,
                        });

                        Some(AiDialogueState::WaitingForInput { messages })
                    }
                    Some(Err(e)) => {
                        Some(AiDialogueState::Error(format!("AI error: {}", e)))
                    }
                    None => None, // Still waiting
                }
            }
            _ => None,
        };

        if let Some(state) = new_state {
            active.state = state;
            true
        } else {
            false
        }
    }

    /// End the current dialogue, saving conversation history
    pub fn end_dialogue(&mut self) {
        if let Some(active) = self.active.take() {
            self.conversation_histories.insert(active.persistent_key, active.chat_history);
        }
    }

    /// Whether an AI dialogue is currently active
    pub fn is_active(&self) -> bool {
        self.active.is_some()
    }

    /// Get the current dialogue state for UI rendering
    pub fn active_state(&self) -> Option<&AiDialogueState> {
        self.active.as_ref().map(|a| &a.state)
    }

    /// Get the active NPC's name
    pub fn active_npc_name(&self) -> Option<&str> {
        self.active.as_ref().map(|a| a.npc_name.as_str())
    }

    /// Get the active NPC's ID
    pub fn active_npc_id(&self) -> Option<super::NpcId> {
        self.active.as_ref().map(|a| a.npc_id)
    }

    /// Get stored conversation histories (for save/load)
    pub fn conversation_histories(&self) -> &HashMap<u64, Vec<ChatMessage>> {
        &self.conversation_histories
    }

    /// Restore conversation histories (from save/load)
    pub fn set_conversation_histories(&mut self, histories: HashMap<u64, Vec<ChatMessage>>) {
        self.conversation_histories = histories;
    }
}

impl Default for AiDialogueManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ai_dialogue_manager_lifecycle() {
        let manager = AiDialogueManager::new();
        assert!(!manager.is_active());
        assert!(manager.active_state().is_none());
        assert!(manager.active_npc_name().is_none());
    }

    #[test]
    fn test_conversation_history_storage() {
        let mut manager = AiDialogueManager::new();
        assert!(manager.conversation_histories().is_empty());

        let histories = HashMap::from([(
            42u64,
            vec![ChatMessage {
                role: "user".into(),
                content: "Hello".into(),
            }],
        )]);
        manager.set_conversation_histories(histories);
        assert_eq!(manager.conversation_histories().len(), 1);
        assert!(manager.conversation_histories().contains_key(&42));
    }
}
