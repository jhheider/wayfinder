//! Apply proficiency advances by level.
//!
//! Uses a static table for core PF2e classes. Unknown classes
//! get no advances (until class feature parsing is added).

use crate::build::mechanics::proficiency::ProficiencyAdvance;
use crate::model::character::Character;
use crate::model::proficiencies::Rank;

/// Apply all proficiency advances up to the character's level.
pub fn apply_proficiency_advances(ch: &mut Character, advances: &[ProficiencyAdvance]) {
    for adv in advances {
        if adv.level > ch.level {
            continue;
        }
        apply_one(ch, &adv.target, adv.rank);
    }
}

fn apply_one(ch: &mut Character, target: &str, rank: Rank) {
    match target.to_lowercase().as_str() {
        "perception" => upgrade(&mut ch.proficiencies.perception, rank),
        "fortitude" | "fortitude saves" => upgrade(&mut ch.proficiencies.fortitude, rank),
        "reflex" | "reflex saves" => upgrade(&mut ch.proficiencies.reflex, rank),
        "will" | "will saves" => upgrade(&mut ch.proficiencies.will, rank),
        "unarmored" | "unarmored defense" => upgrade(&mut ch.proficiencies.unarmored, rank),
        "light armor" => upgrade(&mut ch.proficiencies.light_armor, rank),
        "medium armor" => upgrade(&mut ch.proficiencies.medium_armor, rank),
        "heavy armor" => upgrade(&mut ch.proficiencies.heavy_armor, rank),
        "simple weapons" => upgrade(&mut ch.proficiencies.simple_weapons, rank),
        "martial weapons" => upgrade(&mut ch.proficiencies.martial_weapons, rank),
        "unarmed" | "unarmed attacks" => upgrade(&mut ch.proficiencies.unarmed, rank),
        "all saves" => {
            upgrade(&mut ch.proficiencies.fortitude, rank);
            upgrade(&mut ch.proficiencies.reflex, rank);
            upgrade(&mut ch.proficiencies.will, rank);
        }
        "all armor" | "light armor and medium armor and unarmored defense" => {
            upgrade(&mut ch.proficiencies.unarmored, rank);
            upgrade(&mut ch.proficiencies.light_armor, rank);
            upgrade(&mut ch.proficiencies.medium_armor, rank);
        }
        "all weapons" | "simple weapons and martial weapons and unarmed attacks" => {
            upgrade(&mut ch.proficiencies.simple_weapons, rank);
            upgrade(&mut ch.proficiencies.martial_weapons, rank);
            upgrade(&mut ch.proficiencies.unarmed, rank);
        }
        _ => {
            // Try as a skill
            ch.proficiencies.train_skill(target, rank);
        }
    }
}

fn upgrade(current: &mut Rank, target: Rank) {
    if target > *current {
        *current = target;
    }
}

/// Get the static proficiency advance table for a known class.
/// Returns an empty vec for unknown classes.
pub fn class_proficiency_advances(class_name: &str) -> Vec<ProficiencyAdvance> {
    let lower = class_name.to_lowercase();
    match lower.as_str() {
        "fighter" => fighter_advances(),
        "rogue" => rogue_advances(),
        "wizard" => wizard_advances(),
        "cleric" => cleric_advances(),
        "ranger" => ranger_advances(),
        "barbarian" => barbarian_advances(),
        "bard" => bard_advances(),
        "champion" => champion_advances(),
        "druid" => druid_advances(),
        "monk" => monk_advances(),
        "sorcerer" => sorcerer_advances(),
        "witch" => witch_advances(),
        "oracle" => oracle_advances(),
        "investigator" => investigator_advances(),
        "swashbuckler" => swashbuckler_advances(),
        "magus" => magus_advances(),
        "summoner" => summoner_advances(),
        "psychic" => psychic_advances(),
        "thaumaturge" => thaumaturge_advances(),
        "kineticist" => kineticist_advances(),
        "gunslinger" => gunslinger_advances(),
        "inventor" => inventor_advances(),
        _ => Vec::new(),
    }
}

// Helper to reduce boilerplate
fn adv(level: u8, target: &str, rank: Rank) -> ProficiencyAdvance {
    ProficiencyAdvance {
        level,
        target: target.to_string(),
        rank,
    }
}

fn fighter_advances() -> Vec<ProficiencyAdvance> {
    vec![
        adv(3, "all saves", Rank::Expert),
        adv(5, "all weapons", Rank::Expert),
        adv(7, "perception", Rank::Master),
        adv(9, "fortitude", Rank::Master),
        adv(9, "reflex", Rank::Master),
        adv(11, "all armor", Rank::Expert),
        adv(13, "all weapons", Rank::Master),
        adv(15, "will", Rank::Master),
        adv(17, "all armor", Rank::Master),
        adv(19, "all weapons", Rank::Legendary),
    ]
}

fn rogue_advances() -> Vec<ProficiencyAdvance> {
    vec![
        adv(3, "perception", Rank::Expert),
        adv(3, "will", Rank::Expert),
        adv(5, "simple weapons", Rank::Expert),
        adv(5, "unarmed", Rank::Expert),
        adv(7, "perception", Rank::Master),
        adv(7, "reflex", Rank::Master),
        adv(9, "fortitude", Rank::Expert),
        adv(11, "light armor", Rank::Expert),
        adv(11, "unarmored", Rank::Expert),
        adv(13, "reflex", Rank::Legendary),
        adv(13, "will", Rank::Master),
        adv(15, "perception", Rank::Legendary),
        adv(17, "fortitude", Rank::Master),
        adv(19, "light armor", Rank::Master),
        adv(19, "unarmored", Rank::Master),
    ]
}

fn wizard_advances() -> Vec<ProficiencyAdvance> {
    vec![
        adv(3, "fortitude", Rank::Expert),
        adv(5, "perception", Rank::Expert),
        adv(9, "will", Rank::Master),
        adv(11, "simple weapons", Rank::Expert),
        adv(11, "unarmed", Rank::Expert),
        adv(15, "perception", Rank::Master),
        adv(17, "will", Rank::Legendary),
    ]
}

fn cleric_advances() -> Vec<ProficiencyAdvance> {
    vec![
        adv(3, "fortitude", Rank::Expert),
        adv(5, "perception", Rank::Expert),
        adv(9, "reflex", Rank::Expert),
        adv(11, "will", Rank::Master),
        adv(13, "unarmored", Rank::Expert),
        adv(15, "perception", Rank::Master),
        adv(17, "fortitude", Rank::Master),
        adv(19, "will", Rank::Legendary),
    ]
}

fn ranger_advances() -> Vec<ProficiencyAdvance> {
    vec![
        adv(3, "perception", Rank::Expert),
        adv(3, "will", Rank::Expert),
        adv(5, "simple weapons", Rank::Expert),
        adv(5, "martial weapons", Rank::Expert),
        adv(5, "unarmed", Rank::Expert),
        adv(7, "perception", Rank::Master),
        adv(7, "reflex", Rank::Master),
        adv(9, "fortitude", Rank::Expert),
        adv(11, "medium armor", Rank::Expert),
        adv(11, "light armor", Rank::Expert),
        adv(11, "unarmored", Rank::Expert),
        adv(13, "simple weapons", Rank::Master),
        adv(13, "martial weapons", Rank::Master),
        adv(13, "unarmed", Rank::Master),
        adv(15, "will", Rank::Master),
        adv(17, "perception", Rank::Legendary),
        adv(17, "fortitude", Rank::Master),
        adv(19, "medium armor", Rank::Master),
        adv(19, "light armor", Rank::Master),
        adv(19, "unarmored", Rank::Master),
    ]
}

fn barbarian_advances() -> Vec<ProficiencyAdvance> {
    vec![
        adv(3, "will", Rank::Expert),
        adv(5, "simple weapons", Rank::Expert),
        adv(5, "martial weapons", Rank::Expert),
        adv(5, "unarmed", Rank::Expert),
        adv(7, "perception", Rank::Expert),
        adv(7, "fortitude", Rank::Master),
        adv(9, "reflex", Rank::Expert),
        adv(11, "medium armor", Rank::Expert),
        adv(11, "light armor", Rank::Expert),
        adv(11, "unarmored", Rank::Expert),
        adv(13, "simple weapons", Rank::Master),
        adv(13, "martial weapons", Rank::Master),
        adv(13, "unarmed", Rank::Master),
        adv(15, "fortitude", Rank::Legendary),
        adv(17, "perception", Rank::Master),
        adv(17, "will", Rank::Master),
        adv(19, "medium armor", Rank::Master),
        adv(19, "light armor", Rank::Master),
        adv(19, "unarmored", Rank::Master),
    ]
}

fn bard_advances() -> Vec<ProficiencyAdvance> {
    vec![
        adv(3, "will", Rank::Expert),
        adv(5, "perception", Rank::Expert),
        adv(7, "fortitude", Rank::Expert),
        adv(9, "reflex", Rank::Expert),
        adv(11, "simple weapons", Rank::Expert),
        adv(11, "unarmed", Rank::Expert),
        adv(11, "will", Rank::Master),
        adv(13, "light armor", Rank::Expert),
        adv(13, "unarmored", Rank::Expert),
        adv(15, "perception", Rank::Master),
        adv(17, "will", Rank::Legendary),
    ]
}

fn champion_advances() -> Vec<ProficiencyAdvance> {
    vec![
        adv(3, "will", Rank::Expert),
        adv(5, "simple weapons", Rank::Expert),
        adv(5, "martial weapons", Rank::Expert),
        adv(5, "unarmed", Rank::Expert),
        adv(7, "perception", Rank::Expert),
        adv(7, "fortitude", Rank::Master),
        adv(9, "reflex", Rank::Expert),
        adv(11, "all armor", Rank::Expert),
        adv(11, "heavy armor", Rank::Expert),
        adv(13, "simple weapons", Rank::Master),
        adv(13, "martial weapons", Rank::Master),
        adv(13, "unarmed", Rank::Master),
        adv(15, "will", Rank::Master),
        adv(17, "all armor", Rank::Master),
        adv(17, "heavy armor", Rank::Master),
        adv(19, "fortitude", Rank::Legendary),
    ]
}

fn druid_advances() -> Vec<ProficiencyAdvance> {
    vec![
        adv(3, "fortitude", Rank::Expert),
        adv(5, "perception", Rank::Expert),
        adv(7, "will", Rank::Expert),
        adv(9, "reflex", Rank::Expert),
        adv(11, "will", Rank::Master),
        adv(11, "simple weapons", Rank::Expert),
        adv(11, "unarmed", Rank::Expert),
        adv(13, "light armor", Rank::Expert),
        adv(13, "medium armor", Rank::Expert),
        adv(13, "unarmored", Rank::Expert),
        adv(15, "perception", Rank::Master),
        adv(17, "fortitude", Rank::Master),
        adv(19, "will", Rank::Legendary),
    ]
}

fn monk_advances() -> Vec<ProficiencyAdvance> {
    vec![
        adv(3, "fortitude", Rank::Expert),
        adv(3, "reflex", Rank::Expert),
        adv(5, "unarmed", Rank::Expert),
        adv(7, "perception", Rank::Expert),
        adv(7, "will", Rank::Master),
        adv(9, "fortitude", Rank::Master),
        adv(11, "unarmored", Rank::Expert),
        adv(13, "unarmed", Rank::Master),
        adv(13, "reflex", Rank::Master),
        adv(15, "perception", Rank::Master),
        adv(17, "will", Rank::Legendary),
        adv(19, "unarmored", Rank::Master),
    ]
}

fn sorcerer_advances() -> Vec<ProficiencyAdvance> {
    vec![
        adv(3, "will", Rank::Expert),
        adv(5, "perception", Rank::Expert),
        adv(7, "fortitude", Rank::Expert),
        adv(9, "reflex", Rank::Expert),
        adv(11, "simple weapons", Rank::Expert),
        adv(11, "unarmed", Rank::Expert),
        adv(13, "will", Rank::Master),
        adv(15, "perception", Rank::Master),
        adv(17, "will", Rank::Legendary),
    ]
}

fn witch_advances() -> Vec<ProficiencyAdvance> {
    vec![
        adv(3, "fortitude", Rank::Expert),
        adv(5, "perception", Rank::Expert),
        adv(9, "will", Rank::Master),
        adv(11, "simple weapons", Rank::Expert),
        adv(11, "unarmed", Rank::Expert),
        adv(15, "perception", Rank::Master),
        adv(17, "will", Rank::Legendary),
    ]
}

fn oracle_advances() -> Vec<ProficiencyAdvance> {
    vec![
        adv(3, "will", Rank::Expert),
        adv(5, "perception", Rank::Expert),
        adv(7, "fortitude", Rank::Expert),
        adv(9, "reflex", Rank::Expert),
        adv(11, "simple weapons", Rank::Expert),
        adv(11, "unarmed", Rank::Expert),
        adv(11, "will", Rank::Master),
        adv(13, "light armor", Rank::Expert),
        adv(13, "unarmored", Rank::Expert),
        adv(15, "perception", Rank::Master),
        adv(17, "will", Rank::Legendary),
    ]
}

fn investigator_advances() -> Vec<ProficiencyAdvance> {
    vec![
        adv(3, "perception", Rank::Expert),
        adv(3, "will", Rank::Expert),
        adv(5, "simple weapons", Rank::Expert),
        adv(5, "martial weapons", Rank::Expert),
        adv(5, "unarmed", Rank::Expert),
        adv(7, "perception", Rank::Master),
        adv(7, "fortitude", Rank::Expert),
        adv(9, "reflex", Rank::Expert),
        adv(9, "will", Rank::Master),
        adv(11, "light armor", Rank::Expert),
        adv(11, "medium armor", Rank::Expert),
        adv(11, "unarmored", Rank::Expert),
        adv(13, "simple weapons", Rank::Master),
        adv(13, "martial weapons", Rank::Master),
        adv(13, "unarmed", Rank::Master),
        adv(15, "perception", Rank::Legendary),
        adv(17, "will", Rank::Legendary),
    ]
}

fn swashbuckler_advances() -> Vec<ProficiencyAdvance> {
    vec![
        adv(3, "perception", Rank::Expert),
        adv(3, "reflex", Rank::Expert),
        adv(5, "simple weapons", Rank::Expert),
        adv(5, "martial weapons", Rank::Expert),
        adv(5, "unarmed", Rank::Expert),
        adv(7, "fortitude", Rank::Expert),
        adv(7, "reflex", Rank::Master),
        adv(9, "will", Rank::Expert),
        adv(11, "light armor", Rank::Expert),
        adv(11, "unarmored", Rank::Expert),
        adv(13, "simple weapons", Rank::Master),
        adv(13, "martial weapons", Rank::Master),
        adv(13, "unarmed", Rank::Master),
        adv(15, "perception", Rank::Master),
        adv(15, "reflex", Rank::Legendary),
        adv(17, "fortitude", Rank::Master),
        adv(19, "light armor", Rank::Master),
        adv(19, "unarmored", Rank::Master),
    ]
}

fn magus_advances() -> Vec<ProficiencyAdvance> {
    vec![
        adv(3, "will", Rank::Expert),
        adv(5, "simple weapons", Rank::Expert),
        adv(5, "martial weapons", Rank::Expert),
        adv(5, "unarmed", Rank::Expert),
        adv(7, "perception", Rank::Expert),
        adv(7, "fortitude", Rank::Expert),
        adv(9, "reflex", Rank::Expert),
        adv(11, "medium armor", Rank::Expert),
        adv(11, "light armor", Rank::Expert),
        adv(11, "unarmored", Rank::Expert),
        adv(13, "simple weapons", Rank::Master),
        adv(13, "martial weapons", Rank::Master),
        adv(13, "unarmed", Rank::Master),
        adv(15, "will", Rank::Master),
        adv(17, "perception", Rank::Master),
    ]
}

fn summoner_advances() -> Vec<ProficiencyAdvance> {
    vec![
        adv(3, "will", Rank::Expert),
        adv(5, "perception", Rank::Expert),
        adv(7, "fortitude", Rank::Expert),
        adv(9, "reflex", Rank::Expert),
        adv(11, "simple weapons", Rank::Expert),
        adv(11, "unarmed", Rank::Expert),
        adv(11, "will", Rank::Master),
        adv(13, "unarmored", Rank::Expert),
        adv(15, "perception", Rank::Master),
        adv(17, "will", Rank::Legendary),
    ]
}

fn psychic_advances() -> Vec<ProficiencyAdvance> {
    vec![
        adv(3, "will", Rank::Expert),
        adv(5, "perception", Rank::Expert),
        adv(7, "fortitude", Rank::Expert),
        adv(9, "reflex", Rank::Expert),
        adv(11, "simple weapons", Rank::Expert),
        adv(11, "unarmed", Rank::Expert),
        adv(11, "will", Rank::Master),
        adv(15, "perception", Rank::Master),
        adv(17, "will", Rank::Legendary),
    ]
}

fn thaumaturge_advances() -> Vec<ProficiencyAdvance> {
    vec![
        adv(3, "perception", Rank::Expert),
        adv(3, "will", Rank::Expert),
        adv(5, "simple weapons", Rank::Expert),
        adv(5, "martial weapons", Rank::Expert),
        adv(5, "unarmed", Rank::Expert),
        adv(7, "fortitude", Rank::Expert),
        adv(7, "reflex", Rank::Expert),
        adv(9, "perception", Rank::Master),
        adv(11, "light armor", Rank::Expert),
        adv(11, "medium armor", Rank::Expert),
        adv(11, "unarmored", Rank::Expert),
        adv(13, "simple weapons", Rank::Master),
        adv(13, "martial weapons", Rank::Master),
        adv(13, "unarmed", Rank::Master),
        adv(15, "will", Rank::Master),
        adv(17, "fortitude", Rank::Master),
    ]
}

fn kineticist_advances() -> Vec<ProficiencyAdvance> {
    vec![
        adv(3, "will", Rank::Expert),
        adv(5, "perception", Rank::Expert),
        adv(7, "fortitude", Rank::Master),
        adv(9, "reflex", Rank::Expert),
        adv(11, "light armor", Rank::Expert),
        adv(11, "unarmored", Rank::Expert),
        adv(13, "will", Rank::Master),
        adv(15, "perception", Rank::Master),
        adv(17, "fortitude", Rank::Legendary),
    ]
}

fn gunslinger_advances() -> Vec<ProficiencyAdvance> {
    vec![
        adv(3, "perception", Rank::Expert),
        adv(3, "reflex", Rank::Expert),
        adv(5, "simple weapons", Rank::Expert),
        adv(5, "martial weapons", Rank::Expert),
        adv(5, "unarmed", Rank::Expert),
        adv(7, "perception", Rank::Master),
        adv(7, "fortitude", Rank::Expert),
        adv(9, "reflex", Rank::Master),
        adv(9, "will", Rank::Expert),
        adv(11, "light armor", Rank::Expert),
        adv(11, "medium armor", Rank::Expert),
        adv(11, "unarmored", Rank::Expert),
        adv(13, "simple weapons", Rank::Master),
        adv(13, "martial weapons", Rank::Master),
        adv(13, "unarmed", Rank::Master),
        adv(15, "perception", Rank::Legendary),
        adv(17, "fortitude", Rank::Master),
        adv(19, "reflex", Rank::Legendary),
    ]
}

fn inventor_advances() -> Vec<ProficiencyAdvance> {
    vec![
        adv(3, "will", Rank::Expert),
        adv(5, "simple weapons", Rank::Expert),
        adv(5, "martial weapons", Rank::Expert),
        adv(5, "unarmed", Rank::Expert),
        adv(7, "perception", Rank::Expert),
        adv(7, "fortitude", Rank::Master),
        adv(9, "reflex", Rank::Expert),
        adv(11, "medium armor", Rank::Expert),
        adv(11, "light armor", Rank::Expert),
        adv(11, "unarmored", Rank::Expert),
        adv(13, "simple weapons", Rank::Master),
        adv(13, "martial weapons", Rank::Master),
        adv(13, "unarmed", Rank::Master),
        adv(15, "perception", Rank::Master),
        adv(17, "will", Rank::Master),
    ]
}
