use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum StrikingRune {
    #[default]
    None,
    Striking,
    Greater,
    Major,
}

impl StrikingRune {
    /// Extra damage dice from striking rune (0/1/2/3).
    pub fn extra_dice(self) -> u8 {
        match self {
            Self::None => 0,
            Self::Striking => 1,
            Self::Greater => 2,
            Self::Major => 3,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::None => "",
            Self::Striking => "Striking",
            Self::Greater => "Greater Striking",
            Self::Major => "Major Striking",
        }
    }

    pub fn next(self) -> Self {
        match self {
            Self::None => Self::Striking,
            Self::Striking => Self::Greater,
            Self::Greater => Self::Major,
            Self::Major => Self::None,
        }
    }

    pub fn prev(self) -> Self {
        match self {
            Self::None => Self::Major,
            Self::Striking => Self::None,
            Self::Greater => Self::Striking,
            Self::Major => Self::Greater,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResilientRune {
    #[default]
    None,
    Resilient,
    Greater,
    Major,
}

impl ResilientRune {
    /// Save bonus from resilient rune (0/1/2/3).
    pub fn bonus(self) -> i32 {
        match self {
            Self::None => 0,
            Self::Resilient => 1,
            Self::Greater => 2,
            Self::Major => 3,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::None => "",
            Self::Resilient => "Resilient",
            Self::Greater => "Greater Resilient",
            Self::Major => "Major Resilient",
        }
    }

    pub fn next(self) -> Self {
        match self {
            Self::None => Self::Resilient,
            Self::Resilient => Self::Greater,
            Self::Greater => Self::Major,
            Self::Major => Self::None,
        }
    }

    pub fn prev(self) -> Self {
        match self {
            Self::None => Self::Major,
            Self::Resilient => Self::None,
            Self::Greater => Self::Resilient,
            Self::Major => Self::Greater,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Weapon {
    pub name: String,
    pub potency: u8,
    pub striking: StrikingRune,
    #[serde(default)]
    pub property_runes: Vec<String>,
    pub material: Option<String>,
    // Base stats extracted from AON document
    #[serde(default)]
    pub damage_die: u8,         // 4, 6, 8, 10, 12
    #[serde(default)]
    pub damage_type: String,    // "S", "B", "P"
    #[serde(default)]
    pub weapon_category: String, // "Simple", "Martial", "Advanced", "Unarmed"
    #[serde(default)]
    pub weapon_group: String,   // "Sword", "Bow", etc.
    #[serde(default)]
    pub weapon_type: String,    // "Melee", "Ranged"
    #[serde(default)]
    pub hands: String,          // "1", "1+", "2"
    #[serde(default)]
    pub traits: Vec<String>,    // raw traits like "Finesse", "Thrown 10 ft."
    #[serde(default)]
    pub range: u32,             // 0 = melee
}

impl Weapon {
    pub fn display_name(&self) -> String {
        let mut parts = Vec::new();
        if self.potency > 0 {
            parts.push(format!("+{}", self.potency));
        }
        let s = self.striking.label();
        if !s.is_empty() {
            parts.push(s.to_string());
        }
        parts.push(self.name.clone());
        parts.join(" ")
    }

    pub fn is_ranged(&self) -> bool {
        self.weapon_type.eq_ignore_ascii_case("Ranged")
    }

    pub fn has_trait(&self, name: &str) -> bool {
        self.traits
            .iter()
            .any(|t| t.eq_ignore_ascii_case(name) || t.to_lowercase().starts_with(&name.to_lowercase()))
    }

    pub fn damage_die_str(&self) -> String {
        if self.damage_die > 0 {
            format!("d{}", self.damage_die)
        } else {
            "d6".to_string()
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Armor {
    pub name: String,
    pub potency: u8,
    pub resilient: ResilientRune,
    #[serde(default)]
    pub property_runes: Vec<String>,
    pub material: Option<String>,
    pub worn: bool,
    // Base stats extracted from AON document
    #[serde(default)]
    pub ac_bonus: i32,
    #[serde(default)]
    pub dex_cap: Option<i32>,
    #[serde(default)]
    pub armor_category: String, // "Unarmored", "Light", "Medium", "Heavy"
    #[serde(default)]
    pub armor_group: String,
    #[serde(default)]
    pub check_penalty: i32,
    #[serde(default)]
    pub speed_penalty: i32,
    #[serde(default)]
    pub strength: u32,
}

impl Armor {
    pub fn display_name(&self) -> String {
        let mut parts = Vec::new();
        if self.potency > 0 {
            parts.push(format!("+{}", self.potency));
        }
        let r = self.resilient.label();
        if !r.is_empty() {
            parts.push(r.to_string());
        }
        parts.push(self.name.clone());
        parts.join(" ")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Shield {
    pub name: String,
    pub potency: u8,
    pub resilient: ResilientRune,
    #[serde(default)]
    pub property_runes: Vec<String>,
    pub raised: bool,
    // Base stats extracted from AON document
    #[serde(default)]
    pub ac_bonus: i32,
    #[serde(default)]
    pub hardness: i32,
    #[serde(default)]
    pub hp: i32,
}

impl Shield {
    pub fn display_name(&self) -> String {
        let mut parts = Vec::new();
        if self.potency > 0 {
            parts.push(format!("+{}", self.potency));
        }
        parts.push(self.name.clone());
        parts.join(" ")
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Money {
    pub cp: u32,
    pub sp: u32,
    pub gp: u32,
    pub pp: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Item {
    pub name: String,
    pub quantity: u32,
    pub invested: bool,
    pub worn: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Equipment {
    pub weapons: Vec<Weapon>,
    pub armor: Vec<Armor>,
    pub shield: Option<Shield>,
    pub items: Vec<Item>,
    pub money: Money,
}
