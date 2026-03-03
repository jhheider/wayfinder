//! Extract deity data from AON documents.

use wayfinder_core::aon::Document;

use crate::build::choices::DeityData;
use crate::model::types::{DivineFont, Sanctification};

/// Extract deity mechanical data from a deity document.
pub fn extract_deity_data(doc: &Document) -> DeityData {
    let domains = string_array(&doc.extra, "domain");
    let primary_domains = string_array(&doc.extra, "domain_primary");

    let divine_font = string_array(&doc.extra, "divine_font")
        .iter()
        .filter_map(|s| DivineFont::parse(s))
        .collect();

    let favored_weapon = doc
        .extra
        .get("favored_weapon")
        .and_then(|v| v.as_array())
        .and_then(|a| a.first())
        .and_then(|v| v.as_str())
        .map(String::from);

    let divine_skill = doc
        .extra
        .get("skill")
        .and_then(|v| v.as_array())
        .and_then(|a| a.first())
        .and_then(|v| v.as_str())
        .map(String::from);

    let sanctification = string_array(&doc.extra, "sanctification")
        .iter()
        .filter_map(|s| Sanctification::parse(s))
        .collect();

    let edicts = doc
        .extra
        .get("edict")
        .and_then(|v| v.as_str())
        .map(String::from);

    let anathema = doc
        .extra
        .get("anathema")
        .and_then(|v| v.as_str())
        .map(String::from);

    let cleric_spells = string_array(&doc.extra, "cleric_spell");

    DeityData {
        domains,
        primary_domains,
        divine_font,
        favored_weapon,
        divine_skill,
        sanctification,
        edicts,
        anathema,
        cleric_spells,
    }
}

fn string_array(
    extra: &std::collections::HashMap<String, serde_json::Value>,
    key: &str,
) -> Vec<String> {
    extra
        .get(key)
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default()
}
