//! NPC relationship tracking — affection, conversation memory, and tiers

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// Relationship tier based on affection level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RelationshipTier {
    Stranger,
    Acquaintance,
    Friend,
    CloseFriend,
    Trusted,
    Bonded,
}

impl RelationshipTier {
    /// Get the tier for a given affection level (0-100)
    pub fn from_affection(affection: f32) -> Self {
        match affection as u32 {
            0..=15 => RelationshipTier::Stranger,
            16..=35 => RelationshipTier::Acquaintance,
            36..=55 => RelationshipTier::Friend,
            56..=75 => RelationshipTier::CloseFriend,
            76..=90 => RelationshipTier::Trusted,
            _ => RelationshipTier::Bonded,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            RelationshipTier::Stranger => "Stranger",
            RelationshipTier::Acquaintance => "Acquaintance",
            RelationshipTier::Friend => "Friend",
            RelationshipTier::CloseFriend => "Close Friend",
            RelationshipTier::Trusted => "Trusted",
            RelationshipTier::Bonded => "Bonded",
        }
    }
}

/// A message in the relationship history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationshipMessage {
    pub speaker: String,
    pub text: String,
    pub is_player: bool,
}

/// Relationship data for a single NPC
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NpcRelationship {
    pub affection: f32,
    pub times_spoken: u32,
    pub conversation_summary: Option<String>,
    pub recent_messages: Vec<RelationshipMessage>,
}

impl NpcRelationship {
    pub fn new() -> Self {
        Self {
            affection: 0.0,
            times_spoken: 0,
            conversation_summary: None,
            recent_messages: Vec::new(),
        }
    }

    /// Get the current tier
    pub fn tier(&self) -> RelationshipTier {
        RelationshipTier::from_affection(self.affection)
    }

    /// Record a new conversation (called when dialogue ends)
    pub fn record_conversation(&mut self, messages: &[RelationshipMessage]) {
        // Affection gains: +2 per conversation, +1 per message, cap +5 per conversation
        let message_bonus = (messages.len() as f32).min(3.0);
        let gain = (2.0 + message_bonus).min(5.0);
        self.affection = (self.affection + gain).min(100.0);
        self.times_spoken += 1;

        // Add to recent messages
        self.recent_messages.extend_from_slice(messages);

        // Condense if too many messages
        self.condense_if_needed();
    }

    /// Condense older messages into a summary when >30 messages
    fn condense_if_needed(&mut self) {
        const MAX_RECENT: usize = 30;
        const CONDENSE_COUNT: usize = 15;

        if self.recent_messages.len() > MAX_RECENT {
            let to_condense: Vec<_> = self.recent_messages.drain(..CONDENSE_COUNT).collect();

            // Build a simple text summary of the condensed messages
            let summary_lines: Vec<String> = to_condense
                .iter()
                .map(|m| format!("{}: {}", m.speaker, m.text))
                .collect();
            let new_summary = summary_lines.join("\n");

            self.conversation_summary = Some(match &self.conversation_summary {
                Some(existing) => format!("{}\n---\n{}", existing, new_summary),
                None => new_summary,
            });
        }
    }
}

impl Default for NpcRelationship {
    fn default() -> Self {
        Self::new()
    }
}

/// Save data for all NPC relationships
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RelationshipSaveData {
    /// Relationships keyed by persistent_key (as string for JSON)
    pub relationships: HashMap<String, NpcRelationship>,
}

/// Manages all NPC relationships
pub struct RelationshipManager {
    relationships: HashMap<u64, NpcRelationship>,
}

impl RelationshipManager {
    pub fn new() -> Self {
        Self {
            relationships: HashMap::new(),
        }
    }

    /// Get or create a relationship for an NPC
    pub fn get_or_create(&mut self, persistent_key: u64) -> &mut NpcRelationship {
        self.relationships.entry(persistent_key).or_default()
    }

    /// Get a relationship (read-only)
    pub fn get(&self, persistent_key: u64) -> Option<&NpcRelationship> {
        self.relationships.get(&persistent_key)
    }

    /// Convert to save data
    pub fn to_save_data(&self) -> RelationshipSaveData {
        let relationships = self
            .relationships
            .iter()
            .map(|(k, v)| (k.to_string(), v.clone()))
            .collect();
        RelationshipSaveData { relationships }
    }

    /// Restore from save data
    pub fn from_save_data(data: &RelationshipSaveData) -> Self {
        let relationships = data
            .relationships
            .iter()
            .filter_map(|(k, v)| k.parse::<u64>().ok().map(|key| (key, v.clone())))
            .collect();
        Self { relationships }
    }
}

impl Default for RelationshipManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tier_boundaries() {
        assert_eq!(RelationshipTier::from_affection(0.0), RelationshipTier::Stranger);
        assert_eq!(RelationshipTier::from_affection(15.0), RelationshipTier::Stranger);
        assert_eq!(RelationshipTier::from_affection(16.0), RelationshipTier::Acquaintance);
        assert_eq!(RelationshipTier::from_affection(35.0), RelationshipTier::Acquaintance);
        assert_eq!(RelationshipTier::from_affection(36.0), RelationshipTier::Friend);
        assert_eq!(RelationshipTier::from_affection(55.0), RelationshipTier::Friend);
        assert_eq!(RelationshipTier::from_affection(56.0), RelationshipTier::CloseFriend);
        assert_eq!(RelationshipTier::from_affection(75.0), RelationshipTier::CloseFriend);
        assert_eq!(RelationshipTier::from_affection(76.0), RelationshipTier::Trusted);
        assert_eq!(RelationshipTier::from_affection(90.0), RelationshipTier::Trusted);
        assert_eq!(RelationshipTier::from_affection(91.0), RelationshipTier::Bonded);
        assert_eq!(RelationshipTier::from_affection(100.0), RelationshipTier::Bonded);
    }

    #[test]
    fn test_affection_gain() {
        let mut rel = NpcRelationship::new();
        assert_eq!(rel.affection, 0.0);

        let messages = vec![
            RelationshipMessage { speaker: "NPC".into(), text: "Hello!".into(), is_player: false },
            RelationshipMessage { speaker: "Player".into(), text: "Hi!".into(), is_player: true },
        ];
        rel.record_conversation(&messages);

        assert!(rel.affection > 0.0);
        assert!(rel.affection <= 5.0);
        assert_eq!(rel.times_spoken, 1);
    }

    #[test]
    fn test_affection_cap_per_conversation() {
        let mut rel = NpcRelationship::new();

        // Even with many messages, cap at +5
        let messages: Vec<RelationshipMessage> = (0..20)
            .map(|i| RelationshipMessage {
                speaker: format!("Speaker{}", i),
                text: format!("Message {}", i),
                is_player: i % 2 == 0,
            })
            .collect();
        rel.record_conversation(&messages);
        assert!(rel.affection <= 5.0);
    }

    #[test]
    fn test_message_condensation() {
        let mut rel = NpcRelationship::new();

        // Add 35 messages to trigger condensation
        let messages: Vec<RelationshipMessage> = (0..35)
            .map(|i| RelationshipMessage {
                speaker: "Test".into(),
                text: format!("Message {}", i),
                is_player: false,
            })
            .collect();
        rel.record_conversation(&messages);

        assert!(rel.conversation_summary.is_some());
        assert!(rel.recent_messages.len() <= 30);
    }

    #[test]
    fn test_save_load_roundtrip() {
        let mut manager = RelationshipManager::new();
        let rel = manager.get_or_create(42);
        rel.affection = 50.0;
        rel.times_spoken = 3;

        let save_data = manager.to_save_data();
        let json = serde_json::to_string(&save_data).unwrap();
        let loaded: RelationshipSaveData = serde_json::from_str(&json).unwrap();
        let restored = RelationshipManager::from_save_data(&loaded);

        let restored_rel = restored.get(42).unwrap();
        assert_eq!(restored_rel.affection, 50.0);
        assert_eq!(restored_rel.times_spoken, 3);
    }
}
