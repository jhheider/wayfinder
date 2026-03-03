use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::build::mechanics::proficiency::ProficiencyAdvance;
use crate::model::abilities::{Ability, BoostSpec};
use crate::model::equipment::Equipment;
use crate::model::feat::FeatType;
use crate::model::proficiencies::Rank;
use crate::model::types::{DivineFont, Sanctification};

/// All user decisions that define a character build.
/// Derived state (ability scores, HP, proficiencies) is computed
/// by `recalculate()` from these choices.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BuildChoices {
    pub name: String,
    pub level: u8,

    // Slot selections (name only)
    pub ancestry: Option<String>,
    pub heritage: Option<String>,
    pub background: Option<String>,
    pub class: Option<String>,
    pub deity: Option<String>,
    pub subclass: Option<String>,

    // Parsed AON data (extracted on selection)
    #[serde(default)]
    pub ancestry_data: Option<AncestryData>,
    #[serde(default)]
    pub background_data: Option<BackgroundData>,
    #[serde(default)]
    pub class_data: Option<ClassData>,

    // Ability boost choices keyed by source
    #[serde(default)]
    pub ability_choices: BTreeMap<String, AbilityChoiceSet>,

    // Skill increase choices keyed by level
    #[serde(default)]
    pub skill_increases: BTreeMap<u8, String>,

    // Feat selections keyed by slot ("Ancestry Feat_1", etc.)
    #[serde(default)]
    pub feats: BTreeMap<String, FeatSelection>,

    // Initial trained skill selections at L1
    #[serde(default)]
    pub initial_skills: Vec<String>,

    // Prepared spells keyed by spell rank
    #[serde(default)]
    pub prepared_spells: BTreeMap<u8, Vec<SpellSlotChoice>>,

    // Alternate ancestry boosts toggle
    #[serde(default)]
    pub use_alternate_ancestry: bool,

    // Focus spells keyed by slot name ("focus_0", "focus_1", etc.)
    #[serde(default)]
    pub focus_spells: Vec<SpellSlotChoice>,

    // Equipment (weapons, armor, shield, items, money)
    #[serde(default)]
    pub equipment: Equipment,

    // Deity extracted data
    #[serde(default)]
    pub deity_data: Option<DeityData>,

    // Bonus languages chosen by the player (capped at INT mod)
    #[serde(default)]
    pub bonus_languages: Vec<String>,

    // Deity choice fields
    #[serde(default)]
    pub chosen_domains: Vec<String>,
    #[serde(default)]
    pub chosen_divine_font: Option<DivineFont>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AncestryData {
    pub size: Option<String>,
    pub speed: u32,
    pub hp: u32,
    pub boost_spec: BoostSpec,
    #[serde(default)]
    pub granted_languages: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeityData {
    pub domains: Vec<String>,
    pub primary_domains: Vec<String>,
    pub divine_font: Vec<DivineFont>,
    pub favored_weapon: Option<String>,
    pub divine_skill: Option<String>,
    pub sanctification: Vec<Sanctification>,
    pub edicts: Option<String>,
    pub anathema: Option<String>,
    pub cleric_spells: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackgroundData {
    pub boost_spec: BoostSpec,
    pub granted_skills: Vec<String>,
    pub granted_lores: Vec<String>,
    pub granted_feat: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassData {
    pub hp: u32,
    pub key_ability: Option<String>,
    pub boost_spec: BoostSpec,
    pub perception: Rank,
    pub fortitude: Rank,
    pub reflex: Rank,
    pub will: Rank,
    pub unarmored: Rank,
    pub light_armor: Rank,
    pub medium_armor: Rank,
    pub heavy_armor: Rank,
    pub simple_weapons: Rank,
    pub martial_weapons: Rank,
    pub unarmed: Rank,
    pub granted_skills: Vec<String>,
    #[serde(default)]
    pub proficiency_advances: Vec<ProficiencyAdvance>,
    /// AON category for this class's subclass (e.g. "bloodline").
    #[serde(default)]
    pub subclass_category: Option<String>,
    /// Class features granted automatically by level.
    #[serde(default)]
    pub class_features: Vec<ClassFeatureEntry>,
    /// Number of additional trained skills from class (beyond fixed grants).
    #[serde(default)]
    pub additional_skill_count: u8,
    /// Spell slots per day table: spell_slots[level-1] = vec of slots per rank.
    /// Each inner vec: [cantrips, 1st, 2nd, ...].
    #[serde(default)]
    pub spell_slots: Vec<Vec<String>>,
    /// Spellcasting tradition (e.g. "Arcane", "Divine").
    #[serde(default)]
    pub tradition: Option<String>,
    /// Casting ability (e.g. "Intelligence", "Wisdom").
    #[serde(default)]
    pub casting_ability: Option<String>,
}

/// A class feature auto-granted at a specific level.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassFeatureEntry {
    pub name: String,
    pub level: u8,
    /// True if this entry represents a class feat slot (e.g. "barbarian feat").
    #[serde(default)]
    pub is_feat_slot: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpellSlotChoice {
    pub name: String,
    pub rank: u8,
    pub source_id: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AbilityChoiceSet {
    pub fixed: Vec<Ability>,
    pub chosen: Vec<Ability>,
    pub flaws: Vec<Ability>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatSelection {
    pub name: String,
    pub feat_type: FeatType,
    pub level: u8,
    pub source_id: Option<String>,
    #[serde(default)]
    pub source_context: Option<String>,
}

impl BuildChoices {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            level: 1,
            ..Default::default()
        }
    }
}
