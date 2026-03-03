//! Extract background data from AON documents.

use wayfinder_core::aon::Document;

use crate::build::choices::BackgroundData;

use super::boosts::extract_boost_spec;
use super::skills::extract_skill_grants;

/// Extract background feat, skill grants, and boost spec.
pub fn extract_background_data(doc: &Document) -> BackgroundData {
    let mut spec = extract_boost_spec(doc);
    // Backgrounds list 2 attribute options + 1 free
    if spec.fixed.len() == 2 && spec.free == 0 {
        spec.free = 1;
    }

    let grants = extract_skill_grants(doc);

    let granted_feat = doc
        .extra
        .get("feat")
        .and_then(|v| v.as_array())
        .and_then(|arr| arr.first())
        .and_then(|v| v.as_str())
        .map(String::from);

    BackgroundData {
        boost_spec: spec,
        granted_skills: grants.skills,
        granted_lores: grants.lores,
        granted_feat,
    }
}
