//! Maps NPC roles and time periods to character archetypes and system prompts

use super::NpcRole;

/// Get an archetype key for an NPC role at a given year
pub fn archetype_for(role: NpcRole, year: i64) -> &'static str {
    match (role, year) {
        // Past (<1900): medieval/ancient themed
        (NpcRole::Villager, y) if y < 1900 => "medieval_peasant",
        (NpcRole::Guard, y) if y < 1900 => "medieval_knight",
        (NpcRole::Shopkeeper, y) if y < 1900 => "medieval_merchant",
        (NpcRole::QuestGiver, y) if y < 1900 => "medieval_sage",
        (NpcRole::Enemy, y) if y < 1900 => "medieval_bandit",

        // Present (1900-2100): modern themed
        (NpcRole::Villager, y) if y <= 2100 => "modern_citizen",
        (NpcRole::Guard, y) if y <= 2100 => "modern_officer",
        (NpcRole::Shopkeeper, y) if y <= 2100 => "modern_vendor",
        (NpcRole::QuestGiver, y) if y <= 2100 => "modern_scholar",
        (NpcRole::Enemy, y) if y <= 2100 => "modern_criminal",

        // Future (>2100): sci-fi themed
        (NpcRole::Villager, _) => "future_settler",
        (NpcRole::Guard, _) => "future_enforcer",
        (NpcRole::Shopkeeper, _) => "future_trader",
        (NpcRole::QuestGiver, _) => "future_oracle",
        (NpcRole::Enemy, _) => "future_raider",
    }
}

/// Generate a system prompt for an NPC based on their role and era
pub fn generate_system_prompt(name: &str, role: NpcRole, year: i64) -> String {
    let archetype = archetype_for(role, year);
    let era_context = era_context(year);
    let role_personality = role_personality(role);

    format!(
        "You are {name}, a {archetype_desc} in a world where time travel exists.\n\
         {era_context}\n\
         {role_personality}\n\n\
         RULES:\n\
         - Stay in character at all times\n\
         - Keep responses under 3 sentences\n\
         - React naturally to the player's words\n\
         - Reference the current era and setting when relevant\n\
         - Never break the fourth wall or mention being an AI",
        name = name,
        archetype_desc = archetype_description(archetype),
    )
}

fn archetype_description(archetype: &str) -> &'static str {
    match archetype {
        "medieval_peasant" => "simple village dweller in a medieval settlement",
        "medieval_knight" => "armored guard sworn to protect the realm",
        "medieval_merchant" => "traveling merchant dealing in rare goods",
        "medieval_sage" => "wise elder who studies the mysteries of time",
        "medieval_bandit" => "desperate outlaw living on the fringes",
        "modern_citizen" => "everyday person living in the modern world",
        "modern_officer" => "law enforcement officer keeping the peace",
        "modern_vendor" => "shopkeeper running a small business",
        "modern_scholar" => "researcher studying temporal anomalies",
        "modern_criminal" => "dangerous criminal lurking in the shadows",
        "future_settler" => "pioneer in a strange new era",
        "future_enforcer" => "cybernetic peace officer of the future",
        "future_trader" => "interstellar merchant dealing in exotic wares",
        "future_oracle" => "enigmatic seer who perceives the flow of time",
        "future_raider" => "tech-enhanced scavenger preying on travelers",
        _ => "mysterious figure",
    }
}

fn era_context(year: i64) -> String {
    match year {
        y if y < -3000 => "The world is primal and untamed. Civilization has barely begun.".into(),
        y if y < 0 => format!("The year is roughly {} BCE. Ancient civilizations dot the landscape.", -y),
        y if y < 500 => format!("The year is {} CE. The classical world thrives with philosophy and conquest.", y),
        y if y < 1500 => format!("The year is {}. Castles and feudal lords rule these lands.", y),
        y if y < 1800 => format!("The year is {}. The age of exploration and enlightenment unfolds.", y),
        y if y < 1950 => format!("The year is {}. Industry transforms the world at a rapid pace.", y),
        y if y < 2100 => format!("The year is {}. Technology advances while old traditions persist.", y),
        y => format!("The year is {}. The future is strange and wondrous.", y),
    }
}

fn role_personality(role: NpcRole) -> &'static str {
    match role {
        NpcRole::Villager => "You are friendly and curious about travelers. You gossip about local events and worry about the strange temporal shifts.",
        NpcRole::Guard => "You are dutiful and alert. You take your role seriously and warn travelers of dangers.",
        NpcRole::Shopkeeper => "You are entrepreneurial and cheerful. You enjoy haggling and always have something interesting to offer.",
        NpcRole::QuestGiver => "You are wise and mysterious. You sense the player has a greater destiny and offer guidance.",
        NpcRole::Enemy => "You are hostile and territorial. You threaten intruders and demand tribute.",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_archetype_mapping_eras() {
        assert_eq!(archetype_for(NpcRole::Villager, 500), "medieval_peasant");
        assert_eq!(archetype_for(NpcRole::Villager, 2025), "modern_citizen");
        assert_eq!(archetype_for(NpcRole::Villager, 3000), "future_settler");

        assert_eq!(archetype_for(NpcRole::Guard, 1200), "medieval_knight");
        assert_eq!(archetype_for(NpcRole::Guard, 2000), "modern_officer");
        assert_eq!(archetype_for(NpcRole::Guard, 2500), "future_enforcer");
    }

    #[test]
    fn test_system_prompt_non_empty() {
        let prompt = generate_system_prompt("Elder Morvyn", NpcRole::QuestGiver, 1200);
        assert!(!prompt.is_empty());
        assert!(prompt.contains("Elder Morvyn"));
        assert!(prompt.contains("wise"));
    }

    #[test]
    fn test_all_roles_all_eras() {
        let roles = [NpcRole::Villager, NpcRole::Guard, NpcRole::Shopkeeper, NpcRole::QuestGiver, NpcRole::Enemy];
        let years = [-5000, -500, 200, 1200, 1700, 1925, 2025, 3000];

        for role in &roles {
            for year in &years {
                let archetype = archetype_for(*role, *year);
                assert!(!archetype.is_empty());
                let prompt = generate_system_prompt("Test", *role, *year);
                assert!(!prompt.is_empty());
            }
        }
    }
}
