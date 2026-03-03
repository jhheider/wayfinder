use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DivineFont {
    Heal,
    Harm,
}

impl DivineFont {
    pub fn label(&self) -> &str {
        match self {
            DivineFont::Heal => "Heal",
            DivineFont::Harm => "Harm",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "heal" => Some(DivineFont::Heal),
            "harm" => Some(DivineFont::Harm),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Sanctification {
    Holy,
    Unholy,
}

impl Sanctification {
    pub fn label(&self) -> &str {
        match self {
            Sanctification::Holy => "Holy",
            Sanctification::Unholy => "Unholy",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "holy" => Some(Sanctification::Holy),
            "unholy" => Some(Sanctification::Unholy),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Tradition {
    Arcane,
    Divine,
    Occult,
    Primal,
}

impl Tradition {
    pub fn label(&self) -> &str {
        match self {
            Tradition::Arcane => "Arcane",
            Tradition::Divine => "Divine",
            Tradition::Occult => "Occult",
            Tradition::Primal => "Primal",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "arcane" => Some(Tradition::Arcane),
            "divine" => Some(Tradition::Divine),
            "occult" => Some(Tradition::Occult),
            "primal" => Some(Tradition::Primal),
            _ => None,
        }
    }
}
