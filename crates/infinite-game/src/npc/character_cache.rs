use std::collections::HashMap;

use infinite_integration::ServerCharacter;

/// Status of a cached server character for an NPC
#[derive(Debug, Clone)]
pub enum CharacterCacheEntry {
    /// Request in flight
    Pending,
    /// Character data received
    Ready(Box<ServerCharacter>),
    /// Request failed
    Failed,
}

/// Cache of server characters, keyed by NPC persistent_key
pub struct NpcCharacterCache {
    entries: HashMap<u64, CharacterCacheEntry>,
}

impl NpcCharacterCache {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    /// Get a cache entry by persistent key
    pub fn get(&self, key: &u64) -> Option<&CharacterCacheEntry> {
        self.entries.get(key)
    }

    /// Mark as pending (request in flight)
    pub fn set_pending(&mut self, key: u64) {
        self.entries.insert(key, CharacterCacheEntry::Pending);
    }

    /// Store a ready character
    pub fn set_ready(&mut self, key: u64, character: ServerCharacter) {
        self.entries.insert(key, CharacterCacheEntry::Ready(Box::new(character)));
    }

    /// Mark as failed
    pub fn set_failed(&mut self, key: u64) {
        self.entries.insert(key, CharacterCacheEntry::Failed);
    }

    /// Remove a specific key (on chunk unload)
    pub fn clear_key(&mut self, key: u64) {
        self.entries.remove(&key);
    }
}

impl Default for NpcCharacterCache {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_insert_get_clear() {
        let mut cache = NpcCharacterCache::new();

        assert!(cache.get(&1).is_none());

        cache.set_pending(1);
        assert!(matches!(cache.get(&1), Some(CharacterCacheEntry::Pending)));

        let character = ServerCharacter {
            id: "abc".into(),
            name: "Test NPC".into(),
            system_prompt: "You are a test.".into(),
            lore: None,
            appearance: None,
            project_id: String::new(),
            user_id: String::new(),
        };
        cache.set_ready(1, character);
        assert!(matches!(cache.get(&1), Some(CharacterCacheEntry::Ready(_))));

        cache.clear_key(1);
        assert!(cache.get(&1).is_none());
    }

    #[test]
    fn test_cache_failed() {
        let mut cache = NpcCharacterCache::new();
        cache.set_failed(42);
        assert!(matches!(cache.get(&42), Some(CharacterCacheEntry::Failed)));
    }
}
