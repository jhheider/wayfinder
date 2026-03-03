//! Extract granted skills and lore skills from AON documents.

use wayfinder_core::aon::Document;

/// Skills extracted from a document's `skill` or `skill_proficiency` fields.
#[derive(Debug, Clone, Default)]
pub struct SkillGrants {
    /// Standard skills (e.g. "Intimidation", "Athletics").
    pub skills: Vec<String>,
    /// Lore topics (e.g. "Warfare", "Heraldry") — without " Lore" suffix.
    pub lores: Vec<String>,
}

/// Extract skill grants from the `skill` extra field (used by backgrounds).
pub fn extract_skill_grants(doc: &Document) -> SkillGrants {
    let mut result = SkillGrants::default();
    let Some(arr) = doc.extra.get("skill").and_then(|v| v.as_array()) else {
        return result;
    };
    for item in arr {
        if let Some(name) = item.as_str() {
            classify_skill(name, &mut result);
        }
    }
    result
}

/// Extract granted skills from `skill_proficiency` (used by classes).
pub fn extract_class_skill_grants(doc: &Document) -> Vec<String> {
    let Some(arr) = doc
        .extra
        .get("skill_proficiency")
        .and_then(|v| v.as_array())
    else {
        return Vec::new();
    };
    let mut skills = Vec::new();
    for item in arr {
        if let Some(s) = item.as_str() {
            let lower = s.to_lowercase();
            // Skip meta entries like "additional skills equal to ..."
            if !lower.contains("additional")
                && !lower.contains("skill")
                && !lower.contains("number")
            {
                skills.push(s.to_string());
            }
        }
    }
    skills
}

fn classify_skill(name: &str, grants: &mut SkillGrants) {
    if name.ends_with(" Lore") || name.contains("Lore") {
        let lore = name
            .trim_end_matches(" Lore")
            .trim_end_matches("Lore")
            .trim()
            .to_string();
        grants.lores.push(lore);
    } else {
        grants.skills.push(name.to_string());
    }
}
