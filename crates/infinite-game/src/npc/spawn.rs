//! NPC spawn point definitions and procedural placement

use glam::Vec3;

use super::{NpcData, NpcFaction, NpcRole};

/// Defines where an NPC spawns within a chunk
#[derive(Debug, Clone)]
pub struct NpcSpawnPoint {
    /// Offset from chunk origin (x, z only — y is sampled from terrain)
    pub offset: Vec3,
    /// NPC definition to spawn
    pub data: NpcData,
    /// Only spawn in these era indices (None = all eras)
    pub era_filter: Option<Vec<usize>>,
}

/// Simple deterministic hash of a chunk coordinate to seed NPC placement
fn chunk_hash(cx: i32, cz: i32) -> u64 {
    let mut h = (cx as u64).wrapping_mul(73856093) ^ (cz as u64).wrapping_mul(19349663);
    h = h.wrapping_mul(0x517cc1b727220a95);
    h ^= h >> 32;
    h
}

/// Generate procedural spawn points for a chunk.
///
/// Uses a deterministic hash so the same chunk always produces the same NPCs.
pub fn generate_spawn_points(cx: i32, cz: i32, chunk_size: f32) -> Vec<NpcSpawnPoint> {
    let hash = chunk_hash(cx, cz);
    // 0–3 NPCs per chunk, weighted toward 0–1
    let npc_count = match hash % 8 {
        0..=3 => 0, // 50% chance of no NPCs
        4..=5 => 1, // 25% chance of 1
        6 => 2,     // 12.5%
        _ => 3,     // 12.5%
    };

    let mut points = Vec::with_capacity(npc_count as usize);
    for i in 0..npc_count {
        let sub_hash = hash.wrapping_add(i as u64 * 7919);
        let fx = ((sub_hash >> 0) & 0xFFFF) as f32 / 65535.0;
        let fz = ((sub_hash >> 16) & 0xFFFF) as f32 / 65535.0;
        let role_bits = ((sub_hash >> 32) & 0xFFFF) % 10;

        let offset_x = fx * chunk_size * 0.8 + chunk_size * 0.1;
        let offset_z = fz * chunk_size * 0.8 + chunk_size * 0.1;

        let (role, faction, wander_radius) = match role_bits {
            0..=3 => (NpcRole::Villager, NpcFaction::Friendly, 10.0),
            4..=5 => (NpcRole::Guard, NpcFaction::Friendly, 15.0),
            6 => (NpcRole::Shopkeeper, NpcFaction::Neutral, 3.0),
            7 => (NpcRole::QuestGiver, NpcFaction::Friendly, 5.0),
            _ => (NpcRole::Enemy, NpcFaction::Hostile, 20.0),
        };

        let npc_name_index = (sub_hash >> 48) % 16;
        let name = npc_name(role, npc_name_index as usize);

        let era_filter = if role == NpcRole::Enemy {
            // Enemies don't spawn in the ancient era for now
            Some(vec![1, 2, 3, 4, 5])
        } else {
            None
        };

        points.push(NpcSpawnPoint {
            offset: Vec3::new(offset_x, 0.0, offset_z),
            data: NpcData {
                name,
                role,
                faction,
                home_position: Vec3::ZERO, // set during spawn (world coords)
                wander_radius,
                interaction_radius: 3.0,
                color: role.color(),
            },
            era_filter,
        });
    }

    points
}

fn npc_name(role: NpcRole, index: usize) -> String {
    let names: &[&str] = match role {
        NpcRole::Villager => &[
            "Finn", "Elara", "Rowan", "Iris", "Aldric", "Senna",
            "Bram", "Lila", "Oswin", "Thea", "Cedric", "Mira",
            "Gareth", "Yara", "Dorian", "Vena",
        ],
        NpcRole::Guard => &[
            "Captain Bron", "Sentinel Kael", "Warden Thorne", "Guard Voss",
            "Patrol Hagen", "Watch Sera", "Guard Drex", "Shield Lynne",
            "Sentry Orsk", "Guard Pike", "Watch Farl", "Guard Brin",
            "Shield Tarn", "Guard Nyx", "Watch Rael", "Guard Siv",
        ],
        NpcRole::Shopkeeper => &[
            "Merchant Haldo", "Trader Pim", "Vendor Gris", "Seller Bea",
            "Peddler Tock", "Dealer Faye", "Hawker Rust", "Buyer Nell",
            "Broker Joss", "Vendor Skye", "Trader Opal", "Dealer Wren",
            "Seller Tane", "Vendor Mace", "Trader Glen", "Dealer Sage",
        ],
        NpcRole::QuestGiver => &[
            "Elder Morvyn", "Sage Althea", "Scholar Tobin", "Mystic Fen",
            "Oracle Rhea", "Seer Callum", "Lorekeeper Ida", "Prophet Zev",
            "Diviner Shae", "Augur Brynn", "Hermit Gale", "Sage Lorin",
            "Elder Frey", "Seer Nola", "Oracle Dane", "Wise Elm",
        ],
        NpcRole::Enemy => &[
            "Bandit", "Marauder", "Raider", "Thug",
            "Brigand", "Outlaw", "Rogue", "Cutthroat",
            "Prowler", "Scavenger", "Pillager", "Wretch",
            "Vandal", "Looter", "Brute", "Thief",
        ],
    };
    names[index % names.len()].to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spawn_deterministic() {
        let a = generate_spawn_points(5, 7, 64.0);
        let b = generate_spawn_points(5, 7, 64.0);
        assert_eq!(a.len(), b.len());
        for (sa, sb) in a.iter().zip(b.iter()) {
            assert_eq!(sa.offset, sb.offset);
            assert_eq!(sa.data.role, sb.data.role);
        }
    }

    #[test]
    fn test_spawn_offsets_in_chunk() {
        let points = generate_spawn_points(0, 0, 64.0);
        for p in &points {
            assert!(p.offset.x >= 0.0 && p.offset.x <= 64.0, "x={}", p.offset.x);
            assert!(p.offset.z >= 0.0 && p.offset.z <= 64.0, "z={}", p.offset.z);
        }
    }

    #[test]
    fn test_different_chunks_differ() {
        // Different chunks should generally produce different results
        let a = generate_spawn_points(0, 0, 64.0);
        let b = generate_spawn_points(100, 100, 64.0);
        // They could be the same by coincidence, but it's unlikely
        let same = a.len() == b.len()
            && a.iter().zip(b.iter()).all(|(x, y)| x.offset == y.offset);
        // Weak assertion: just check it doesn't crash and produces something
        assert!(a.len() <= 3);
        assert!(b.len() <= 3);
        // If they happen to be the same length, check they're not all identical
        if a.len() == b.len() && !a.is_empty() {
            assert!(!same, "different chunks should not be identical");
        }
    }

    #[test]
    fn test_era_filter_on_enemies() {
        // Generate many chunks and check any enemies have era filters
        for cx in 0..50 {
            for cz in 0..2 {
                let points = generate_spawn_points(cx, cz, 64.0);
                for p in &points {
                    if p.data.role == NpcRole::Enemy {
                        assert!(p.era_filter.is_some(), "enemies should have era filter");
                    }
                }
            }
        }
    }
}
