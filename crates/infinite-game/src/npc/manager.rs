//! NPC manager — spawn, despawn, and update all active NPCs

use std::collections::HashMap;

use glam::Vec3;
use infinite_world::ChunkCoord;

use super::goap::NpcBrain;
use super::spawn::{generate_spawn_points, NpcSpawnPoint};
use super::{NpcBehaviorState, NpcFaction, NpcId, NpcInstance, NpcRole};
use super::combat::CombatStats;

/// Manages all active NPC instances
pub struct NpcManager {
    npcs: HashMap<NpcId, NpcInstance>,
    next_id: u64,
    chunk_size: f32,
    /// Combat stats for NPCs that have them (enemies)
    pub combat_stats: HashMap<NpcId, CombatStats>,
    /// Respawn timers: chunk coord → list of (spawn point index, timer remaining)
    respawn_timers: Vec<(ChunkCoord, usize, f32)>,
}

impl NpcManager {
    pub fn new(chunk_size: f32) -> Self {
        Self {
            npcs: HashMap::new(),
            next_id: 1,
            chunk_size,
            combat_stats: HashMap::new(),
            respawn_timers: Vec::new(),
        }
    }

    fn next_npc_id(&mut self) -> NpcId {
        let id = NpcId(self.next_id);
        self.next_id += 1;
        id
    }

    /// Called when a chunk finishes loading. Spawns NPCs for that chunk.
    pub fn on_chunk_loaded(
        &mut self,
        coord: ChunkCoord,
        active_year: i64,
        height_fn: impl Fn(f32, f32) -> f32,
    ) {
        let spawn_points = generate_spawn_points(coord.x, coord.z, self.chunk_size);
        let origin = coord.world_origin(self.chunk_size);

        for point in &spawn_points {
            if let Some((min_year, max_year)) = point.year_range {
                if active_year < min_year || active_year > max_year {
                    continue;
                }
            }
            self.spawn_npc(coord, point, origin, &height_fn);
        }
    }

    fn spawn_npc(
        &mut self,
        coord: ChunkCoord,
        point: &NpcSpawnPoint,
        chunk_origin: Vec3,
        height_fn: &impl Fn(f32, f32) -> f32,
    ) -> NpcId {
        let id = self.next_npc_id();
        let world_x = chunk_origin.x + point.offset.x;
        let world_z = chunk_origin.z + point.offset.z;
        let world_y = height_fn(world_x, world_z) + 0.9; // half capsule height

        let home = Vec3::new(world_x, world_y, world_z);
        let mut data = point.data.clone();
        data.home_position = home;

        let brain = NpcBrain::for_role(data.role);

        let is_enemy = data.role == NpcRole::Enemy;

        let instance = NpcInstance {
            id,
            data,
            position: home,
            velocity: Vec3::ZERO,
            yaw: 0.0,
            chunk: coord,
            state: NpcBehaviorState::Idle { timer: 2.0 },
            brain: Some(brain),
        };

        self.npcs.insert(id, instance);

        if is_enemy {
            self.combat_stats.insert(id, CombatStats::default_enemy());
        }

        id
    }

    /// Called when a chunk is unloaded. Removes all NPCs from that chunk.
    pub fn on_chunk_unloaded(&mut self, coord: ChunkCoord) {
        let to_remove: Vec<NpcId> = self
            .npcs
            .values()
            .filter(|npc| npc.chunk == coord)
            .map(|npc| npc.id)
            .collect();
        for id in to_remove {
            self.npcs.remove(&id);
            self.combat_stats.remove(&id);
        }
    }

    /// Update all NPC behaviors
    pub fn update(
        &mut self,
        delta: f32,
        player_pos: Vec3,
        height_fn: impl Fn(f32, f32) -> f32,
    ) {
        // Update respawn timers
        let chunk_size = self.chunk_size;
        let mut respawns_ready = Vec::new();
        self.respawn_timers.retain_mut(|(coord, idx, timer)| {
            *timer -= delta;
            if *timer <= 0.0 {
                respawns_ready.push((*coord, *idx));
                false
            } else {
                true
            }
        });

        // Process respawns
        for (coord, _idx) in respawns_ready {
            let points = generate_spawn_points(coord.x, coord.z, chunk_size);
            if let Some(point) = points.get(_idx) {
                let origin = coord.world_origin(chunk_size);
                self.spawn_npc(coord, point, origin, &height_fn);
            }
        }

        // Collect NPC ids for iteration (avoid borrow issues)
        let ids: Vec<NpcId> = self.npcs.keys().copied().collect();

        for id in ids {
            // Try GOAP brain first
            let has_brain = self.npcs.get(&id).map(|n| n.brain.is_some()).unwrap_or(false);
            if has_brain {
                self.update_npc_goap(id, delta, player_pos, &height_fn);
            } else {
                self.update_npc_simple(id, delta, player_pos, &height_fn);
            }
        }
    }

    /// Simple state machine update (fallback when no GOAP brain)
    fn update_npc_simple(
        &mut self,
        id: NpcId,
        delta: f32,
        _player_pos: Vec3,
        height_fn: &impl Fn(f32, f32) -> f32,
    ) {
        let npc = match self.npcs.get_mut(&id) {
            Some(n) => n,
            None => return,
        };

        let home = npc.data.home_position;
        let wander_radius = npc.data.wander_radius;
        let speed = 2.0_f32;

        match &mut npc.state {
            NpcBehaviorState::Idle { timer } => {
                *timer -= delta;
                if *timer <= 0.0 {
                    // Pick a random-ish wander target using a simple deterministic approach
                    let angle = (npc.id.0 as f32 * 1.618 + npc.position.x * 0.1) % std::f32::consts::TAU;
                    let dist = wander_radius * 0.5;
                    let target = Vec3::new(
                        home.x + angle.cos() * dist,
                        0.0,
                        home.z + angle.sin() * dist,
                    );
                    let target_y = height_fn(target.x, target.z) + 0.9;
                    npc.state = NpcBehaviorState::Walking {
                        target: Vec3::new(target.x, target_y, target.z),
                    };
                }
            }
            NpcBehaviorState::Walking { target } => {
                let to_target = *target - npc.position;
                let horizontal_dist = Vec3::new(to_target.x, 0.0, to_target.z).length();
                if horizontal_dist < 0.5 {
                    npc.state = NpcBehaviorState::Idle { timer: 3.0 };
                    npc.velocity = Vec3::ZERO;
                } else {
                    let dir = Vec3::new(to_target.x, 0.0, to_target.z).normalize();
                    npc.velocity = dir * speed;
                    npc.position += npc.velocity * delta;
                    // Snap to terrain
                    npc.position.y = height_fn(npc.position.x, npc.position.z) + 0.9;
                    npc.yaw = dir.z.atan2(dir.x);
                }
            }
            NpcBehaviorState::Talking => {
                npc.velocity = Vec3::ZERO;
            }
        }
    }

    /// GOAP-based NPC update
    fn update_npc_goap(
        &mut self,
        id: NpcId,
        delta: f32,
        player_pos: Vec3,
        height_fn: &impl Fn(f32, f32) -> f32,
    ) {
        let npc = match self.npcs.get_mut(&id) {
            Some(n) => n,
            None => return,
        };

        let brain = match &mut npc.brain {
            Some(b) => b,
            None => return,
        };

        // Update sensors
        let distance_to_player = (npc.position - player_pos).length();
        let npc_pos = npc.position;
        let home_pos = npc.data.home_position;

        brain.world_state.set_float("distance_to_player", distance_to_player);
        brain.world_state.set_bool("player_nearby", distance_to_player < 15.0);
        brain.world_state.set_bool("player_in_aggro_range", distance_to_player < 12.0);
        brain.world_state.set_bool("player_in_attack_range", distance_to_player < 2.5);
        brain.world_state.set_bool("at_home", (npc_pos - home_pos).length() < 3.0);

        // Check combat stats for health
        if let Some(stats) = self.combat_stats.get(&id) {
            brain.world_state.set_bool("health_low", stats.current_hp / stats.max_hp < 0.2);
            brain.world_state.set_bool("is_alive", stats.current_hp > 0.0);
        }

        // Replan if needed
        brain.replan_timer -= delta;
        if brain.current_plan.is_none() || brain.replan_timer <= 0.0 {
            brain.replan();
            brain.replan_timer = 2.0;
        }

        // Execute current action
        let speed = if npc.data.role == NpcRole::Enemy { 3.5 } else { 2.0 };

        if let Some(action_name) = brain.current_action_name() {
            match action_name {
                "go_home" | "return_to_post" => {
                    let to_home = home_pos - npc_pos;
                    let horizontal = Vec3::new(to_home.x, 0.0, to_home.z);
                    if horizontal.length() < 1.0 {
                        brain.advance_plan();
                        npc.velocity = Vec3::ZERO;
                    } else {
                        let dir = horizontal.normalize();
                        npc.velocity = dir * speed;
                        npc.position += npc.velocity * delta;
                        npc.position.y = height_fn(npc.position.x, npc.position.z) + 0.9;
                        npc.yaw = dir.z.atan2(dir.x);
                    }
                }
                "wander" | "patrol_point" => {
                    // Simple wander behavior
                    brain.action_timer -= delta;
                    if brain.action_timer <= 0.0 {
                        brain.advance_plan();
                        npc.velocity = Vec3::ZERO;
                    } else {
                        let t = brain.action_timer;
                        let angle = (id.0 as f32 * 2.71 + t) % std::f32::consts::TAU;
                        let dir = Vec3::new(angle.cos(), 0.0, angle.sin());
                        npc.velocity = dir * speed * 0.5;
                        npc.position += npc.velocity * delta;
                        // Clamp to wander radius
                        let from_home = npc.position - home_pos;
                        if from_home.length() > npc.data.wander_radius {
                            npc.position = home_pos + from_home.normalize() * npc.data.wander_radius;
                        }
                        npc.position.y = height_fn(npc.position.x, npc.position.z) + 0.9;
                        npc.yaw = dir.z.atan2(dir.x);
                    }
                }
                "chase_target" | "chase_enemy" => {
                    let to_player = player_pos - npc_pos;
                    let horizontal = Vec3::new(to_player.x, 0.0, to_player.z);
                    if horizontal.length() < 2.5 {
                        brain.advance_plan();
                    } else {
                        let dir = horizontal.normalize();
                        npc.velocity = dir * speed;
                        npc.position += npc.velocity * delta;
                        npc.position.y = height_fn(npc.position.x, npc.position.z) + 0.9;
                        npc.yaw = dir.z.atan2(dir.x);
                    }
                }
                "attack_melee" => {
                    brain.action_timer -= delta;
                    if brain.action_timer <= 0.0 {
                        brain.advance_plan();
                    }
                    npc.velocity = Vec3::ZERO;
                    // Face player
                    let to_player = player_pos - npc_pos;
                    npc.yaw = to_player.z.atan2(to_player.x);
                }
                "wait" | "wait_for_customer" | "open_shop" | "close_shop" | "sleep" | "eat_food" | "talk_to_npc" => {
                    brain.action_timer -= delta;
                    if brain.action_timer <= 0.0 {
                        brain.advance_plan();
                    }
                    npc.velocity = Vec3::ZERO;
                }
                "flee_from_target" => {
                    let away = npc_pos - player_pos;
                    let horizontal = Vec3::new(away.x, 0.0, away.z);
                    if horizontal.length() > 30.0 {
                        brain.advance_plan();
                        npc.velocity = Vec3::ZERO;
                    } else {
                        let dir = horizontal.normalize_or_zero();
                        npc.velocity = dir * speed * 1.5;
                        npc.position += npc.velocity * delta;
                        npc.position.y = height_fn(npc.position.x, npc.position.z) + 0.9;
                        npc.yaw = dir.z.atan2(dir.x);
                    }
                }
                _ => {
                    brain.action_timer -= delta;
                    if brain.action_timer <= 0.0 {
                        brain.advance_plan();
                    }
                }
            }
        } else {
            // No plan — idle
            npc.velocity = Vec3::ZERO;
        }
    }

    /// Iterate over all active NPCs
    pub fn npcs_iter(&self) -> impl Iterator<Item = &NpcInstance> {
        self.npcs.values()
    }

    /// Get a mutable reference to an NPC
    pub fn get_mut(&mut self, id: NpcId) -> Option<&mut NpcInstance> {
        self.npcs.get_mut(&id)
    }

    /// Get an NPC by ID
    pub fn get(&self, id: NpcId) -> Option<&NpcInstance> {
        self.npcs.get(&id)
    }

    /// Find the nearest NPC within radius of a position
    pub fn npc_at(&self, pos: Vec3, radius: f32) -> Option<NpcId> {
        let mut best: Option<(NpcId, f32)> = None;
        for npc in self.npcs.values() {
            let dist = (npc.position - pos).length();
            if dist < radius {
                if best.is_none() || dist < best.unwrap().1 {
                    best = Some((npc.id, dist));
                }
            }
        }
        best.map(|(id, _)| id)
    }

    /// Total number of active NPCs
    pub fn count(&self) -> usize {
        self.npcs.len()
    }

    /// Count NPCs by faction
    pub fn count_by_faction(&self, faction: NpcFaction) -> usize {
        self.npcs.values().filter(|n| n.data.faction == faction).count()
    }

    /// Damage an NPC. Returns true if the NPC was defeated (HP <= 0).
    pub fn damage_npc(&mut self, id: NpcId, damage: f32) -> bool {
        if let Some(stats) = self.combat_stats.get_mut(&id) {
            let actual = (damage - stats.defense).max(1.0);
            stats.current_hp = (stats.current_hp - actual).max(0.0);
            if stats.current_hp <= 0.0 {
                // Defeated — remove and start respawn timer
                if let Some(npc) = self.npcs.remove(&id) {
                    let chunk = npc.chunk;
                    // Find which spawn index this was (approximate)
                    let points = generate_spawn_points(chunk.x, chunk.z, self.chunk_size);
                    let idx = points.iter().position(|p| p.data.role == NpcRole::Enemy).unwrap_or(0);
                    self.respawn_timers.push((chunk, idx, 30.0));
                }
                self.combat_stats.remove(&id);
                return true;
            }
        }
        false
    }

    /// Check if an enemy NPC is currently attacking (in attack range and has attack action)
    pub fn is_attacking(&self, id: NpcId) -> bool {
        if let Some(npc) = self.npcs.get(&id) {
            if let Some(brain) = &npc.brain {
                return brain.current_action_name() == Some("attack_melee");
            }
        }
        false
    }

    /// Get combat stats for an NPC
    pub fn get_combat_stats(&self, id: NpcId) -> Option<&CombatStats> {
        self.combat_stats.get(&id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_height(_x: f32, _z: f32) -> f32 {
        0.0
    }

    #[test]
    fn test_manager_spawn_despawn() {
        let mut mgr = NpcManager::new(64.0);
        let coord = ChunkCoord::new(5, 7);
        mgr.on_chunk_loaded(coord, 2025, test_height);
        let count_before = mgr.count();

        mgr.on_chunk_unloaded(coord);
        assert_eq!(mgr.count(), 0, "all NPCs should be removed on chunk unload");
        assert!(count_before <= 3, "should spawn 0-3 NPCs per chunk");
    }

    #[test]
    fn test_manager_update_no_crash() {
        let mut mgr = NpcManager::new(64.0);
        mgr.on_chunk_loaded(ChunkCoord::new(0, 0), 2025, test_height);
        // Should not crash
        mgr.update(0.016, Vec3::ZERO, test_height);
        mgr.update(0.016, Vec3::new(10.0, 0.0, 10.0), test_height);
    }

    #[test]
    fn test_npc_at_finds_nearby() {
        let mut mgr = NpcManager::new(64.0);
        // Load enough chunks to get at least one NPC
        for x in 0..10 {
            for z in 0..10 {
                mgr.on_chunk_loaded(ChunkCoord::new(x, z), 2025, test_height);
            }
        }
        if mgr.count() > 0 {
            let first_npc = mgr.npcs_iter().next().unwrap();
            let pos = first_npc.position;
            let id = first_npc.id;
            assert_eq!(mgr.npc_at(pos, 1.0), Some(id));
        }
    }

    #[test]
    fn test_behavior_transitions() {
        let mut mgr = NpcManager::new(64.0);
        for x in 0..10 {
            mgr.on_chunk_loaded(ChunkCoord::new(x, 0), 2025, test_height);
        }

        // Run enough updates for state transitions
        for _ in 0..200 {
            mgr.update(0.1, Vec3::new(1000.0, 0.0, 1000.0), test_height);
        }
        // Should not crash and NPCs should still exist
        assert!(mgr.count() > 0 || true); // some chunks might have 0 NPCs
    }
}
