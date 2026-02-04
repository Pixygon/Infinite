//! Goal Oriented Action Planning (GOAP) system for NPC AI

pub mod action;
pub mod goal;
pub mod planner;
pub mod world_state;

pub use action::Action;
pub use goal::Goal;
pub use planner::GoapPlanner;
pub use world_state::{WorldFact, WorldState};

use super::NpcRole;

/// Per-NPC AI state
pub struct NpcBrain {
    pub world_state: WorldState,
    pub goals: Vec<Goal>,
    pub actions: Vec<Action>,
    pub current_plan: Option<Vec<usize>>,
    pub plan_step: usize,
    pub action_timer: f32,
    pub replan_timer: f32,
}

impl NpcBrain {
    /// Create a brain with role-specific goals and actions
    pub fn for_role(role: NpcRole) -> Self {
        let (goals, actions) = match role {
            NpcRole::Villager => Self::villager_setup(),
            NpcRole::Guard => Self::guard_setup(),
            NpcRole::Shopkeeper => Self::shopkeeper_setup(),
            NpcRole::QuestGiver => Self::quest_giver_setup(),
            NpcRole::Enemy => Self::enemy_setup(),
        };

        Self {
            world_state: WorldState::new(),
            goals,
            actions,
            current_plan: None,
            plan_step: 0,
            action_timer: 0.0,
            replan_timer: 0.0,
        }
    }

    /// Get the name of the currently executing action
    pub fn current_action_name(&self) -> Option<&str> {
        let plan = self.current_plan.as_ref()?;
        let idx = plan.get(self.plan_step)?;
        Some(&self.actions[*idx].name)
    }

    /// Advance to the next step in the current plan
    pub fn advance_plan(&mut self) {
        self.plan_step += 1;
        if let Some(plan) = &self.current_plan {
            if self.plan_step < plan.len() {
                let action_idx = plan[self.plan_step];
                self.action_timer = self.actions[action_idx].duration;
                // Apply effects of completed action
                if self.plan_step > 0 {
                    let prev_idx = plan[self.plan_step - 1];
                    self.world_state.apply(&self.actions[prev_idx].effects);
                }
            } else {
                // Plan complete
                if let Some(last) = plan.last() {
                    self.world_state.apply(&self.actions[*last].effects);
                }
                self.current_plan = None;
                self.plan_step = 0;
            }
        }
    }

    /// Select highest priority unsatisfied goal and run the planner
    pub fn replan(&mut self) {
        // Sort goals by priority (highest first)
        let mut sorted_goals: Vec<usize> = (0..self.goals.len()).collect();
        sorted_goals.sort_by(|a, b| {
            self.goals[*b].priority.partial_cmp(&self.goals[*a].priority).unwrap()
        });

        for goal_idx in sorted_goals {
            let goal = &self.goals[goal_idx];
            if self.world_state.satisfies(&goal.desired_state) {
                continue; // already satisfied
            }
            if let Some(plan) = GoapPlanner::plan(&self.world_state, goal, &self.actions) {
                if !plan.is_empty() {
                    self.action_timer = self.actions[plan[0]].duration;
                }
                self.current_plan = Some(plan);
                self.plan_step = 0;
                return;
            }
        }

        // No goal needs pursuing — default to wander
        // Find the "wander" or "patrol_point" action
        let wander_idx = self.actions.iter().position(|a| a.name == "wander" || a.name == "patrol_point");
        if let Some(idx) = wander_idx {
            self.current_plan = Some(vec![idx]);
            self.plan_step = 0;
            self.action_timer = self.actions[idx].duration;
        } else {
            self.current_plan = None;
            self.plan_step = 0;
        }
    }

    // --- Role setup factories ---

    fn villager_setup() -> (Vec<Goal>, Vec<Action>) {
        let goals = vec![
            Goal {
                name: "stay_near_home".into(),
                desired_state: WorldState::from_bool("at_home", true),
                priority: 0.3,
            },
            Goal {
                name: "wander_around".into(),
                desired_state: WorldState::from_bool("has_wandered", true),
                priority: 0.2,
            },
        ];

        let actions = vec![
            Action {
                name: "go_home".into(),
                preconditions: WorldState::new(),
                effects: WorldState::from_bool("at_home", true),
                cost: 1.0,
                duration: 5.0,
            },
            Action {
                name: "wander".into(),
                preconditions: WorldState::new(),
                effects: WorldState::from_bool("has_wandered", true),
                cost: 0.5,
                duration: 4.0,
            },
            Action {
                name: "talk_to_npc".into(),
                preconditions: WorldState::from_bool("player_nearby", true),
                effects: WorldState::from_bool("socialized", true),
                cost: 1.0,
                duration: 3.0,
            },
        ];

        (goals, actions)
    }

    fn guard_setup() -> (Vec<Goal>, Vec<Action>) {
        let goals = vec![
            Goal {
                name: "patrol".into(),
                desired_state: WorldState::from_bool("patrol_complete", true),
                priority: 0.4,
            },
            Goal {
                name: "respond_to_threat".into(),
                desired_state: WorldState::from_bool("threat_neutralized", true),
                priority: 0.8,
            },
        ];

        let actions = vec![
            Action {
                name: "patrol_point".into(),
                preconditions: WorldState::new(),
                effects: WorldState::from_bool("patrol_complete", true),
                cost: 1.0,
                duration: 6.0,
            },
            Action {
                name: "return_to_post".into(),
                preconditions: WorldState::new(),
                effects: WorldState::from_bool("at_home", true),
                cost: 1.5,
                duration: 4.0,
            },
            Action {
                name: "chase_enemy".into(),
                preconditions: WorldState::from_bool("player_in_aggro_range", true),
                effects: WorldState::from_bool("threat_neutralized", true),
                cost: 2.0,
                duration: 5.0,
            },
        ];

        (goals, actions)
    }

    fn shopkeeper_setup() -> (Vec<Goal>, Vec<Action>) {
        let goals = vec![
            Goal {
                name: "tend_shop".into(),
                desired_state: WorldState::from_bool("shop_tended", true),
                priority: 0.5,
            },
        ];

        let actions = vec![
            Action {
                name: "wait_for_customer".into(),
                preconditions: WorldState::new(),
                effects: WorldState::from_bool("shop_tended", true),
                cost: 0.5,
                duration: 8.0,
            },
            Action {
                name: "wander".into(),
                preconditions: WorldState::new(),
                effects: WorldState::from_bool("has_wandered", true),
                cost: 1.0,
                duration: 3.0,
            },
        ];

        (goals, actions)
    }

    fn quest_giver_setup() -> (Vec<Goal>, Vec<Action>) {
        let goals = vec![
            Goal {
                name: "wait_for_hero".into(),
                desired_state: WorldState::from_bool("waiting", true),
                priority: 0.5,
            },
        ];

        let actions = vec![
            Action {
                name: "wait".into(),
                preconditions: WorldState::new(),
                effects: WorldState::from_bool("waiting", true),
                cost: 0.5,
                duration: 5.0,
            },
            Action {
                name: "wander".into(),
                preconditions: WorldState::new(),
                effects: WorldState::from_bool("has_wandered", true),
                cost: 1.0,
                duration: 4.0,
            },
        ];

        (goals, actions)
    }

    fn enemy_setup() -> (Vec<Goal>, Vec<Action>) {
        let goals = vec![
            Goal {
                name: "patrol_area".into(),
                desired_state: WorldState::from_bool("patrol_complete", true),
                priority: 0.3,
            },
            Goal {
                name: "chase_player".into(),
                desired_state: WorldState::from_bool("player_in_attack_range", true),
                priority: 0.8,
            },
            Goal {
                name: "attack_player".into(),
                desired_state: WorldState::from_bool("player_damaged", true),
                priority: 0.9,
            },
            Goal {
                name: "flee".into(),
                desired_state: WorldState::from_bool("is_safe", true),
                priority: 1.0,
            },
        ];

        let actions = vec![
            Action {
                name: "patrol_point".into(),
                preconditions: WorldState::new(),
                effects: WorldState::from_bool("patrol_complete", true),
                cost: 1.0,
                duration: 5.0,
            },
            Action {
                name: "chase_target".into(),
                preconditions: WorldState::from_bool("player_in_aggro_range", true),
                effects: WorldState::from_bool("player_in_attack_range", true),
                cost: 1.0,
                duration: 3.0,
            },
            Action {
                name: "attack_melee".into(),
                preconditions: WorldState::from_bool("player_in_attack_range", true),
                effects: WorldState::from_bool("player_damaged", true),
                cost: 0.5,
                duration: 1.0,
            },
            Action {
                name: "flee_from_target".into(),
                preconditions: WorldState::from_bool("health_low", true),
                effects: WorldState::from_bool("is_safe", true),
                cost: 0.5,
                duration: 5.0,
            },
        ];

        (goals, actions)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_brain_for_each_role() {
        for role in [NpcRole::Villager, NpcRole::Guard, NpcRole::Shopkeeper, NpcRole::QuestGiver, NpcRole::Enemy] {
            let brain = NpcBrain::for_role(role);
            assert!(!brain.goals.is_empty(), "{:?} should have goals", role);
            assert!(!brain.actions.is_empty(), "{:?} should have actions", role);
        }
    }

    #[test]
    fn test_replan_produces_plan() {
        let mut brain = NpcBrain::for_role(NpcRole::Villager);
        brain.replan();
        assert!(brain.current_plan.is_some());
    }

    #[test]
    fn test_advance_plan_completes() {
        let mut brain = NpcBrain::for_role(NpcRole::Villager);
        brain.replan();
        // Advance through the entire plan
        for _ in 0..10 {
            if brain.current_plan.is_some() {
                brain.advance_plan();
            }
        }
        // Should eventually complete
    }
}
