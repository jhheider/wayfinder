use std::collections::BTreeMap;
use std::path::PathBuf;

use anyhow::{Context, Result};
use serde_json::{Value, json};

use crate::build::choices::BuildChoices;
use crate::model::abilities::Ability;
use crate::model::character::Character;

/// Export character in Pathbuilder-compatible JSON format.
pub fn export_pathbuilder(choices: &BuildChoices, character: &Character) -> Result<PathBuf> {
    let dir = super::save::characters_dir();
    std::fs::create_dir_all(&dir).context("Failed to create characters directory")?;

    let filename = sanitize(&character.name);
    let path = dir.join(format!("{filename}_pathbuilder.json"));

    let json = build_pathbuilder_json(choices, character);
    let output =
        serde_json::to_string_pretty(&json).context("Failed to serialize Pathbuilder JSON")?;
    std::fs::write(&path, output).context("Failed to write export file")?;
    Ok(path)
}

fn build_pathbuilder_json(choices: &BuildChoices, ch: &Character) -> Value {
    let breakdown = build_ability_breakdown(choices, ch);
    let abilities = json!({
        "str": ch.abilities.score(Ability::Strength),
        "dex": ch.abilities.score(Ability::Dexterity),
        "con": ch.abilities.score(Ability::Constitution),
        "int": ch.abilities.score(Ability::Intelligence),
        "wis": ch.abilities.score(Ability::Wisdom),
        "cha": ch.abilities.score(Ability::Charisma),
        "breakdown": breakdown,
    });

    let proficiencies = build_proficiencies(ch);
    let feats = build_feats(ch);
    let lores: Vec<Value> = ch
        .proficiencies
        .lores
        .iter()
        .map(|(name, rank)| json!([name, rank.bonus()]))
        .collect();

    json!({
        "success": true,
        "build": {
            "name": ch.name,
            "class": ch.class.as_deref().unwrap_or(""),
            "dualClass": null,
            "level": ch.level,
            "ancestry": ch.ancestry.as_deref().unwrap_or(""),
            "heritage": ch.heritage.as_deref().unwrap_or(""),
            "background": ch.background.as_deref().unwrap_or(""),
            "alignment": ch.alignment.as_deref().unwrap_or(""),
            "gender": "",
            "age": "",
            "deity": ch.deity.as_deref().unwrap_or(""),
            "sizeName": ch.size.as_deref().unwrap_or("Medium"),
            "keyability": ch.key_ability.as_deref().unwrap_or(""),
            "languages": ch.languages,
            "abilities": abilities,
            "attributes": {
                "ancestryhp": ch.ancestry_hp,
                "classhp": ch.class_hp,
                "bonushp": 0,
                "bonushpPerLevel": 0,
                "speed": ch.speed,
                "speedBonus": 0,
            },
            "proficiencies": proficiencies,
            "feats": feats,
            "specials": [],
            "lores": lores,
            "equipment": build_equipment_items(choices),
            "weapons": build_weapons(choices, ch),
            "money": {
                "cp": choices.equipment.money.cp,
                "sp": choices.equipment.money.sp,
                "gp": choices.equipment.money.gp,
                "pp": choices.equipment.money.pp,
            },
            "armor": build_armor_export(choices),
            "spellCasters": [],
            "focusPoints": ch.focus_points,
            "focus": build_focus_export(choices),
            "formula": [],
            "pets": [],
            "familiars": [],
        }
    })
}

fn build_ability_breakdown(choices: &BuildChoices, ch: &Character) -> Value {
    let ancestry_fixed = choices
        .ancestry_data
        .as_ref()
        .map(|d| &d.boost_spec.fixed)
        .cloned()
        .unwrap_or_default();

    let mut ancestry_free: Vec<String> = Vec::new();
    let mut ancestry_boosts: Vec<String> = Vec::new();
    let mut ancestry_flaws: Vec<String> = Vec::new();
    let mut background_boosts: Vec<String> = Vec::new();
    let mut class_boosts: Vec<String> = Vec::new();
    let mut levelled: BTreeMap<String, Vec<String>> = BTreeMap::new();

    for boost in &ch.abilities.boosts {
        let abbr = boost.ability.abbr().to_string();
        match boost.source.as_str() {
            "ancestry" => {
                if ancestry_fixed.contains(&boost.ability) {
                    ancestry_boosts.push(abbr);
                } else {
                    ancestry_free.push(abbr);
                }
            }
            "background" => background_boosts.push(abbr),
            "class" => class_boosts.push(abbr),
            s if s.starts_with("level_") => {
                let lvl = s.trim_start_matches("level_");
                levelled.entry(lvl.to_string()).or_default().push(abbr);
            }
            _ => {}
        }
    }
    for flaw in &ch.abilities.flaws {
        let abbr = flaw.ability.abbr().to_string();
        if flaw.source.contains("ancestry") {
            ancestry_flaws.push(abbr);
        }
    }

    json!({
        "ancestryFree": ancestry_free,
        "ancestryBoosts": ancestry_boosts,
        "ancestryFlaws": ancestry_flaws,
        "backgroundBoosts": background_boosts,
        "classBoosts": class_boosts,
        "mapLevelledBoosts": levelled,
    })
}

fn build_proficiencies(ch: &Character) -> Value {
    let p = &ch.proficiencies;
    let mut obj = json!({
        "classDC": 0,
        "perception": p.perception.bonus(),
        "fortitude": p.fortitude.bonus(),
        "reflex": p.reflex.bonus(),
        "will": p.will.bonus(),
        "heavy": p.heavy_armor.bonus(),
        "medium": p.medium_armor.bonus(),
        "light": p.light_armor.bonus(),
        "unarmored": p.unarmored.bonus(),
        "advanced": 0,
        "martial": p.martial_weapons.bonus(),
        "simple": p.simple_weapons.bonus(),
        "unarmed": p.unarmed.bonus(),
        "castingArcane": 0,
        "castingDivine": 0,
        "castingOccult": 0,
        "castingPrimal": 0,
    });

    // Add each standard skill
    let skills_map = obj.as_object_mut().unwrap();
    for &skill in crate::model::proficiencies::SKILLS {
        let key = skill.to_lowercase();
        let rank = p.skill_rank(skill);
        skills_map.insert(key, json!(rank.bonus()));
    }

    obj
}

fn build_feats(ch: &Character) -> Vec<Value> {
    ch.feats
        .iter()
        .map(|f| {
            let type_label = match f.feat_type {
                crate::model::feat::FeatType::Ancestry => "Ancestry Feat",
                crate::model::feat::FeatType::Class => "Class Feat",
                crate::model::feat::FeatType::General => "General Feat",
                crate::model::feat::FeatType::Skill => "Skill Feat",
                crate::model::feat::FeatType::Heritage => "Heritage",
                crate::model::feat::FeatType::Bonus => "Awarded Feat",
            };
            json!([
                f.name,
                null,
                type_label,
                f.level,
                f.slot,
                "standardChoice",
                null,
            ])
        })
        .collect()
}

fn build_weapons(choices: &BuildChoices, ch: &Character) -> Vec<Value> {
    choices
        .equipment
        .weapons
        .iter()
        .enumerate()
        .map(|(i, w)| {
            let attack = ch.weapon_attacks.get(i);
            json!({
                "name": w.name,
                "qty": 1,
                "prof": w.weapon_category.to_lowercase(),
                "die": w.damage_die_str(),
                "pot": w.potency,
                "str": w.striking.extra_dice(),
                "mat": w.material.as_deref().unwrap_or(""),
                "display": w.display_name(),
                "runes": w.property_runes,
                "damageType": w.damage_type,
                "attack": attack.map(|a| a.attack_bonus).unwrap_or(0),
                "damageBonus": 0,
            })
        })
        .collect()
}

fn build_armor_export(choices: &BuildChoices) -> Vec<Value> {
    choices
        .equipment
        .armor
        .iter()
        .map(|a| {
            json!({
                "name": a.name,
                "qty": 1,
                "prof": a.armor_category.to_lowercase(),
                "pot": a.potency,
                "res": a.resilient.bonus(),
                "mat": a.material.as_deref().unwrap_or(""),
                "display": a.display_name(),
                "worn": a.worn,
                "runes": a.property_runes,
            })
        })
        .collect()
}

fn build_equipment_items(choices: &BuildChoices) -> Vec<Value> {
    choices
        .equipment
        .items
        .iter()
        .map(|item| {
            let invested = if item.invested { "Invested" } else { "" };
            json!([item.name, item.quantity, invested])
        })
        .collect()
}

fn build_focus_export(choices: &BuildChoices) -> Value {
    let spells: Vec<Value> = choices
        .focus_spells
        .iter()
        .map(|s| json!({"name": s.name, "rank": s.rank}))
        .collect();
    json!({ "spells": spells })
}

fn sanitize(name: &str) -> String {
    let s: String = name
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect();
    if s.is_empty() { "unnamed".into() } else { s }
}
