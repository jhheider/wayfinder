//! Extract class data from AON documents.

use wayfinder_core::aon::Document;
use wayfinder_core::render::content::{ContentBlock, parse_content};

use crate::build::choices::{ClassData, ClassFeatureEntry};
use crate::model::abilities::BoostSpec;
use crate::model::proficiencies::Rank;

use crate::build::rules::proficiency::class_proficiency_advances;

use super::boosts::parse_ability;
use super::proficiency::extract_rank_field;
use super::skills::extract_class_skill_grants;
use super::subclass::subclass_category;

/// Extract class HP, key ability, proficiencies, and granted skills.
pub fn extract_class_data(doc: &Document) -> ClassData {
    let hp = doc
        .extra
        .get("hp")
        .and_then(|v| v.as_u64())
        .unwrap_or(0) as u32;
    let key_ability = doc.attribute.first().cloned();

    // Class key ability boost spec
    let mut spec = BoostSpec::default();
    for attr in &doc.attribute {
        if attr.eq_ignore_ascii_case("Free") {
            spec.free += 1;
        } else if let Some(ability) = parse_ability(attr) {
            spec.fixed.push(ability);
        }
    }
    // Multiple fixed abilities → treat as 1 free choice among them
    if spec.fixed.len() > 1 {
        spec.free = 1;
        spec.fixed.clear();
    }

    let mut data = ClassData {
        hp,
        key_ability,
        boost_spec: spec,
        perception: Rank::Untrained,
        fortitude: Rank::Untrained,
        reflex: Rank::Untrained,
        will: Rank::Untrained,
        unarmored: Rank::Untrained,
        light_armor: Rank::Untrained,
        medium_armor: Rank::Untrained,
        heavy_armor: Rank::Untrained,
        simple_weapons: Rank::Untrained,
        martial_weapons: Rank::Untrained,
        unarmed: Rank::Untrained,
        granted_skills: Vec::new(),
        proficiency_advances: Vec::new(),
        subclass_category: None,
        class_features: Vec::new(),
        additional_skill_count: 0,
        spell_slots: Vec::new(),
        tradition: doc.tradition.first().cloned(),
        casting_ability: casting_ability_for(doc),
    };

    if let Some(r) = extract_rank_field(doc, "perception_proficiency") {
        data.perception = r;
    }
    if let Some(r) = extract_rank_field(doc, "fortitude_proficiency") {
        data.fortitude = r;
    }
    if let Some(r) = extract_rank_field(doc, "reflex_proficiency") {
        data.reflex = r;
    }
    if let Some(r) = extract_rank_field(doc, "will_proficiency") {
        data.will = r;
    }

    extract_attack_proficiencies(doc, &mut data);
    extract_defense_proficiencies(doc, &mut data);
    data.granted_skills = extract_class_skill_grants(doc);
    data.additional_skill_count = extract_additional_skill_count(doc);

    // Look up static proficiency advance table for known classes
    if let Some(name) = &doc.name {
        data.proficiency_advances = class_proficiency_advances(name);
        data.subclass_category = subclass_category(name).map(String::from);
    }

    // Extract class features and spell slots from tables.
    // Primary: parse markdown HTML tables. Fallback: JSON content, then text.
    let tables = extract_tables_from_markdown(doc);
    if tables.is_empty() {
        data.class_features = extract_class_features_from_json(doc);
        if data.class_features.is_empty() {
            data.class_features = extract_class_features_from_text(doc);
        }
    } else {
        for (headers, rows) in &tables {
            if is_feature_table(headers) {
                data.class_features = parse_feature_table_rows(rows);
            } else if is_spell_table(headers) {
                data.spell_slots = parse_spell_table(headers, rows);
            }
        }
        // Fallback if markdown didn't yield features
        if data.class_features.is_empty() {
            data.class_features = extract_class_features_from_json(doc);
        }
        if data.class_features.is_empty() {
            data.class_features = extract_class_features_from_text(doc);
        }
    }

    data
}

/// Parse all tables from the document's markdown field.
fn extract_tables_from_markdown(doc: &Document) -> Vec<(Vec<String>, Vec<Vec<String>>)> {
    let Some(md) = doc.markdown.as_deref() else {
        return Vec::new();
    };
    let blocks = parse_content(md, "https://2e.aonprd.com");
    let mut tables = Vec::new();
    for block in &blocks {
        if let ContentBlock::Table { headers, rows } = block {
            tables.push((headers.clone(), rows.clone()));
        }
    }
    tables
}

fn is_feature_table(headers: &[String]) -> bool {
    let has_level = headers
        .iter()
        .any(|h| h.contains("Level") || h.contains("level"));
    let has_features = headers
        .iter()
        .any(|h| h.contains("Feature") || h.contains("feature"));
    has_level && has_features
}

fn is_spell_table(headers: &[String]) -> bool {
    let has_level = headers
        .iter()
        .any(|h| h.contains("Level") || h.contains("level"));
    let has_cantrips = headers
        .iter()
        .any(|h| h.contains("Cantrip") || h.contains("cantrip"));
    has_level && has_cantrips
}

fn parse_feature_table_rows(rows: &[Vec<String>]) -> Vec<ClassFeatureEntry> {
    let mut features = Vec::new();
    for row in rows {
        if row.len() < 2 {
            continue;
        }
        let Ok(level) = row[0].parse::<u8>() else {
            continue;
        };
        for name in row[1].split(',') {
            let name = name.trim();
            if name.is_empty() || is_generic_progression(name) {
                continue;
            }
            let is_feat = is_class_feat_entry(name);
            features.push(ClassFeatureEntry {
                name: name.to_string(),
                level,
                is_feat_slot: is_feat,
            });
        }
    }
    features
}

fn parse_spell_table(
    headers: &[String],
    rows: &[Vec<String>],
) -> Vec<Vec<String>> {
    // Headers: ["Your Level", "Cantrips", "1st", "2nd", ...]
    // Rows: ["1", "5", "2", "—", ...] per level
    // Return: one vec per level, containing slot strings
    let slot_headers: Vec<String> = headers.iter().skip(1).cloned().collect();
    let mut result = Vec::new();
    for row in rows {
        if row.len() < 2 {
            continue;
        }
        // First cell is level, rest are slot counts
        let slots: Vec<String> = row.iter().skip(1).cloned().collect();
        result.push(slots);
    }
    let _ = slot_headers; // available for future use
    result
}

/// Extract class features from the JSON content array (fallback).
fn extract_class_features_from_json(doc: &Document) -> Vec<ClassFeatureEntry> {
    let Some(content) = doc.extra.get("content").and_then(|v| v.as_array()) else {
        return Vec::new();
    };
    let mut features = Vec::new();
    for item in content {
        if item.get("type").and_then(|v| v.as_str()) != Some("table") {
            continue;
        }
        let Some(headers) = item.get("headers").and_then(|v| v.as_array()) else {
            continue;
        };
        // Look for the class features table
        let has_level = headers.iter().any(|h| {
            h.as_str()
                .is_some_and(|s| s.contains("Level") || s.contains("level"))
        });
        let has_features = headers.iter().any(|h| {
            h.as_str()
                .is_some_and(|s| s.contains("Feature") || s.contains("feature"))
        });
        if !has_level || !has_features {
            continue;
        }
        let Some(rows) = item.get("rows").and_then(|v| v.as_array()) else {
            continue;
        };
        for row in rows {
            let Some(cells) = row.as_array() else {
                continue;
            };
            if cells.len() < 2 {
                continue;
            }
            let level_str = cells[0].as_str().unwrap_or("");
            let features_str = cells[1].as_str().unwrap_or("");
            let Ok(level) = level_str.parse::<u8>() else {
                continue;
            };
            for name in features_str.split(',') {
                let name = name.trim();
                if name.is_empty() {
                    continue;
                }
                if is_generic_progression(name) {
                    continue;
                }
                let is_feat = is_class_feat_entry(name);
                features.push(ClassFeatureEntry {
                    name: name.to_string(),
                    level,
                    is_feat_slot: is_feat,
                });
            }
        }
        break; // Only parse the first matching table
    }
    features
}

/// Fallback: parse "Your Level Class Features 1 ... 2 ..." from text field.
fn extract_class_features_from_text(doc: &Document) -> Vec<ClassFeatureEntry> {
    let Some(text) = &doc.text else {
        return Vec::new();
    };
    // Find the table start marker
    let Some(start) = text.find("Your Level") else {
        return Vec::new();
    };
    let table_text = &text[start..];
    // Skip the header line(s) — find first digit that starts a level row
    let mut features = Vec::new();
    let mut current_level: Option<u8> = None;
    let mut current_features = String::new();

    // Split by whitespace-separated tokens, looking for level numbers
    for word in table_text.split_whitespace() {
        if let Ok(lvl) = word.parse::<u8>()
            && (1..=20).contains(&lvl)
            && (current_level.is_none() || lvl > current_level.unwrap())
        {
            // Flush previous level
            if let Some(prev_lvl) = current_level {
                parse_feature_list(&current_features, prev_lvl, &mut features);
            }
            current_level = Some(lvl);
            current_features.clear();
            continue;
        }
        if current_level.is_some() {
            if !current_features.is_empty() {
                current_features.push(' ');
            }
            current_features.push_str(word);
        }
    }
    // Flush last level
    if let Some(lvl) = current_level {
        parse_feature_list(&current_features, lvl, &mut features);
    }
    features
}

fn parse_feature_list(text: &str, level: u8, features: &mut Vec<ClassFeatureEntry>) {
    for name in text.split(',') {
        let name = name.trim();
        if name.is_empty() {
            continue;
        }
        if is_generic_progression(name) {
            continue;
        }
        let is_feat = is_class_feat_entry(name);
        features.push(ClassFeatureEntry {
            name: name.to_string(),
            level,
            is_feat_slot: is_feat,
        });
    }
}

fn is_generic_progression(name: &str) -> bool {
    let lower = name.to_lowercase();
    lower.starts_with("skill feat")
        || lower.starts_with("skill increase")
        || lower.starts_with("general feat")
        || lower.starts_with("ancestry feat")
        || lower.starts_with("attribute boost")
        || lower.starts_with("ancestry and background")
        || lower.starts_with("initial proficienc")
        || is_spell_progression(&lower)
}

/// Match spell-progression entries like "2nd-rank spells", "wizard spellcasting".
fn is_spell_progression(lower: &str) -> bool {
    // "Nth-rank spells" pattern
    if lower.ends_with("-rank spells") {
        return true;
    }
    // "* spellcasting" pattern (e.g. "wizard spellcasting")
    if lower.ends_with(" spellcasting") {
        return true;
    }
    false
}

/// Detect entries like "barbarian feat", "fighter feat", "class feat" that
/// represent class feat slots (handled by progression as BuildSlot::ClassFeat).
pub fn is_class_feat_entry(name: &str) -> bool {
    let lower = name.to_lowercase();
    lower == "class feat"
        || (lower.ends_with(" feat")
            && !lower.contains("skill")
            && !lower.contains("general")
            && !lower.contains("ancestry"))
}

/// Parse "additional skills equal to N + ..." from skill_proficiency entries.
fn extract_additional_skill_count(doc: &Document) -> u8 {
    let Some(arr) = doc
        .extra
        .get("skill_proficiency")
        .and_then(|v| v.as_array())
    else {
        return 0;
    };
    for item in arr {
        if let Some(s) = item.as_str() {
            let lower = s.to_lowercase();
            if lower.contains("additional") {
                // Look for a digit: "N additional skills" or "additional skills equal to N"
                for word in lower.split_whitespace() {
                    if let Ok(n) = word.parse::<u8>() {
                        return n;
                    }
                }
            }
        }
    }
    0
}

fn extract_attack_proficiencies(doc: &Document, data: &mut ClassData) {
    let Some(arr) = doc
        .extra
        .get("attack_proficiency")
        .and_then(|v| v.as_array())
    else {
        return;
    };
    for item in arr {
        if let Some(s) = item.as_str() {
            let lower = s.to_lowercase();
            if lower.contains("unarmed") {
                data.unarmed = Rank::Trained;
            } else if lower.contains("simple") {
                data.simple_weapons = Rank::Trained;
            } else if lower.contains("martial") {
                data.martial_weapons = Rank::Trained;
            }
        }
    }
}

/// Determine the casting ability for a class.
/// Most casters use their key ability; this maps known exceptions.
fn casting_ability_for(doc: &Document) -> Option<String> {
    // Only relevant if the class has a tradition
    if doc.tradition.is_empty() {
        return None;
    }
    // Default: use key ability
    doc.attribute.first().cloned()
}

fn extract_defense_proficiencies(doc: &Document, data: &mut ClassData) {
    let Some(arr) = doc
        .extra
        .get("defense_proficiency")
        .and_then(|v| v.as_array())
    else {
        return;
    };
    for item in arr {
        if let Some(s) = item.as_str() {
            let lower = s.to_lowercase();
            if lower.contains("unarmored") {
                data.unarmored = Rank::Trained;
            } else if lower.contains("light") {
                data.light_armor = Rank::Trained;
            } else if lower.contains("medium") {
                data.medium_armor = Rank::Trained;
            } else if lower.contains("heavy") {
                data.heavy_armor = Rank::Trained;
            }
        }
    }
}
