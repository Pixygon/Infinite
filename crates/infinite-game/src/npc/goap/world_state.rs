//! World state representation for GOAP planning

use std::collections::HashMap;

/// A single fact about the world
#[derive(Debug, Clone, PartialEq)]
pub enum WorldFact {
    Bool(bool),
    Float(f32),
}

/// Set of facts describing the current world state
#[derive(Debug, Clone)]
pub struct WorldState {
    facts: HashMap<String, WorldFact>,
}

impl WorldState {
    pub fn new() -> Self {
        Self {
            facts: HashMap::new(),
        }
    }

    /// Create a WorldState with a single bool fact
    pub fn from_bool(key: &str, val: bool) -> Self {
        let mut ws = Self::new();
        ws.set_bool(key, val);
        ws
    }

    pub fn set_bool(&mut self, key: &str, val: bool) {
        self.facts.insert(key.to_string(), WorldFact::Bool(val));
    }

    pub fn get_bool(&self, key: &str) -> Option<bool> {
        match self.facts.get(key) {
            Some(WorldFact::Bool(v)) => Some(*v),
            _ => None,
        }
    }

    pub fn set_float(&mut self, key: &str, val: f32) {
        self.facts.insert(key.to_string(), WorldFact::Float(val));
    }

    pub fn get_float(&self, key: &str) -> Option<f32> {
        match self.facts.get(key) {
            Some(WorldFact::Float(v)) => Some(*v),
            _ => None,
        }
    }

    /// Check if all facts in `required` are satisfied by this state.
    /// Missing facts are treated as not satisfied.
    pub fn satisfies(&self, required: &WorldState) -> bool {
        for (key, required_val) in &required.facts {
            match self.facts.get(key) {
                Some(actual) => {
                    if actual != required_val {
                        return false;
                    }
                }
                None => return false,
            }
        }
        true
    }

    /// Count unsatisfied facts compared to `required`
    pub fn unsatisfied_count(&self, required: &WorldState) -> usize {
        let mut count = 0;
        for (key, required_val) in &required.facts {
            match self.facts.get(key) {
                Some(actual) if actual == required_val => {}
                _ => count += 1,
            }
        }
        count
    }

    /// Apply effects: merge all facts from `effects` into this state
    pub fn apply(&mut self, effects: &WorldState) {
        for (key, val) in &effects.facts {
            self.facts.insert(key.clone(), val.clone());
        }
    }

    /// Number of facts
    pub fn len(&self) -> usize {
        self.facts.len()
    }

    pub fn is_empty(&self) -> bool {
        self.facts.is_empty()
    }
}

impl Default for WorldState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_get_bool() {
        let mut ws = WorldState::new();
        ws.set_bool("hungry", true);
        assert_eq!(ws.get_bool("hungry"), Some(true));
        assert_eq!(ws.get_bool("missing"), None);
    }

    #[test]
    fn test_set_get_float() {
        let mut ws = WorldState::new();
        ws.set_float("health", 100.0);
        assert_eq!(ws.get_float("health"), Some(100.0));
    }

    #[test]
    fn test_satisfies_empty() {
        let ws = WorldState::new();
        let required = WorldState::new();
        assert!(ws.satisfies(&required));
    }

    #[test]
    fn test_satisfies_matching() {
        let mut ws = WorldState::new();
        ws.set_bool("at_home", true);
        ws.set_bool("hungry", false);

        let mut required = WorldState::new();
        required.set_bool("at_home", true);
        assert!(ws.satisfies(&required));
    }

    #[test]
    fn test_satisfies_missing() {
        let ws = WorldState::new();
        let required = WorldState::from_bool("at_home", true);
        assert!(!ws.satisfies(&required));
    }

    #[test]
    fn test_satisfies_wrong_value() {
        let mut ws = WorldState::new();
        ws.set_bool("at_home", false);
        let required = WorldState::from_bool("at_home", true);
        assert!(!ws.satisfies(&required));
    }

    #[test]
    fn test_apply_effects() {
        let mut ws = WorldState::new();
        ws.set_bool("hungry", true);

        let mut effects = WorldState::new();
        effects.set_bool("hungry", false);
        effects.set_bool("full", true);

        ws.apply(&effects);
        assert_eq!(ws.get_bool("hungry"), Some(false));
        assert_eq!(ws.get_bool("full"), Some(true));
    }

    #[test]
    fn test_unsatisfied_count() {
        let mut ws = WorldState::new();
        ws.set_bool("a", true);

        let mut required = WorldState::new();
        required.set_bool("a", true);
        required.set_bool("b", true);
        required.set_bool("c", true);

        assert_eq!(ws.unsatisfied_count(&required), 2);
    }
}
