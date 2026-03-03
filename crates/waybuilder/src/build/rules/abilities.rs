use crate::model::abilities::{Abilities, Ability, BoostSource};

/// Check if a boost source+ability combination is already present.
pub fn has_boost(abilities: &Abilities, source: &str, ability: Ability) -> bool {
    abilities
        .boosts
        .iter()
        .any(|b| b.source == source && b.ability == ability)
}

/// Add a boost if not already present for that source+ability.
pub fn add_boost(abilities: &mut Abilities, source: &str, ability: Ability) {
    if !has_boost(abilities, source, ability) {
        abilities.boosts.push(BoostSource {
            source: source.to_string(),
            ability,
        });
    }
}

/// Add a flaw for a given source+ability.
pub fn add_flaw(abilities: &mut Abilities, source: &str, ability: Ability) {
    let already = abilities
        .flaws
        .iter()
        .any(|b| b.source == source && b.ability == ability);
    if !already {
        abilities.flaws.push(BoostSource {
            source: source.to_string(),
            ability,
        });
    }
}

/// Remove all boosts and flaws for a given source.
#[allow(dead_code)]
pub fn clear_source(abilities: &mut Abilities, source: &str) {
    abilities.boosts.retain(|b| b.source != source);
    abilities.flaws.retain(|b| b.source != source);
}

/// Count how many boosts a source has contributed.
#[allow(dead_code)]
pub fn count_for_source(abilities: &Abilities, source: &str) -> usize {
    abilities
        .boosts
        .iter()
        .filter(|b| b.source == source)
        .count()
}
