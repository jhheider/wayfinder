//! Extract ability boost/flaw specs from AON documents.

use wayfinder_core::aon::Document;

use crate::model::abilities::{Ability, BoostSpec};

/// Parse ability boost spec from a document's `attribute` field.
pub fn extract_boost_spec(doc: &Document) -> BoostSpec {
    let mut fixed = Vec::new();
    let mut free = 0u8;
    for attr in &doc.attribute {
        if attr.eq_ignore_ascii_case("Free") {
            free += 1;
        } else if let Some(ability) = parse_ability(attr) {
            fixed.push(ability);
        }
    }
    let mut flaws = Vec::new();
    if let Some(flaw_arr) = doc.extra.get("attribute_flaw").and_then(|v| v.as_array()) {
        for v in flaw_arr {
            if let Some(s) = v.as_str()
                && let Some(ability) = parse_ability(s)
            {
                flaws.push(ability);
            }
        }
    }
    BoostSpec { fixed, free, flaws }
}

/// Parse an ability name (full or abbreviated) to an `Ability`.
pub fn parse_ability(s: &str) -> Option<Ability> {
    match s.to_lowercase().as_str() {
        "strength" | "str" => Some(Ability::Strength),
        "dexterity" | "dex" => Some(Ability::Dexterity),
        "constitution" | "con" => Some(Ability::Constitution),
        "intelligence" | "int" => Some(Ability::Intelligence),
        "wisdom" | "wis" => Some(Ability::Wisdom),
        "charisma" | "cha" => Some(Ability::Charisma),
        _ => None,
    }
}
