use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Ability {
    Strength,
    Dexterity,
    Constitution,
    Intelligence,
    Wisdom,
    Charisma,
}

impl Ability {
    pub const ALL: [Ability; 6] = [
        Ability::Strength,
        Ability::Dexterity,
        Ability::Constitution,
        Ability::Intelligence,
        Ability::Wisdom,
        Ability::Charisma,
    ];

    pub fn abbr(&self) -> &'static str {
        match self {
            Ability::Strength => "STR",
            Ability::Dexterity => "DEX",
            Ability::Constitution => "CON",
            Ability::Intelligence => "INT",
            Ability::Wisdom => "WIS",
            Ability::Charisma => "CHA",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoostSource {
    pub source: String,
    pub ability: Ability,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Abilities {
    pub boosts: Vec<BoostSource>,
    pub flaws: Vec<BoostSource>,
}

impl Abilities {
    pub fn score(&self, ability: Ability) -> i32 {
        let base = 10;
        let boosts = self.boosts.iter().filter(|b| b.ability == ability).count() as i32;
        let flaws = self.flaws.iter().filter(|b| b.ability == ability).count() as i32;
        base + boosts * 2 - flaws * 2
    }

    pub fn modifier(&self, ability: Ability) -> i32 {
        (self.score(ability) - 10) / 2
    }
}

/// Describes what boosts a source (ancestry/background/class) offers.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BoostSpec {
    /// Fixed ability boosts (e.g. "Dexterity", "Intelligence").
    pub fixed: Vec<Ability>,
    /// Number of free boosts the user can assign.
    pub free: u8,
    /// Ability flaws (ancestry only).
    pub flaws: Vec<Ability>,
}
