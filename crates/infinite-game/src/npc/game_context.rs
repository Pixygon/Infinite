//! Game context injected into AI system prompts for NPC dialogue

/// Current game state relevant to NPC conversations
pub struct GameContext {
    pub active_year: i64,
    pub time_of_day: f32,
    pub weather: String,
    pub player_name: String,
    pub npc_goap_state: String,
    pub npc_location_desc: String,
    pub relationship_level: f32,
    pub relationship_tier: String,
    pub conversation_summary: Option<String>,
}

impl GameContext {
    /// Format the game context as a system prompt section
    pub fn to_system_context(&self) -> String {
        let era_desc = match self.active_year {
            y if y < -3000 => "Prehistoric Era",
            y if y < 0 => "Ancient Era",
            y if y < 500 => "Classical Era",
            y if y < 1500 => "Medieval Era",
            y if y < 1800 => "Early Modern Era",
            y if y < 1950 => "Industrial Era",
            y if y < 2100 => "Modern Era",
            _ => "Future Era",
        };

        let time_desc = match self.time_of_day as u32 {
            0..=5 => "Night",
            6..=11 => "Morning",
            12..=17 => "Afternoon",
            _ => "Evening",
        };

        let mut context = format!(
            "[GAME CONTEXT]\n\
             Year: {} ({})\n\
             Time: {} ({:.0}:00)\n\
             Weather: {}\n\
             Player: {}\n\
             NPC Activity: {}\n\
             Location: {}\n\
             Relationship: {} ({:.0}/100)",
            self.active_year,
            era_desc,
            time_desc,
            self.time_of_day,
            self.weather,
            self.player_name,
            self.npc_goap_state,
            self.npc_location_desc,
            self.relationship_tier,
            self.relationship_level,
        );

        if let Some(summary) = &self.conversation_summary {
            context.push_str(&format!(
                "\n\n[PREVIOUS CONVERSATION SUMMARY]\n{}",
                summary
            ));
        }

        context
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_system_context_produces_valid_string() {
        let ctx = GameContext {
            active_year: 2025,
            time_of_day: 14.5,
            weather: "Clear".into(),
            player_name: "TestPlayer".into(),
            npc_goap_state: "idle".into(),
            npc_location_desc: "village square".into(),
            relationship_level: 25.0,
            relationship_tier: "Acquaintance".into(),
            conversation_summary: None,
        };

        let result = ctx.to_system_context();
        assert!(result.contains("2025"));
        assert!(result.contains("Modern Era"));
        assert!(result.contains("Afternoon"));
        assert!(result.contains("TestPlayer"));
        assert!(result.contains("Acquaintance"));
    }

    #[test]
    fn test_context_with_conversation_summary() {
        let ctx = GameContext {
            active_year: -500,
            time_of_day: 8.0,
            weather: "Rainy".into(),
            player_name: "Hero".into(),
            npc_goap_state: "patrolling".into(),
            npc_location_desc: "city walls".into(),
            relationship_level: 50.0,
            relationship_tier: "Friend".into(),
            conversation_summary: Some("Previously discussed the coming war.".into()),
        };

        let result = ctx.to_system_context();
        assert!(result.contains("Ancient Era"));
        assert!(result.contains("PREVIOUS CONVERSATION SUMMARY"));
        assert!(result.contains("coming war"));
    }

    #[test]
    fn test_era_descriptions() {
        let eras = vec![
            (-5000, "Prehistoric"),
            (-1000, "Ancient"),
            (200, "Classical"),
            (1200, "Medieval"),
            (1700, "Early Modern"),
            (1900, "Industrial"),
            (2025, "Modern"),
            (3000, "Future"),
        ];

        for (year, expected) in eras {
            let ctx = GameContext {
                active_year: year,
                time_of_day: 12.0,
                weather: "Clear".into(),
                player_name: "Test".into(),
                npc_goap_state: "idle".into(),
                npc_location_desc: "here".into(),
                relationship_level: 0.0,
                relationship_tier: "Stranger".into(),
                conversation_summary: None,
            };
            let result = ctx.to_system_context();
            assert!(result.contains(expected), "Year {} should map to era containing '{}', got: {}", year, expected, result);
        }
    }
}
