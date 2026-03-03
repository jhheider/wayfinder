use serde::{Deserialize, Serialize};

/// A type of build choice slot in the character progression.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BuildSlot {
    Ancestry,
    Heritage,
    Background,
    Class,
    Subclass,
    AbilityBoosts,
    AncestryFeat,
    ClassFeat,
    ClassFeature,
    SkillFeat,
    GeneralFeat,
    SkillIncrease,
    Deity,
    SkillSelection,
}

impl BuildSlot {
    pub fn label(&self) -> &'static str {
        match self {
            BuildSlot::Ancestry => "Ancestry",
            BuildSlot::Heritage => "Heritage",
            BuildSlot::Background => "Background",
            BuildSlot::Class => "Class",
            BuildSlot::Subclass => "Subclass",
            BuildSlot::AbilityBoosts => "Ability Boosts",
            BuildSlot::AncestryFeat => "Ancestry Feat",
            BuildSlot::ClassFeat => "Class Feat",
            BuildSlot::ClassFeature => "Class Feature",
            BuildSlot::SkillFeat => "Skill Feat",
            BuildSlot::GeneralFeat => "General Feat",
            BuildSlot::SkillIncrease => "Skill Increase",
            BuildSlot::Deity => "Deity",
            BuildSlot::SkillSelection => "Skill Selection",
        }
    }

    /// AON category to search when filling this slot.
    pub fn search_category(&self) -> Option<&'static str> {
        match self {
            BuildSlot::Ancestry => Some("ancestry"),
            BuildSlot::Heritage => Some("heritage"),
            BuildSlot::Background => Some("background"),
            BuildSlot::Class => Some("class"),
            BuildSlot::AncestryFeat => Some("feat"),
            BuildSlot::ClassFeat => Some("feat"),
            BuildSlot::SkillFeat => Some("feat"),
            BuildSlot::GeneralFeat => Some("feat"),
            BuildSlot::Deity => Some("deity"),
            BuildSlot::ClassFeature => Some("class-feature"),
            // Subclass category is dynamic; handled in events.rs
            BuildSlot::Subclass
            | BuildSlot::AbilityBoosts
            | BuildSlot::SkillIncrease
            | BuildSlot::SkillSelection => None,
        }
    }
}

/// Tracks whether a slot has been filled.
#[derive(Debug, Clone)]
pub struct SlotState {
    pub slot: BuildSlot,
    pub level: u8,
    pub filled: Option<String>,
    pub valid: bool,
}

impl SlotState {
    pub fn new(slot: BuildSlot, level: u8) -> Self {
        Self {
            slot,
            level,
            filled: None,
            valid: true,
        }
    }

    pub fn display(&self) -> String {
        match &self.filled {
            Some(name) => format!("{}: {name}", self.slot.label()),
            None => format!("{}: —", self.slot.label()),
        }
    }
}
