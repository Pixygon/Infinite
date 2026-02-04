//! A* GOAP planner — searches action space to find a plan that reaches a goal

use std::collections::BinaryHeap;
use std::cmp::Ordering;

use super::action::Action;
use super::goal::Goal;
use super::world_state::WorldState;

/// A node in the A* search
#[derive(Debug)]
struct PlanNode {
    /// World state after applying actions so far
    state: WorldState,
    /// Indices of actions taken to reach this state
    actions: Vec<usize>,
    /// Actual cost so far (g)
    cost: f32,
    /// Estimated total cost (f = g + h)
    estimated_total: f32,
}

impl PartialEq for PlanNode {
    fn eq(&self, other: &Self) -> bool {
        self.estimated_total == other.estimated_total
    }
}
impl Eq for PlanNode {}

impl PartialOrd for PlanNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PlanNode {
    fn cmp(&self, other: &Self) -> Ordering {
        // Reverse order for min-heap (BinaryHeap is a max-heap)
        other.estimated_total.partial_cmp(&self.estimated_total)
            .unwrap_or(Ordering::Equal)
    }
}

pub struct GoapPlanner;

impl GoapPlanner {
    /// Find a sequence of actions that transforms `current_state` into one
    /// that satisfies the goal's `desired_state`.
    ///
    /// Returns action indices into `available_actions`, or None if impossible.
    pub fn plan(
        current_state: &WorldState,
        goal: &Goal,
        available_actions: &[Action],
    ) -> Option<Vec<usize>> {
        if current_state.satisfies(&goal.desired_state) {
            return Some(Vec::new()); // already satisfied
        }

        let mut open = BinaryHeap::new();
        let h = current_state.unsatisfied_count(&goal.desired_state) as f32;

        open.push(PlanNode {
            state: current_state.clone(),
            actions: Vec::new(),
            cost: 0.0,
            estimated_total: h,
        });

        let max_iterations = 100;
        let mut iterations = 0;

        while let Some(node) = open.pop() {
            iterations += 1;
            if iterations > max_iterations {
                break;
            }

            // Check if goal reached
            if node.state.satisfies(&goal.desired_state) {
                return Some(node.actions);
            }

            // Try each available action
            for (i, action) in available_actions.iter().enumerate() {
                // Check preconditions
                if !node.state.satisfies(&action.preconditions) {
                    continue;
                }

                // Avoid repeating the same action consecutively
                if node.actions.last() == Some(&i) {
                    continue;
                }

                // Apply effects
                let mut new_state = node.state.clone();
                new_state.apply(&action.effects);

                let new_cost = node.cost + action.cost;
                let h = new_state.unsatisfied_count(&goal.desired_state) as f32;
                let mut new_actions = node.actions.clone();
                new_actions.push(i);

                // Limit plan length
                if new_actions.len() > 5 {
                    continue;
                }

                open.push(PlanNode {
                    state: new_state,
                    actions: new_actions,
                    cost: new_cost,
                    estimated_total: new_cost + h,
                });
            }
        }

        None // no plan found
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plan_already_satisfied() {
        let mut state = WorldState::new();
        state.set_bool("at_home", true);

        let goal = Goal {
            name: "go_home".into(),
            desired_state: WorldState::from_bool("at_home", true),
            priority: 1.0,
        };

        let plan = GoapPlanner::plan(&state, &goal, &[]).unwrap();
        assert!(plan.is_empty());
    }

    #[test]
    fn test_plan_single_action() {
        let state = WorldState::new();
        let goal = Goal {
            name: "go_home".into(),
            desired_state: WorldState::from_bool("at_home", true),
            priority: 1.0,
        };

        let actions = vec![
            Action {
                name: "walk_home".into(),
                preconditions: WorldState::new(),
                effects: WorldState::from_bool("at_home", true),
                cost: 1.0,
                duration: 5.0,
            },
        ];

        let plan = GoapPlanner::plan(&state, &goal, &actions).unwrap();
        assert_eq!(plan, vec![0]);
    }

    #[test]
    fn test_plan_chain() {
        let state = WorldState::new();
        let goal = Goal {
            name: "eat".into(),
            desired_state: WorldState::from_bool("full", true),
            priority: 1.0,
        };

        let actions = vec![
            Action {
                name: "find_food".into(),
                preconditions: WorldState::new(),
                effects: WorldState::from_bool("has_food", true),
                cost: 2.0,
                duration: 3.0,
            },
            Action {
                name: "eat_food".into(),
                preconditions: WorldState::from_bool("has_food", true),
                effects: WorldState::from_bool("full", true),
                cost: 1.0,
                duration: 2.0,
            },
        ];

        let plan = GoapPlanner::plan(&state, &goal, &actions).unwrap();
        assert_eq!(plan, vec![0, 1]); // find_food then eat_food
    }

    #[test]
    fn test_plan_impossible() {
        let state = WorldState::new();
        let goal = Goal {
            name: "fly".into(),
            desired_state: WorldState::from_bool("flying", true),
            priority: 1.0,
        };

        // Only action requires something we can never get
        let actions = vec![
            Action {
                name: "flap_wings".into(),
                preconditions: WorldState::from_bool("has_wings", true),
                effects: WorldState::from_bool("flying", true),
                cost: 1.0,
                duration: 1.0,
            },
        ];

        assert!(GoapPlanner::plan(&state, &goal, &actions).is_none());
    }

    #[test]
    fn test_plan_prefers_lower_cost() {
        let state = WorldState::new();
        let goal = Goal {
            name: "go_home".into(),
            desired_state: WorldState::from_bool("at_home", true),
            priority: 1.0,
        };

        let actions = vec![
            Action {
                name: "walk_home_slowly".into(),
                preconditions: WorldState::new(),
                effects: WorldState::from_bool("at_home", true),
                cost: 5.0,
                duration: 10.0,
            },
            Action {
                name: "run_home".into(),
                preconditions: WorldState::new(),
                effects: WorldState::from_bool("at_home", true),
                cost: 1.0,
                duration: 3.0,
            },
        ];

        let plan = GoapPlanner::plan(&state, &goal, &actions).unwrap();
        assert_eq!(plan, vec![1]); // should pick lower cost action
    }
}
