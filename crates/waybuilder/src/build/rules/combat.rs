//! Derived combat stats: AC, saves, attacks, class DC.
//!
//! Formula: ability_mod + proficiency_rank.bonus() + level
//! AC adds +10 base. DCs add +10 base.

use crate::model::abilities::{Abilities, Ability};
use crate::model::proficiencies::Rank;

/// Compute Armor Class from equipped armor.
/// AC = 10 + min(DEX_mod, dex_cap) + armor_rank.bonus() + level + ac_bonus + potency
pub fn compute_ac(
    abilities: &Abilities,
    armor_rank: Rank,
    level: u8,
    ac_bonus: i32,
    dex_cap: Option<i32>,
    potency: u8,
) -> i32 {
    let dex = abilities.modifier(Ability::Dexterity);
    let capped_dex = match dex_cap {
        Some(cap) => dex.min(cap),
        None => dex,
    };
    10 + capped_dex + armor_rank.bonus() + level as i32 + ac_bonus + potency as i32
}

/// Compute a saving throw bonus: ability_mod + rank.bonus() + level.
pub fn compute_save(
    abilities: &Abilities,
    ability: Ability,
    rank: Rank,
    level: u8,
) -> i32 {
    abilities.modifier(ability) + rank.bonus() + level as i32
}

/// Compute perception bonus: WIS mod + rank.bonus() + level.
pub fn compute_perception(
    abilities: &Abilities,
    rank: Rank,
    level: u8,
) -> i32 {
    abilities.modifier(Ability::Wisdom) + rank.bonus() + level as i32
}

/// Compute class DC: 10 + key ability mod + rank.bonus() + level.
pub fn compute_class_dc(
    abilities: &Abilities,
    key_ability: Ability,
    rank: Rank,
    level: u8,
) -> i32 {
    10 + abilities.modifier(key_ability) + rank.bonus() + level as i32
}

/// Compute melee attack bonus: STR mod + rank.bonus() + level.
pub fn compute_melee_attack(
    abilities: &Abilities,
    rank: Rank,
    level: u8,
) -> i32 {
    abilities.modifier(Ability::Strength) + rank.bonus() + level as i32
}

/// Compute ranged attack bonus: DEX mod + rank.bonus() + level.
pub fn compute_ranged_attack(
    abilities: &Abilities,
    rank: Rank,
    level: u8,
) -> i32 {
    abilities.modifier(Ability::Dexterity) + rank.bonus() + level as i32
}
