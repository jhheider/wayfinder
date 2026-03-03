use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Rank {
    #[default]
    Untrained = 0,
    Trained = 2,
    Expert = 4,
    Master = 6,
    Legendary = 8,
}

impl Rank {
    pub fn label(&self) -> &'static str {
        match self {
            Rank::Untrained => "Untrained",
            Rank::Trained => "Trained",
            Rank::Expert => "Expert",
            Rank::Master => "Master",
            Rank::Legendary => "Legendary",
        }
    }

    pub fn bonus(&self) -> i32 {
        *self as i32
    }

    pub fn next(self) -> Option<Self> {
        match self {
            Rank::Untrained => Some(Rank::Trained),
            Rank::Trained => Some(Rank::Expert),
            Rank::Expert => Some(Rank::Master),
            Rank::Master => Some(Rank::Legendary),
            Rank::Legendary => None,
        }
    }
}

/// Standard PF2e skill names.
pub const SKILLS: &[&str] = &[
    "Acrobatics",
    "Arcana",
    "Athletics",
    "Crafting",
    "Deception",
    "Diplomacy",
    "Intimidation",
    "Medicine",
    "Nature",
    "Occultism",
    "Performance",
    "Religion",
    "Society",
    "Stealth",
    "Survival",
    "Thievery",
];

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Proficiencies {
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
    /// Standard skill proficiencies: (skill_name, rank).
    pub skills: Vec<(String, Rank)>,
    /// Lore skill proficiencies: (lore_name, rank).
    #[serde(default)]
    pub lores: Vec<(String, Rank)>,
}

impl Proficiencies {
    /// Get skill rank, defaulting to Untrained.
    pub fn skill_rank(&self, skill: &str) -> Rank {
        self.skills
            .iter()
            .find(|(s, _)| s.eq_ignore_ascii_case(skill))
            .map(|(_, r)| *r)
            .unwrap_or(Rank::Untrained)
    }

    /// Set skill to at least the given rank (never downgrades).
    pub fn train_skill(&mut self, skill: &str, rank: Rank) {
        if let Some(entry) = self
            .skills
            .iter_mut()
            .find(|(s, _)| s.eq_ignore_ascii_case(skill))
        {
            if rank > entry.1 {
                entry.1 = rank;
            }
        } else {
            self.skills.push((skill.to_string(), rank));
        }
    }

    /// Increase a skill by one rank. Returns true if successful.
    pub fn increase_skill(&mut self, skill: &str) -> bool {
        if let Some(entry) = self
            .skills
            .iter_mut()
            .find(|(s, _)| s.eq_ignore_ascii_case(skill))
            && let Some(next) = entry.1.next()
        {
            entry.1 = next;
            return true;
        }
        false
    }

    /// Add a lore skill at Trained.
    pub fn add_lore(&mut self, lore: &str) {
        if !self.lores.iter().any(|(l, _)| l.eq_ignore_ascii_case(lore)) {
            self.lores.push((lore.to_string(), Rank::Trained));
        }
    }
}
