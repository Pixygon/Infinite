//! Lazy NPC character generation — creates server characters on first interaction

use infinite_integration::{
    CreateCharacterRequest, IntegrationClient, PendingRequest, ServerCharacter,
};

use super::archetype_mapping::generate_system_prompt;
use super::character_cache::NpcCharacterCache;
use super::NpcRole;

/// A pending character generation
struct PendingGeneration {
    persistent_key: u64,
    pending: PendingRequest<ServerCharacter>,
}

/// Manages lazy character generation for NPCs
pub struct NpcGenerator {
    pending: Vec<PendingGeneration>,
}

impl NpcGenerator {
    pub fn new() -> Self {
        Self {
            pending: Vec::new(),
        }
    }

    /// Request character generation for an NPC (called on first interaction)
    pub fn generate_for_npc(
        &mut self,
        persistent_key: u64,
        npc_name: &str,
        role: NpcRole,
        year: i64,
        client: &IntegrationClient,
    ) {
        let system_prompt = generate_system_prompt(npc_name, role, year);

        let req = CreateCharacterRequest {
            name: npc_name.to_string(),
            system_prompt,
            lore: None,
        };

        let pending = client.create_character(req);
        self.pending.push(PendingGeneration {
            persistent_key,
            pending,
        });
    }

    /// Poll pending generations, updating the cache with results
    pub fn poll(&mut self, cache: &mut NpcCharacterCache) {
        self.pending.retain_mut(|gen| {
            match gen.pending.try_recv() {
                Some(Ok(character)) => {
                    tracing::info!("Generated character for NPC key {}: {}", gen.persistent_key, character.name);
                    cache.set_ready(gen.persistent_key, character);
                    false // Remove from pending
                }
                Some(Err(e)) => {
                    tracing::warn!("Failed to generate character for NPC key {}: {}", gen.persistent_key, e);
                    cache.set_failed(gen.persistent_key);
                    false // Remove from pending
                }
                None => true, // Still waiting
            }
        });
    }

    /// Whether there are any pending generations
    pub fn has_pending(&self) -> bool {
        !self.pending.is_empty()
    }
}

impl Default for NpcGenerator {
    fn default() -> Self {
        Self::new()
    }
}
