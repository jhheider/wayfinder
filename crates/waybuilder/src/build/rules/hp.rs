use crate::model::abilities::Ability;
use crate::model::character::Character;

/// Calculate max HP: ancestry HP + (class HP + CON modifier) * level.
#[allow(dead_code)]
pub fn calculate_hp(character: &Character) -> u32 {
    let con_mod = character.abilities.modifier(Ability::Constitution);
    let per_level = (character.class_hp as i32 + con_mod).max(1);
    character.ancestry_hp + (per_level * character.level as i32) as u32
}
