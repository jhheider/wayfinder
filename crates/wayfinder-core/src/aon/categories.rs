//! Known AON categories with filterable fields and grouping.

use super::client::GameSystem;

/// All known AON category keys (the union of the PF2e and SF2e index category
/// sets; see [`PF2E_ONLY`] / [`SF2E_ONLY`] for the ones exclusive to a system).
pub const ALL_CATEGORIES: &[&str] = &[
    "action",
    "ammunition",
    "ancestry",
    "anchor",
    "animal-companion",
    "animal-companion-advanced",
    "animal-companion-specialization",
    "animal-companion-unique",
    "apparition",
    "arcane-school",
    "arcane-thesis",
    "archetype",
    "armor",
    "armor-group",
    "article",
    "background",
    "bloodline",
    "campsite-meal",
    "category-page",
    "cause",
    "class",
    "class-feature",
    "class-kit",
    "class-sample",
    "computer",
    "condition",
    "connection",
    "conscious-mind",
    "creature",
    "creature-ability",
    "creature-adjustment",
    "creature-family",
    "creature-theme-template",
    "cult-activity",
    "curse",
    "deity",
    "deity-category",
    "deviant-ability-classification",
    "disease",
    "doctrine",
    "domain",
    "draconic-exemplar",
    "druidic-order",
    "eidolon",
    "element",
    "epithet",
    "equipment",
    "faction",
    "familiar-ability",
    "familiar-specific",
    "feat",
    "fighting-style",
    "hazard",
    "hazard-family",
    "hellknight-order",
    "heritage",
    "hunters-edge",
    "hybrid-study",
    "ikon",
    "implement",
    "innovation",
    "instinct",
    "item-bonus",
    "kingdom-event",
    "kingdom-structure",
    "language",
    "leadership-style",
    "lesson",
    "methodology",
    "muse",
    "mystery",
    "mythic-calling",
    "paradox",
    "patron",
    "plane",
    "planet",
    "practice",
    "racket",
    "relic",
    "research-field",
    "ritual",
    "rules",
    "set-relic",
    "shield",
    "sidebar",
    "siege-weapon",
    "skill",
    "skill-general-action",
    "solar-manifestation",
    "source",
    "specialization",
    "spell",
    "starship-scene",
    "style",
    "subconscious-mind",
    "tactic",
    "tenet",
    "tradition",
    "trait",
    "vehicle",
    "warfare-army",
    "warfare-tactic",
    "way",
    "weapon",
    "weapon-group",
    "weather-hazard",
];

/// Categories present only in the PF2e index (`aon70`), not in SF2e (`aonsf10`).
/// Seeded from a live category aggregation of both indices (2026-07-10).
/// Everything in [`ALL_CATEGORIES`] that is in neither this list nor
/// [`SF2E_ONLY`] is shared by both game systems.
pub const PF2E_ONLY: &[&str] = &[
    "animal-companion",
    "animal-companion-advanced",
    "animal-companion-specialization",
    "animal-companion-unique",
    "apparition",
    "arcane-school",
    "arcane-thesis",
    "article",
    "bloodline",
    "campsite-meal",
    "cause",
    "class-kit",
    "class-sample",
    "conscious-mind",
    "creature-theme-template",
    "cult-activity",
    "deviant-ability-classification",
    "doctrine",
    "draconic-exemplar",
    "druidic-order",
    "eidolon",
    "element",
    "epithet",
    "familiar-ability",
    "familiar-specific",
    "hellknight-order",
    "hunters-edge",
    "hybrid-study",
    "ikon",
    "implement",
    "innovation",
    "instinct",
    "kingdom-event",
    "kingdom-structure",
    "lesson",
    "methodology",
    "muse",
    "mystery",
    "mythic-calling",
    "patron",
    "practice",
    "racket",
    "relic",
    "research-field",
    "set-relic",
    "siege-weapon",
    "style",
    "subconscious-mind",
    "tactic",
    "tenet",
    "warfare-army",
    "warfare-tactic",
    "way",
    "weather-hazard",
];

/// Categories present only in the SF2e index (`aonsf10`), not in PF2e (`aon70`).
/// Seeded from the same 2026-07-10 aggregation as [`PF2E_ONLY`].
pub const SF2E_ONLY: &[&str] = &[
    "ammunition",
    "anchor",
    "computer",
    "connection",
    "faction",
    "fighting-style",
    "hazard-family",
    "leadership-style",
    "paradox",
    "planet",
    "solar-manifestation",
    "specialization",
    "starship-scene",
];

/// Whether `category` exists in the given game system's index. Unknown
/// categories (not in [`ALL_CATEGORIES`]) return `false`.
pub fn category_in_system(category: &str, system: GameSystem) -> bool {
    if !ALL_CATEGORIES.contains(&category) {
        return false;
    }
    match system {
        GameSystem::Pathfinder => !SF2E_ONLY.contains(&category),
        GameSystem::Starfinder => !PF2E_ONLY.contains(&category),
    }
}

/// The categories available in the given game system, sorted.
pub fn categories_for(system: GameSystem) -> Vec<&'static str> {
    ALL_CATEGORIES
        .iter()
        .copied()
        .filter(|c| category_in_system(c, system))
        .collect()
}

/// Filterable fields for a category (excludes common metadata).
/// Returns `None` for unknown categories.
pub fn filterable_fields(category: &str) -> Option<&'static [&'static str]> {
    Some(match category {
        "spell" => &[
            "actions",
            "bloodline",
            "component",
            "deity",
            "element",
            "heighten_group",
            "level",
            "patron_theme",
            "rarity",
            "saving_throw",
            "school",
            "spell_type",
            "tradition",
            "trait",
        ],
        "feat" => &["feat", "level", "rarity", "trait"],
        "deity" => &[
            "alignment",
            "area_of_concern",
            "attribute",
            "cleric_spell",
            "deity",
            "divine_font",
            "domain",
            "domain_alternate",
            "domain_primary",
            "favored_weapon",
            "follower_alignment",
            "pantheon",
            "sanctification",
            "skill",
            "spell",
            "trait",
        ],
        "ancestry" => &[
            "attribute",
            "attribute_flaw",
            "hp",
            "language",
            "rarity",
            "size",
            "trait",
        ],
        "class" => &[
            "attack_proficiency",
            "attribute",
            "defense_proficiency",
            "hp",
            "rarity",
            "skill_proficiency",
        ],
        "creature" => &[
            "alignment",
            "creature_ability",
            "hp",
            "immunity",
            "language",
            "level",
            "rarity",
            "size",
            "skill",
            "strongest_save",
            "trait",
            "weakest_save",
        ],
        "equipment" => &["actions", "level", "rarity", "trait"],
        "background" => &["attribute", "feat", "rarity", "skill"],
        "heritage" => &["rarity"],
        "archetype" => &["archetype", "archetype_category", "level", "rarity"],
        "action" => &["actions", "rarity"],
        "hazard" => &["level", "rarity", "trait"],
        "condition" => &["rarity"],
        "ritual" => &["level", "rarity", "school", "trait"],
        "domain" => &["deity", "domain", "spell"],
        "weapon" => &["damage_type", "deity", "level", "rarity", "trait"],
        "armor" => &["level", "rarity"],
        "shield" => &["hp", "level", "rarity"],
        "vehicle" => &["hp", "level", "rarity", "size", "trait"],
        "disease" => &["level", "rarity", "saving_throw", "trait"],
        "curse" => &["level", "rarity", "school", "trait"],
        "bloodline" => &["bloodline", "rarity", "skill", "spell", "tradition"],
        "mystery" => &["domain", "rarity", "skill", "spell"],
        "eidolon" => &[
            "alignment",
            "language",
            "rarity",
            "size",
            "skill",
            "tradition",
            "trait",
        ],
        "patron" => &["rarity", "skill", "spell", "tradition"],
        "relic" => &["aspect", "element", "rarity", "school", "trait"],
        "familiar-ability" => &["rarity"],
        "familiar-specific" => &["familiar_ability", "rarity", "trait"],
        "animal-companion" => &["hp", "level", "rarity", "size", "skill", "trait"],
        "plane" => &["alignment", "rarity", "trait"],
        "language" => &["rarity"],
        "trait" => &["rarity", "trait", "trait_group"],
        "class-feature" => &["level", "rarity"],
        "draconic-exemplar" => &["rarity", "skill", "spell", "tradition"],
        _ => return None,
    })
}

/// Group structure for hierarchical display.
pub struct CategoryGroup {
    pub name: &'static str,
    pub members: &'static [&'static str],
}

/// Categories organized into display groups.
pub const CATEGORY_GROUPS: &[CategoryGroup] = &[
    CategoryGroup {
        name: "Character Building",
        members: &[
            "ancestry",
            "heritage",
            "background",
            "class",
            "class-feature",
            "class-kit",
            "class-sample",
            "feat",
        ],
    },
    CategoryGroup {
        name: "Class Options",
        members: &[
            "anchor",
            "apparition",
            "arcane-school",
            "arcane-thesis",
            "bloodline",
            "cause",
            "connection",
            "conscious-mind",
            "doctrine",
            "draconic-exemplar",
            "druidic-order",
            "eidolon",
            "element",
            "epithet",
            "fighting-style",
            "hunters-edge",
            "hybrid-study",
            "ikon",
            "implement",
            "innovation",
            "instinct",
            "leadership-style",
            "lesson",
            "methodology",
            "muse",
            "mystery",
            "paradox",
            "patron",
            "practice",
            "racket",
            "research-field",
            "solar-manifestation",
            "specialization",
            "style",
            "subconscious-mind",
            "tactic",
            "way",
        ],
    },
    CategoryGroup {
        name: "Magic",
        members: &["spell", "ritual", "domain", "tradition"],
    },
    CategoryGroup {
        name: "Equipment",
        members: &[
            "equipment",
            "ammunition",
            "armor",
            "armor-group",
            "computer",
            "shield",
            "weapon",
            "weapon-group",
            "item-bonus",
            "siege-weapon",
        ],
    },
    CategoryGroup {
        name: "Creatures & Companions",
        members: &[
            "creature",
            "creature-ability",
            "creature-adjustment",
            "creature-family",
            "creature-theme-template",
            "animal-companion",
            "animal-companion-advanced",
            "animal-companion-specialization",
            "animal-companion-unique",
            "familiar-ability",
            "familiar-specific",
        ],
    },
    CategoryGroup {
        name: "World & Lore",
        members: &[
            "deity",
            "deity-category",
            "action",
            "archetype",
            "condition",
            "disease",
            "curse",
            "faction",
            "hazard",
            "hazard-family",
            "language",
            "plane",
            "planet",
            "relic",
            "set-relic",
            "skill",
            "skill-general-action",
            "trait",
            "vehicle",
        ],
    },
    CategoryGroup {
        name: "Rules & Reference",
        members: &[
            "rules",
            "article",
            "background",
            "campsite-meal",
            "category-page",
            "cult-activity",
            "deviant-ability-classification",
            "hellknight-order",
            "kingdom-event",
            "kingdom-structure",
            "mythic-calling",
            "sidebar",
            "starship-scene",
            "source",
            "tenet",
            "warfare-army",
            "warfare-tactic",
            "weather-hazard",
        ],
    },
];

/// Find the closest matching category for a given input.
/// Returns `None` if no reasonable match exists.
pub fn suggest_category(input: &str) -> Option<&'static str> {
    let input = input.to_lowercase();

    // Exact match
    if let Some(&cat) = ALL_CATEGORIES.iter().find(|&&c| c == input) {
        return Some(cat);
    }

    // Prefix match
    let prefix_matches: Vec<&&str> = ALL_CATEGORIES
        .iter()
        .filter(|&&c| c.starts_with(&input))
        .collect();
    if prefix_matches.len() == 1 {
        return Some(prefix_matches[0]);
    }

    // Substring match
    let substr_matches: Vec<&&str> = ALL_CATEGORIES
        .iter()
        .filter(|&&c| c.contains(input.as_str()))
        .collect();
    if substr_matches.len() == 1 {
        return Some(substr_matches[0]);
    }

    // Edit distance (simple Levenshtein)
    let mut best = None;
    let mut best_dist = usize::MAX;
    for &cat in ALL_CATEGORIES {
        let d = edit_distance(&input, cat);
        if d < best_dist {
            best_dist = d;
            best = Some(cat);
        }
    }
    // Only suggest if distance is reasonable (≤ 3 edits)
    if best_dist <= 3 {
        return best;
    }

    None
}

/// Emoji icon for a category.
pub fn category_icon(cat: &str) -> &'static str {
    match cat {
        "spell" | "ritual" => "✨",
        "feat" => "⭐",
        "deity" => "🔱",
        "ancestry" => "🧬",
        "heritage" => "🌿",
        "class" => "⚔️",
        "creature" => "🐉",
        "equipment" | "armor" | "weapon" | "shield" => "🛡️",
        "background" => "📜",
        "archetype" => "🏛️",
        "action" => "⚡",
        "condition" | "disease" | "curse" => "💀",
        "hazard" => "⚠️",
        "trait" => "🏷️",
        "domain" => "🌐",
        _ => "📄",
    }
}

fn edit_distance(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let mut dp = vec![vec![0usize; b.len() + 1]; a.len() + 1];
    for (i, row) in dp.iter_mut().enumerate().take(a.len() + 1) {
        row[0] = i;
    }
    for (j, val) in dp[0].iter_mut().enumerate().take(b.len() + 1) {
        *val = j;
    }
    for i in 1..=a.len() {
        for j in 1..=b.len() {
            let cost = if a[i - 1] == b[j - 1] { 0 } else { 1 };
            dp[i][j] = (dp[i - 1][j] + 1)
                .min(dp[i][j - 1] + 1)
                .min(dp[i - 1][j - 1] + cost);
        }
    }
    dp[a.len()][b.len()]
}
