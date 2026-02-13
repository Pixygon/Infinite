//! Character data structures and persistence
//!
//! Defines all character customization options and save/load functionality.

mod persistence;

pub use persistence::save_character;

use chrono::{DateTime, Utc};
use infinite_game::combat::element::Element;
use infinite_game::combat::weapon::WeaponType;
use infinite_game::player::stats::{CharacterStats, StatGrowth};
use serde::{Deserialize, Serialize};

/// Biological sex selection for character base body type
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Sex {
    #[default]
    Male,
    Female,
}

impl Sex {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Male => "Male",
            Self::Female => "Female",
        }
    }
}

/// Complete character data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterData {
    /// Unique character identifier
    pub id: String,
    /// Character name
    pub name: String,
    /// Biological sex (base body type)
    pub sex: Sex,
    /// Character archetype/class (None until chosen in gameplay)
    pub archetype: Option<Archetype>,
    /// Visual appearance
    pub appearance: CharacterAppearance,
    /// When the character was created
    pub created_at: DateTime<Utc>,
    /// Total play time in seconds
    pub play_time: u64,
}

impl CharacterData {
    /// Create a new character with default appearance (no archetype yet)
    pub fn new(name: String, sex: Sex) -> Self {
        let appearance = match sex {
            Sex::Male => CharacterAppearance::default_male(),
            Sex::Female => CharacterAppearance::default_female(),
        };
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            sex,
            archetype: None,
            appearance,
            created_at: Utc::now(),
            play_time: 0,
        }
    }

    /// Create a character with custom appearance
    pub fn with_appearance(name: String, sex: Sex, appearance: CharacterAppearance) -> Self {
        Self {
            sex,
            appearance,
            ..Self::new(name, sex)
        }
    }

    /// Set the character's archetype
    #[allow(dead_code)]
    pub fn set_archetype(&mut self, archetype: Archetype) {
        self.archetype = Some(archetype);
    }
}

/// Character archetypes (classes)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Archetype {
    /// Master of time manipulation, bends reality itself
    Chronomancer,
    /// Swift hunter from the timeline, tracks temporal anomalies
    TemporalHunter,
    /// Frontline warrior, anchor in the timestream
    Vanguard,
    /// Technology-enhanced mage, merges magic with machines
    Technomage,
    /// Walks between paradoxes, embraces contradictions
    ParadoxWeaver,
}

#[allow(dead_code)]
impl Archetype {
    /// Get the display name
    pub fn name(&self) -> &'static str {
        match self {
            Self::Chronomancer => "Chronomancer",
            Self::TemporalHunter => "Temporal Hunter",
            Self::Vanguard => "Vanguard",
            Self::Technomage => "Technomage",
            Self::ParadoxWeaver => "Paradox Weaver",
        }
    }

    /// Get the description
    pub fn description(&self) -> &'static str {
        match self {
            Self::Chronomancer => "Master of time manipulation. Slows, stops, and reverses the flow of time itself.",
            Self::TemporalHunter => "Swift hunter who tracks temporal anomalies. Strikes from the shadows of time.",
            Self::Vanguard => "Frontline warrior and anchor in the timestream. Protects allies from temporal flux.",
            Self::Technomage => "Merges ancient magic with future technology. Creates temporal constructs and devices.",
            Self::ParadoxWeaver => "Embraces contradictions and walks between paradoxes. Unpredictable and dangerous.",
        }
    }

    /// Get all archetypes
    pub fn all() -> &'static [Archetype] {
        &[
            Self::Chronomancer,
            Self::TemporalHunter,
            Self::Vanguard,
            Self::Technomage,
            Self::ParadoxWeaver,
        ]
    }

    /// Get base combat stats for this archetype at level 1
    pub fn base_stats(&self) -> CharacterStats {
        match self {
            // Chronomancer: Balanced mage, moderate HP, high crit, high mana
            Self::Chronomancer => CharacterStats {
                max_hp: 80.0,
                current_hp: 80.0,
                attack: 12.0,
                defense: 4.0,
                speed: 1.0,
                crit_chance: 0.10,
                crit_multiplier: 1.75,
                elemental_affinity: self.starting_element(),
                max_mana: 120.0,
                current_mana: 120.0,
                mana_regen: 2.5,
            },
            // Temporal Hunter: Glass cannon, low HP, high damage/speed, low mana
            Self::TemporalHunter => CharacterStats {
                max_hp: 70.0,
                current_hp: 70.0,
                attack: 15.0,
                defense: 3.0,
                speed: 1.3,
                crit_chance: 0.15,
                crit_multiplier: 2.0,
                elemental_affinity: self.starting_element(),
                max_mana: 80.0,
                current_mana: 80.0,
                mana_regen: 1.5,
            },
            // Vanguard: Tank, high HP/defense, lower damage, low mana
            Self::Vanguard => CharacterStats {
                max_hp: 120.0,
                current_hp: 120.0,
                attack: 10.0,
                defense: 8.0,
                speed: 0.9,
                crit_chance: 0.05,
                crit_multiplier: 1.5,
                elemental_affinity: self.starting_element(),
                max_mana: 60.0,
                current_mana: 60.0,
                mana_regen: 1.0,
            },
            // Technomage: Hybrid, moderate stats, high crit damage, highest mana
            Self::Technomage => CharacterStats {
                max_hp: 85.0,
                current_hp: 85.0,
                attack: 14.0,
                defense: 4.0,
                speed: 1.0,
                crit_chance: 0.08,
                crit_multiplier: 2.0,
                elemental_affinity: self.starting_element(),
                max_mana: 130.0,
                current_mana: 130.0,
                mana_regen: 3.0,
            },
            // Paradox Weaver: Unpredictable, balanced with very high crit chance
            Self::ParadoxWeaver => CharacterStats {
                max_hp: 90.0,
                current_hp: 90.0,
                attack: 11.0,
                defense: 5.0,
                speed: 1.1,
                crit_chance: 0.20,
                crit_multiplier: 1.6,
                elemental_affinity: self.starting_element(),
                max_mana: 100.0,
                current_mana: 100.0,
                mana_regen: 2.0,
            },
        }
    }

    /// Get the starting elemental affinity for this archetype
    pub fn starting_element(&self) -> Element {
        match self {
            Self::Chronomancer => Element::Void,
            Self::TemporalHunter => Element::Air,
            Self::Vanguard => Element::Earth,
            Self::Technomage => Element::Fire,
            Self::ParadoxWeaver => Element::Meta,
        }
    }

    /// Get the starting weapon type for this archetype
    pub fn starting_weapon_type(&self) -> WeaponType {
        match self {
            Self::Chronomancer => WeaponType::Staff,
            Self::TemporalHunter => WeaponType::DualBlades,
            Self::Vanguard => WeaponType::Sword,
            Self::Technomage => WeaponType::Wand,
            Self::ParadoxWeaver => WeaponType::Scythe,
        }
    }

    /// Get stat growth rates per level for this archetype
    pub fn stat_growth(&self) -> StatGrowth {
        match self {
            // Chronomancer: Balanced growth
            Self::Chronomancer => StatGrowth {
                hp_per_level: 8.0,
                attack_per_level: 2.5,
                defense_per_level: 1.0,
                speed_per_level: 0.0,
            },
            // Temporal Hunter: Attack-focused growth
            Self::TemporalHunter => StatGrowth {
                hp_per_level: 6.0,
                attack_per_level: 3.5,
                defense_per_level: 0.5,
                speed_per_level: 0.02,
            },
            // Vanguard: Defense/HP focused growth
            Self::Vanguard => StatGrowth {
                hp_per_level: 15.0,
                attack_per_level: 1.5,
                defense_per_level: 2.0,
                speed_per_level: 0.0,
            },
            // Technomage: Attack focused with some HP
            Self::Technomage => StatGrowth {
                hp_per_level: 7.0,
                attack_per_level: 3.0,
                defense_per_level: 1.0,
                speed_per_level: 0.0,
            },
            // Paradox Weaver: Balanced growth
            Self::ParadoxWeaver => StatGrowth {
                hp_per_level: 9.0,
                attack_per_level: 2.0,
                defense_per_level: 1.5,
                speed_per_level: 0.01,
            },
        }
    }
}

impl Default for Archetype {
    fn default() -> Self {
        Self::Chronomancer
    }
}

/// Complete character appearance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterAppearance {
    /// Body shape and proportions
    pub body: BodyCustomization,
    /// Facial features
    pub face: FaceCustomization,
    /// Hair styling
    pub hair: HairCustomization,
    /// Skin details
    pub skin: SkinCustomization,
}

impl Default for CharacterAppearance {
    fn default() -> Self {
        Self {
            body: BodyCustomization::default(),
            face: FaceCustomization::default(),
            hair: HairCustomization::default(),
            skin: SkinCustomization::default(),
        }
    }
}

impl CharacterAppearance {
    /// Default male appearance preset
    pub fn default_male() -> Self {
        Self {
            body: BodyCustomization {
                height: 0.55,
                build: 0.5,
                shoulder_width: 0.6,
                hip_width: 0.4,
                ..Default::default()
            },
            face: FaceCustomization {
                jaw: 0.6,
                face_width: 0.5,
                brow_thickness: 0.6,
                ..Default::default()
            },
            ..Default::default()
        }
    }

    /// Default female appearance preset
    pub fn default_female() -> Self {
        Self {
            body: BodyCustomization {
                height: 0.45,
                build: 0.4,
                shoulder_width: 0.4,
                hip_width: 0.6,
                ..Default::default()
            },
            face: FaceCustomization {
                jaw: 0.4,
                face_width: 0.45,
                lip_fullness: 0.6,
                brow_thickness: 0.4,
                ..Default::default()
            },
            ..Default::default()
        }
    }

    /// Randomize all appearance values
    pub fn randomize(&mut self) {
        let mut rng = rand::thread_rng();

        self.body.randomize(&mut rng);
        self.face.randomize(&mut rng);
        self.hair.randomize(&mut rng);
        self.skin.randomize(&mut rng);
    }
}

/// Body shape customization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BodyCustomization {
    /// Height (0.0 = short, 1.0 = tall)
    pub height: f32,
    /// Body build (0.0 = slim, 1.0 = heavy)
    pub build: f32,
    /// Shoulder width (0.0 = narrow, 1.0 = wide)
    pub shoulder_width: f32,
    /// Hip width (0.0 = narrow, 1.0 = wide)
    pub hip_width: f32,
    /// Arm length (0.0 = short, 1.0 = long)
    pub arm_length: f32,
    /// Leg length (0.0 = short, 1.0 = long)
    pub leg_length: f32,
    /// Torso length (0.0 = short, 1.0 = long)
    pub torso_length: f32,
}

impl Default for BodyCustomization {
    fn default() -> Self {
        Self {
            height: 0.5,
            build: 0.5,
            shoulder_width: 0.5,
            hip_width: 0.5,
            arm_length: 0.5,
            leg_length: 0.5,
            torso_length: 0.5,
        }
    }
}

impl BodyCustomization {
    /// Randomize body values
    pub fn randomize<R: rand::Rng>(&mut self, rng: &mut R) {
        self.height = rng.gen();
        self.build = rng.gen();
        self.shoulder_width = rng.gen();
        self.hip_width = rng.gen();
        self.arm_length = rng.gen::<f32>() * 0.4 + 0.3; // 0.3-0.7 range
        self.leg_length = rng.gen::<f32>() * 0.4 + 0.3;
        self.torso_length = rng.gen::<f32>() * 0.4 + 0.3;
    }
}

/// Face shape customization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FaceCustomization {
    /// Overall face shape (0.0 = round, 1.0 = angular)
    pub face_shape: f32,
    /// Face width (0.0 = narrow, 1.0 = wide)
    pub face_width: f32,
    /// Face length (0.0 = short, 1.0 = long)
    pub face_length: f32,
    /// Jaw definition (0.0 = soft, 1.0 = sharp)
    pub jaw: f32,
    /// Cheekbone prominence (0.0 = flat, 1.0 = prominent)
    pub cheekbones: f32,

    // Eyes
    /// Eye size (0.0 = small, 1.0 = large)
    pub eye_size: f32,
    /// Eye spacing (0.0 = close, 1.0 = far)
    pub eye_spacing: f32,
    /// Eye slant (0.0 = downward, 1.0 = upward)
    pub eye_slant: f32,
    /// Eye color (hue 0.0-1.0)
    pub eye_color: f32,

    // Nose
    /// Nose length (0.0 = short, 1.0 = long)
    pub nose_length: f32,
    /// Nose width (0.0 = narrow, 1.0 = wide)
    pub nose_width: f32,
    /// Nose bridge (0.0 = flat, 1.0 = high)
    pub nose_bridge: f32,

    // Mouth
    /// Lip fullness (0.0 = thin, 1.0 = full)
    pub lip_fullness: f32,
    /// Mouth width (0.0 = narrow, 1.0 = wide)
    pub mouth_width: f32,

    // Ears
    /// Ear size (0.0 = small, 1.0 = large)
    pub ear_size: f32,
    /// Ear pointiness (0.0 = round, 1.0 = pointed)
    pub ear_pointiness: f32,

    // Chin
    /// Chin length (0.0 = short, 1.0 = long)
    pub chin_length: f32,
    /// Chin width (0.0 = narrow, 1.0 = wide)
    pub chin_width: f32,

    // Brows
    /// Brow thickness (0.0 = thin, 1.0 = thick)
    pub brow_thickness: f32,
    /// Brow arch (0.0 = flat, 1.0 = arched)
    pub brow_arch: f32,
}

impl Default for FaceCustomization {
    fn default() -> Self {
        Self {
            face_shape: 0.5,
            face_width: 0.5,
            face_length: 0.5,
            jaw: 0.5,
            cheekbones: 0.5,
            eye_size: 0.5,
            eye_spacing: 0.5,
            eye_slant: 0.5,
            eye_color: 0.6, // Blue-ish
            nose_length: 0.5,
            nose_width: 0.5,
            nose_bridge: 0.5,
            lip_fullness: 0.5,
            mouth_width: 0.5,
            ear_size: 0.5,
            ear_pointiness: 0.0,
            chin_length: 0.5,
            chin_width: 0.5,
            brow_thickness: 0.5,
            brow_arch: 0.5,
        }
    }
}

impl FaceCustomization {
    /// Randomize face values
    pub fn randomize<R: rand::Rng>(&mut self, rng: &mut R) {
        self.face_shape = rng.gen();
        self.face_width = rng.gen();
        self.face_length = rng.gen();
        self.jaw = rng.gen();
        self.cheekbones = rng.gen();
        self.eye_size = rng.gen::<f32>() * 0.6 + 0.2; // 0.2-0.8
        self.eye_spacing = rng.gen::<f32>() * 0.4 + 0.3;
        self.eye_slant = rng.gen();
        self.eye_color = rng.gen();
        self.nose_length = rng.gen();
        self.nose_width = rng.gen();
        self.nose_bridge = rng.gen();
        self.lip_fullness = rng.gen();
        self.mouth_width = rng.gen::<f32>() * 0.4 + 0.3;
        self.ear_size = rng.gen::<f32>() * 0.6 + 0.2;
        self.ear_pointiness = rng.gen::<f32>() * 0.3; // Usually not too pointy
        self.chin_length = rng.gen();
        self.chin_width = rng.gen();
        self.brow_thickness = rng.gen();
        self.brow_arch = rng.gen();
    }
}

/// Hair customization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HairCustomization {
    /// Hair style index (preset styles)
    pub style: u32,
    /// Primary hair color (hue 0.0-1.0)
    pub color_hue: f32,
    /// Hair color saturation (0.0-1.0)
    pub color_saturation: f32,
    /// Hair color brightness (0.0-1.0)
    pub color_brightness: f32,
    /// Highlight color hue (0.0-1.0)
    pub highlight_hue: f32,
    /// Highlight intensity (0.0 = none, 1.0 = strong)
    pub highlight_intensity: f32,
    /// Hair length (0.0 = bald/short, 1.0 = very long)
    pub length: f32,
    /// Hair volume (0.0 = flat, 1.0 = voluminous)
    pub volume: f32,
    /// Facial hair style index (0 = none)
    pub facial_hair_style: u32,
    /// Facial hair density (0.0 = none, 1.0 = full)
    pub facial_hair_density: f32,
}

impl Default for HairCustomization {
    fn default() -> Self {
        Self {
            style: 0,
            color_hue: 0.08, // Brown
            color_saturation: 0.5,
            color_brightness: 0.3,
            highlight_hue: 0.1,
            highlight_intensity: 0.0,
            length: 0.3,
            volume: 0.5,
            facial_hair_style: 0,
            facial_hair_density: 0.0,
        }
    }
}

impl HairCustomization {
    /// Number of available hair styles
    pub const HAIR_STYLE_COUNT: u32 = 20;
    /// Number of available facial hair styles
    pub const FACIAL_HAIR_STYLE_COUNT: u32 = 10;

    /// Randomize hair values
    pub fn randomize<R: rand::Rng>(&mut self, rng: &mut R) {
        self.style = rng.gen_range(0..Self::HAIR_STYLE_COUNT);
        self.color_hue = rng.gen();
        self.color_saturation = rng.gen::<f32>() * 0.7 + 0.1;
        self.color_brightness = rng.gen::<f32>() * 0.6 + 0.1;
        self.highlight_hue = rng.gen();
        self.highlight_intensity = if rng.gen_bool(0.3) {
            rng.gen::<f32>() * 0.5
        } else {
            0.0
        };
        self.length = rng.gen();
        self.volume = rng.gen();
        self.facial_hair_style = if rng.gen_bool(0.4) {
            rng.gen_range(0..Self::FACIAL_HAIR_STYLE_COUNT)
        } else {
            0
        };
        self.facial_hair_density = if self.facial_hair_style > 0 {
            rng.gen::<f32>() * 0.7 + 0.3
        } else {
            0.0
        };
    }
}

/// Skin customization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkinCustomization {
    /// Skin tone (0.0 = very light, 1.0 = very dark)
    pub tone: f32,
    /// Undertone (0.0 = cool/pink, 0.5 = neutral, 1.0 = warm/yellow)
    pub undertone: f32,
    /// Apparent age (0.0 = young, 1.0 = elderly)
    pub age: f32,
    /// Freckle density (0.0 = none, 1.0 = many)
    pub freckles: f32,
    /// Skin blemishes/marks (0.0 = clear, 1.0 = rough)
    pub blemishes: f32,

    /// Tattoo style index (0 = none)
    pub tattoo_style: u32,
    /// Tattoo intensity (0.0 = faded, 1.0 = bold)
    pub tattoo_intensity: f32,
    /// Tattoo color hue (0.0-1.0)
    pub tattoo_color: f32,

    /// Face paint style index (0 = none)
    pub face_paint_style: u32,
    /// Face paint intensity (0.0 = subtle, 1.0 = bold)
    pub face_paint_intensity: f32,
    /// Face paint primary color hue (0.0-1.0)
    pub face_paint_color: f32,
}

impl Default for SkinCustomization {
    fn default() -> Self {
        Self {
            tone: 0.4,
            undertone: 0.5,
            age: 0.3,
            freckles: 0.0,
            blemishes: 0.1,
            tattoo_style: 0,
            tattoo_intensity: 0.0,
            tattoo_color: 0.0,
            face_paint_style: 0,
            face_paint_intensity: 0.0,
            face_paint_color: 0.0,
        }
    }
}

impl SkinCustomization {
    /// Number of available tattoo styles
    pub const TATTOO_STYLE_COUNT: u32 = 15;
    /// Number of available face paint styles
    pub const FACE_PAINT_STYLE_COUNT: u32 = 12;

    /// Randomize skin values
    pub fn randomize<R: rand::Rng>(&mut self, rng: &mut R) {
        self.tone = rng.gen();
        self.undertone = rng.gen();
        self.age = rng.gen::<f32>() * 0.6; // Generally younger characters
        self.freckles = if rng.gen_bool(0.3) {
            rng.gen::<f32>() * 0.6
        } else {
            0.0
        };
        self.blemishes = rng.gen::<f32>() * 0.3;

        // Tattoos (30% chance)
        if rng.gen_bool(0.3) {
            self.tattoo_style = rng.gen_range(1..Self::TATTOO_STYLE_COUNT);
            self.tattoo_intensity = rng.gen::<f32>() * 0.5 + 0.5;
            self.tattoo_color = rng.gen();
        } else {
            self.tattoo_style = 0;
            self.tattoo_intensity = 0.0;
        }

        // Face paint (15% chance)
        if rng.gen_bool(0.15) {
            self.face_paint_style = rng.gen_range(1..Self::FACE_PAINT_STYLE_COUNT);
            self.face_paint_intensity = rng.gen::<f32>() * 0.5 + 0.5;
            self.face_paint_color = rng.gen();
        } else {
            self.face_paint_style = 0;
            self.face_paint_intensity = 0.0;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_character_creation() {
        let character = CharacterData::new("Test".to_string(), Sex::Male);
        assert_eq!(character.name, "Test");
        assert_eq!(character.sex, Sex::Male);
        assert_eq!(character.archetype, None);
        assert_eq!(character.play_time, 0);
    }

    #[test]
    fn test_sex_defaults() {
        let male = CharacterAppearance::default_male();
        let female = CharacterAppearance::default_female();
        // Male should have broader shoulders
        assert!(male.body.shoulder_width > female.body.shoulder_width);
        // Female should have wider hips
        assert!(female.body.hip_width > male.body.hip_width);
    }

    #[test]
    fn test_archetype_descriptions() {
        for archetype in Archetype::all() {
            assert!(!archetype.name().is_empty());
            assert!(!archetype.description().is_empty());
        }
    }

    #[test]
    fn test_appearance_randomize() {
        let mut appearance = CharacterAppearance::default();
        let original = appearance.clone();
        appearance.randomize();
        // Something should have changed (with very high probability)
        assert!(
            appearance.body.height != original.body.height
                || appearance.face.eye_color != original.face.eye_color
                || appearance.hair.style != original.hair.style
        );
    }
}
