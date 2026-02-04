//! NPC dialogue system — conversation trees and dialogue state

use std::collections::HashMap;

use super::{NpcId, NpcRole};

/// A single dialogue step
#[derive(Debug, Clone)]
pub struct DialogueNode {
    pub speaker: String,
    pub text: String,
    pub responses: Vec<DialogueResponse>,
}

/// A player response option
#[derive(Debug, Clone)]
pub struct DialogueResponse {
    pub text: String,
    /// Index of next DialogueNode (None = end conversation)
    pub next_node: Option<usize>,
}

/// A full conversation tree
#[derive(Debug, Clone)]
pub struct DialogueTree {
    pub nodes: Vec<DialogueNode>,
    pub start_node: usize,
}

/// Tracks which NPCs have been talked to (for quest flags etc.)
#[derive(Debug, Clone, Default)]
pub struct ConversationHistory {
    pub talked_to: Vec<NpcId>,
}

/// Active conversation state
#[derive(Debug, Clone)]
pub struct ActiveDialogue {
    pub npc_id: NpcId,
    pub npc_name: String,
    pub tree_key: String,
    pub current_node: usize,
}

/// Manages dialogue trees and active conversations
pub struct DialogueSystem {
    trees: HashMap<String, DialogueTree>,
    active: Option<ActiveDialogue>,
    pub history: ConversationHistory,
}

impl DialogueSystem {
    pub fn new() -> Self {
        let mut sys = Self {
            trees: HashMap::new(),
            active: None,
            history: ConversationHistory::default(),
        };
        sys.register_defaults();
        sys
    }

    /// Start a dialogue with an NPC
    pub fn start_dialogue(&mut self, npc_id: NpcId, npc_name: String, role: NpcRole) {
        let tree_key = role_tree_key(role);
        if self.trees.contains_key(&tree_key) {
            let start = self.trees[&tree_key].start_node;
            self.active = Some(ActiveDialogue {
                npc_id,
                npc_name,
                tree_key,
                current_node: start,
            });
            if !self.history.talked_to.contains(&npc_id) {
                self.history.talked_to.push(npc_id);
            }
        }
    }

    /// Get the current dialogue node (if a conversation is active)
    pub fn current_node(&self) -> Option<&DialogueNode> {
        let active = self.active.as_ref()?;
        let tree = self.trees.get(&active.tree_key)?;
        tree.nodes.get(active.current_node)
    }

    /// Get the active dialogue info
    pub fn active(&self) -> Option<&ActiveDialogue> {
        self.active.as_ref()
    }

    /// Whether a dialogue is currently active
    pub fn is_active(&self) -> bool {
        self.active.is_some()
    }

    /// Choose a response by index, advancing the dialogue
    pub fn choose_response(&mut self, index: usize) {
        let active = match &self.active {
            Some(a) => a,
            None => return,
        };
        let tree = match self.trees.get(&active.tree_key) {
            Some(t) => t,
            None => return,
        };
        let node = match tree.nodes.get(active.current_node) {
            Some(n) => n,
            None => return,
        };
        let response = match node.responses.get(index) {
            Some(r) => r,
            None => return,
        };

        match response.next_node {
            Some(next) => {
                if let Some(active) = &mut self.active {
                    active.current_node = next;
                }
            }
            None => {
                self.active = None;
            }
        }
    }

    /// End the current dialogue
    pub fn end_dialogue(&mut self) {
        self.active = None;
    }

    /// Register default dialogue trees for each NPC role
    fn register_defaults(&mut self) {
        self.trees.insert("villager".into(), villager_tree());
        self.trees.insert("guard".into(), guard_tree());
        self.trees.insert("shopkeeper".into(), shopkeeper_tree());
        self.trees.insert("quest_giver".into(), quest_giver_tree());
    }
}

impl Default for DialogueSystem {
    fn default() -> Self {
        Self::new()
    }
}

fn role_tree_key(role: NpcRole) -> String {
    match role {
        NpcRole::Villager => "villager".into(),
        NpcRole::Guard => "guard".into(),
        NpcRole::Shopkeeper => "shopkeeper".into(),
        NpcRole::QuestGiver => "quest_giver".into(),
        NpcRole::Enemy => "enemy".into(), // enemies don't talk, but key won't match
    }
}

fn villager_tree() -> DialogueTree {
    DialogueTree {
        start_node: 0,
        nodes: vec![
            // 0: greeting
            DialogueNode {
                speaker: String::new(), // filled in at runtime with NPC name
                text: "Hello, traveler! It's not often we see new faces around here.".into(),
                responses: vec![
                    DialogueResponse { text: "Tell me about this place.".into(), next_node: Some(1) },
                    DialogueResponse { text: "What era is this?".into(), next_node: Some(2) },
                    DialogueResponse { text: "Goodbye.".into(), next_node: None },
                ],
            },
            // 1: about the area
            DialogueNode {
                speaker: String::new(),
                text: "These rolling hills stretch as far as the eye can see. They say the terrain itself changes as the eras shift. Quite unsettling, really.".into(),
                responses: vec![
                    DialogueResponse { text: "Interesting. Tell me more.".into(), next_node: Some(2) },
                    DialogueResponse { text: "Thanks. Goodbye.".into(), next_node: None },
                ],
            },
            // 2: about the era
            DialogueNode {
                speaker: String::new(),
                text: "The era? Well, you can feel it in the air, can't you? The world shifts around us. Some say there are portals that let you travel through time itself.".into(),
                responses: vec![
                    DialogueResponse { text: "Where can I find these portals?".into(), next_node: Some(3) },
                    DialogueResponse { text: "Thanks for the info. Goodbye.".into(), next_node: None },
                ],
            },
            // 3: portals
            DialogueNode {
                speaker: String::new(),
                text: "Look for the glowing stones scattered across the land. Step through one and you might find yourself in a very different time. Be careful — the past and future both hold dangers.".into(),
                responses: vec![
                    DialogueResponse { text: "I'll keep my eyes open. Goodbye.".into(), next_node: None },
                ],
            },
        ],
    }
}

fn guard_tree() -> DialogueTree {
    DialogueTree {
        start_node: 0,
        nodes: vec![
            // 0: greeting
            DialogueNode {
                speaker: String::new(),
                text: "Move along, citizen. These are troubled times.".into(),
                responses: vec![
                    DialogueResponse { text: "Any threats I should know about?".into(), next_node: Some(1) },
                    DialogueResponse { text: "What are you guarding?".into(), next_node: Some(2) },
                    DialogueResponse { text: "Understood. Moving on.".into(), next_node: None },
                ],
            },
            // 1: threats
            DialogueNode {
                speaker: String::new(),
                text: "Bandits have been spotted in the outskirts. Stay alert, especially at night. They're more aggressive when the sun goes down.".into(),
                responses: vec![
                    DialogueResponse { text: "I can handle myself.".into(), next_node: None },
                    DialogueResponse { text: "Thanks for the warning.".into(), next_node: None },
                ],
            },
            // 2: duty
            DialogueNode {
                speaker: String::new(),
                text: "I patrol this area to keep the peace. Someone has to do it. If you see anything suspicious, let me know.".into(),
                responses: vec![
                    DialogueResponse { text: "Will do. Stay safe.".into(), next_node: None },
                ],
            },
        ],
    }
}

fn shopkeeper_tree() -> DialogueTree {
    DialogueTree {
        start_node: 0,
        nodes: vec![
            // 0: greeting
            DialogueNode {
                speaker: String::new(),
                text: "Welcome to my shop! I've got wares from across the eras. What catches your eye?".into(),
                responses: vec![
                    DialogueResponse { text: "What do you have for sale?".into(), next_node: Some(1) },
                    DialogueResponse { text: "How's business?".into(), next_node: Some(2) },
                    DialogueResponse { text: "Just browsing. Goodbye.".into(), next_node: None },
                ],
            },
            // 1: wares
            DialogueNode {
                speaker: String::new(),
                text: "Well, the shop system isn't quite set up yet. But between you and me, I'll have the best inventory in the realm once it is. Check back soon!".into(),
                responses: vec![
                    DialogueResponse { text: "I'll be back.".into(), next_node: None },
                ],
            },
            // 2: business
            DialogueNode {
                speaker: String::new(),
                text: "Business is... complicated when your customers keep vanishing into different eras. One minute they're here, next they're a thousand years in the past!".into(),
                responses: vec![
                    DialogueResponse { text: "Ha! I can imagine. Goodbye.".into(), next_node: None },
                ],
            },
        ],
    }
}

fn quest_giver_tree() -> DialogueTree {
    DialogueTree {
        start_node: 0,
        nodes: vec![
            // 0: greeting
            DialogueNode {
                speaker: String::new(),
                text: "Ah, you have the look of an adventurer. I could use someone with your talents...".into(),
                responses: vec![
                    DialogueResponse { text: "What do you need?".into(), next_node: Some(1) },
                    DialogueResponse { text: "I'm busy right now.".into(), next_node: None },
                ],
            },
            // 1: quest details
            DialogueNode {
                speaker: String::new(),
                text: "The quest system is still being built, but when it's ready, I'll have important missions that span across the eras themselves. The fate of the timeline may depend on it.".into(),
                responses: vec![
                    DialogueResponse { text: "Sounds exciting. I'll check back.".into(), next_node: Some(2) },
                    DialogueResponse { text: "Not interested.".into(), next_node: None },
                ],
            },
            // 2: farewell
            DialogueNode {
                speaker: String::new(),
                text: "Good. The world needs heroes who aren't afraid to walk between eras. Until we meet again, traveler.".into(),
                responses: vec![
                    DialogueResponse { text: "Farewell.".into(), next_node: None },
                ],
            },
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dialogue_lifecycle() {
        let mut system = DialogueSystem::new();
        assert!(!system.is_active());

        system.start_dialogue(NpcId(1), "Finn".into(), NpcRole::Villager);
        assert!(system.is_active());

        let node = system.current_node().unwrap();
        assert!(node.text.contains("Hello, traveler"));
        assert_eq!(node.responses.len(), 3);
    }

    #[test]
    fn test_dialogue_navigation() {
        let mut system = DialogueSystem::new();
        system.start_dialogue(NpcId(1), "Finn".into(), NpcRole::Villager);

        // Choose first response: "Tell me about this place"
        system.choose_response(0);
        assert!(system.is_active());
        let node = system.current_node().unwrap();
        assert!(node.text.contains("rolling hills"));

        // Choose "Thanks. Goodbye."
        system.choose_response(1);
        assert!(!system.is_active());
    }

    #[test]
    fn test_dialogue_end() {
        let mut system = DialogueSystem::new();
        system.start_dialogue(NpcId(1), "Finn".into(), NpcRole::Villager);
        system.end_dialogue();
        assert!(!system.is_active());
    }

    #[test]
    fn test_conversation_history() {
        let mut system = DialogueSystem::new();
        system.start_dialogue(NpcId(42), "Guard Bron".into(), NpcRole::Guard);
        assert!(system.history.talked_to.contains(&NpcId(42)));
    }

    #[test]
    fn test_all_default_trees_valid() {
        let system = DialogueSystem::new();
        for role in [NpcRole::Villager, NpcRole::Guard, NpcRole::Shopkeeper, NpcRole::QuestGiver] {
            let key = role_tree_key(role);
            let tree = system.trees.get(&key).expect(&format!("missing tree for {:?}", role));
            assert!(!tree.nodes.is_empty());
            // Verify all next_node references are valid
            for node in &tree.nodes {
                for response in &node.responses {
                    if let Some(next) = response.next_node {
                        assert!(next < tree.nodes.len(), "invalid node ref {} in tree {}", next, key);
                    }
                }
            }
        }
    }
}
