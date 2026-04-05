//! Extract proficiency grants and advances from AON document fields.

use wayfinder_core::aon::Document;

use crate::model::proficiencies::Rank;

use super::ProficiencyGrant;

/// A proficiency rank-up at a specific level.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ProficiencyAdvance {
    pub level: u8,
    pub target: String,
    pub rank: Rank,
}

/// Parse a rank string ("trained", "expert", etc.) into a `Rank`.
pub fn parse_rank(s: &str) -> Option<Rank> {
    match s.to_lowercase().as_str() {
        "trained" => Some(Rank::Trained),
        "expert" => Some(Rank::Expert),
        "master" => Some(Rank::Master),
        "legendary" => Some(Rank::Legendary),
        _ => None,
    }
}

/// Extract a proficiency rank from a named extra field on a document.
pub fn extract_rank_field(doc: &Document, field: &str) -> Option<Rank> {
    doc.extra
        .get(field)
        .and_then(|v| v.as_str())
        .and_then(parse_rank)
}

/// Extract proficiency grants from text patterns like
/// "trained in Perception", "expert in Fortitude".
#[allow(dead_code)]
pub fn extract_proficiency_grants(text: &str) -> Vec<ProficiencyGrant> {
    let mut grants = Vec::new();
    let lower = text.to_lowercase();
    for rank_word in &["trained", "expert", "master", "legendary"] {
        let prefix = format!("{rank_word} in ");
        let Some(rank) = parse_rank(rank_word) else {
            continue;
        };
        for (idx, _) in lower.match_indices(&prefix) {
            let after = &text[idx + prefix.len()..];
            let target = after
                .split([',', '.', ';', '\n'])
                .next()
                .unwrap_or("")
                .trim();
            if !target.is_empty() {
                grants.push(ProficiencyGrant {
                    target: target.to_string(),
                    rank,
                });
            }
        }
    }
    grants
}

/// Extract proficiency advances from class feature text.
/// Looks for patterns like:
/// - "Your proficiency rank for X increases to Y"
/// - "You become an expert in X"
/// - "Your X proficiency increases to Y"
#[allow(dead_code)]
pub fn extract_proficiency_advances(text: &str, level: u8) -> Vec<ProficiencyAdvance> {
    let mut advances = Vec::new();
    let lower = text.to_lowercase();

    // Pattern: "become(s) (a/an) <rank> in <target>"
    for rank_word in &["trained", "expert", "master", "legendary"] {
        let Some(rank) = parse_rank(rank_word) else {
            continue;
        };
        for pattern in &[
            format!("become an {rank_word} in "),
            format!("become a {rank_word} in "),
            format!("becomes an {rank_word} in "),
            format!("becomes a {rank_word} in "),
            format!("become {rank_word} in "),
            format!("becomes {rank_word} in "),
        ] {
            for (idx, _) in lower.match_indices(pattern.as_str()) {
                let after = &text[idx + pattern.len()..];
                let target = after
                    .split([',', '.', ';', '\n'])
                    .next()
                    .unwrap_or("")
                    .trim();
                if !target.is_empty() {
                    advances.push(ProficiencyAdvance {
                        level,
                        target: target.to_string(),
                        rank,
                    });
                }
            }
        }

        // Pattern: "increases to <rank>"
        let inc_pat = format!("increases to {rank_word}");
        for (idx, _) in lower.match_indices(&inc_pat) {
            // Look backwards for "proficiency rank for <target>" or
            // "your <target> proficiency"
            let before = &lower[..idx];
            let target = extract_target_before_increases(before);
            if let Some(t) = target {
                // Use original case from text
                let t_start = before.rfind(&t).unwrap_or(0);
                let original = &text[t_start..t_start + t.len()];
                advances.push(ProficiencyAdvance {
                    level,
                    target: original.trim().to_string(),
                    rank,
                });
            }
        }
    }
    advances
}

fn extract_target_before_increases(before: &str) -> Option<String> {
    // "proficiency rank for <target> " → target is between "for " and end
    if let Some(pos) = before.rfind("proficiency rank for ") {
        let after_for = &before[pos + "proficiency rank for ".len()..];
        let target = after_for.trim().trim_end_matches(" proficiency");
        if !target.is_empty() {
            return Some(target.to_string());
        }
    }
    // "<target> proficiency " at end
    if let Some(pos) = before.rfind(" proficiency") {
        let chunk = &before[..pos];
        // Take last phrase (after "your " or sentence start)
        let target = chunk
            .rsplit(['.', ',', ';'])
            .next()
            .unwrap_or(chunk)
            .trim()
            .trim_start_matches("your ")
            .trim();
        if !target.is_empty() {
            return Some(target.to_string());
        }
    }
    None
}
