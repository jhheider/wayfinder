//! Known AON categories with filterable fields and grouping.

/// All known AON category keys.
pub const ALL_CATEGORIES: &[&str] = &[
    "action",
    "ancestry",
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
    "condition",
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
    "familiar-ability",
    "familiar-specific",
    "feat",
    "hazard",
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
    "lesson",
    "methodology",
    "muse",
    "mystery",
    "mythic-calling",
    "patron",
    "plane",
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
    "source",
    "spell",
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
            "apparition",
            "arcane-school",
            "arcane-thesis",
            "bloodline",
            "cause",
            "conscious-mind",
            "doctrine",
            "draconic-exemplar",
            "druidic-order",
            "eidolon",
            "element",
            "epithet",
            "hunters-edge",
            "hybrid-study",
            "ikon",
            "implement",
            "innovation",
            "instinct",
            "lesson",
            "methodology",
            "muse",
            "mystery",
            "patron",
            "practice",
            "racket",
            "research-field",
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
            "armor",
            "armor-group",
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
            "hazard",
            "language",
            "plane",
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
