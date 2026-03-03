use serde::{Deserialize, Serialize};

use super::abilities::Abilities;
use super::feat::FeatChoice;
use super::proficiencies::Proficiencies;
use super::spell::SpellCaster;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Character {
    pub name: String,
    pub level: u8,
    pub ancestry: Option<String>,
    pub heritage: Option<String>,
    pub background: Option<String>,
    pub class: Option<String>,
    pub deity: Option<String>,
    pub key_ability: Option<String>,
    pub alignment: Option<String>,
    pub size: Option<String>,
    pub speed: u32,
    pub hp_max: u32,
    pub ancestry_hp: u32,
    pub class_hp: u32,
    pub abilities: Abilities,
    pub proficiencies: Proficiencies,
    pub feats: Vec<FeatChoice>,
    pub spell_caster: Option<SpellCaster>,

    // Derived combat stats (recomputed by recalculate)
    #[serde(skip)]
    pub ac: i32,
    #[serde(skip)]
    pub perception_bonus: i32,
    #[serde(skip)]
    pub fortitude_bonus: i32,
    #[serde(skip)]
    pub reflex_bonus: i32,
    #[serde(skip)]
    pub will_bonus: i32,
    #[serde(skip)]
    pub class_dc: i32,
    #[serde(skip)]
    pub melee_attack: i32,
    #[serde(skip)]
    pub ranged_attack: i32,
    #[serde(skip)]
    pub spell_attack: i32,
    #[serde(skip)]
    pub spell_dc: i32,
    #[serde(skip)]
    pub weapon_attacks: Vec<WeaponAttack>,
    #[serde(skip)]
    pub shield_ac_bonus: i32,
    #[serde(skip)]
    pub focus_points: u8,
    #[serde(skip)]
    pub languages: Vec<String>,
    #[serde(skip)]
    pub favored_weapon: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct WeaponAttack {
    pub display: String,
    pub attack_bonus: i32,
    pub damage: String,
}

impl Character {
    pub fn new(name: &str) -> Self {
        Character {
            name: name.to_string(),
            level: 1,
            speed: 25,
            ..Default::default()
        }
    }

    pub fn summary_line(&self) -> String {
        let ancestry = self.ancestry.as_deref().unwrap_or("???");
        let heritage = self.heritage.as_deref();
        let class = self.class.as_deref().unwrap_or("???");
        match heritage {
            Some(h) => format!("{ancestry} {h} / {class} {}", self.level),
            None => format!("{ancestry} / {class} {}", self.level),
        }
    }

}
