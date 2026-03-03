//! Extract equipment base stats from AON documents.

use wayfinder_core::aon::Document;

use crate::model::equipment::{Armor, ResilientRune, Shield, StrikingRune, Weapon};

/// Extract weapon base stats from an AON weapon document.
pub fn extract_weapon(doc: &Document) -> Weapon {
    let extra = &doc.extra;
    let name = doc.name.clone().unwrap_or_default();
    let damage_die = extra
        .get("damage_die")
        .and_then(|v| v.as_u64())
        .unwrap_or(6) as u8;
    // damage_type is an array like ["Slashing","Piercing"]; take first letter
    let damage_type = extra
        .get("damage_type")
        .and_then(|v| v.as_array())
        .and_then(|a| a.first())
        .and_then(|v| v.as_str())
        .map(|s| s.chars().next().unwrap_or('B').to_string())
        .unwrap_or_else(|| "B".to_string());
    let weapon_category = extra
        .get("weapon_category")
        .and_then(|v| v.as_str())
        .unwrap_or("Simple")
        .to_string();
    let weapon_group = extra
        .get("weapon_group")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let weapon_type = extra
        .get("weapon_type")
        .and_then(|v| v.as_str())
        .unwrap_or("Melee")
        .to_string();
    let hands = extra
        .get("hands")
        .and_then(|v| v.as_str())
        .unwrap_or("1")
        .to_string();
    let traits: Vec<String> = extra
        .get("trait_raw")
        .and_then(|v| v.as_array())
        .map(|a| {
            a.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();
    let range = extra
        .get("range")
        .and_then(|v| v.as_u64())
        .unwrap_or(0) as u32;

    Weapon {
        name,
        potency: 0,
        striking: StrikingRune::None,
        property_runes: Vec::new(),
        material: None,
        damage_die,
        damage_type,
        weapon_category,
        weapon_group,
        weapon_type,
        hands,
        traits,
        range,
    }
}

/// Extract armor base stats from an AON armor document.
pub fn extract_armor(doc: &Document) -> Armor {
    let extra = &doc.extra;
    let name = doc.name.clone().unwrap_or_default();
    let ac_bonus = extra
        .get("ac")
        .and_then(|v| v.as_i64())
        .unwrap_or(0) as i32;
    let dex_cap = extra
        .get("dex_cap")
        .and_then(|v| v.as_i64())
        .map(|v| v as i32);
    let armor_category = extra
        .get("armor_category")
        .and_then(|v| v.as_str())
        .unwrap_or("Unarmored")
        .to_string();
    let armor_group = extra
        .get("armor_group")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let check_penalty = extra
        .get("check_penalty")
        .and_then(|v| v.as_i64())
        .unwrap_or(0) as i32;
    let speed_penalty = extra
        .get("speed_penalty")
        .and_then(|v| v.as_i64())
        .unwrap_or(0) as i32;
    let strength = extra
        .get("strength")
        .and_then(|v| v.as_u64())
        .unwrap_or(0) as u32;

    Armor {
        name,
        potency: 0,
        resilient: ResilientRune::None,
        property_runes: Vec::new(),
        material: None,
        worn: true,
        ac_bonus,
        dex_cap,
        armor_category,
        armor_group,
        check_penalty,
        speed_penalty,
        strength,
    }
}

/// Extract shield base stats from an AON shield document.
pub fn extract_shield(doc: &Document) -> Shield {
    let extra = &doc.extra;
    let name = doc.name.clone().unwrap_or_default();
    let ac_bonus = extra
        .get("ac")
        .and_then(|v| v.as_i64())
        .unwrap_or(2) as i32;
    let hardness = extra
        .get("hardness")
        .and_then(|v| v.as_i64())
        .unwrap_or(0) as i32;
    let hp = extra
        .get("hp")
        .and_then(|v| v.as_i64())
        .unwrap_or(0) as i32;

    Shield {
        name,
        potency: 0,
        resilient: ResilientRune::None,
        property_runes: Vec::new(),
        raised: false,
        ac_bonus,
        hardness,
        hp,
    }
}
