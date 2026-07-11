use wayfinder_core::aon::GameSystem;
use wayfinder_core::aon::categories::{
    ALL_CATEGORIES, CATEGORY_GROUPS, PF2E_ONLY, SF2E_ONLY, categories_for, category_in_system,
    filterable_fields, suggest_category,
};

#[test]
fn all_categories_sorted() {
    let mut sorted = ALL_CATEGORIES.to_vec();
    sorted.sort();
    assert_eq!(ALL_CATEGORIES, &sorted[..]);
}

#[test]
fn suggest_exact_match() {
    assert_eq!(suggest_category("deity"), Some("deity"));
    assert_eq!(suggest_category("spell"), Some("spell"));
}

#[test]
fn suggest_prefix_match() {
    assert_eq!(suggest_category("dei"), Some("deity"));
    assert_eq!(suggest_category("spe"), Some("spell"));
}

#[test]
fn suggest_edit_distance() {
    assert_eq!(suggest_category("diety"), Some("deity"));
    assert_eq!(suggest_category("spel"), Some("spell"));
    assert_eq!(suggest_category("feat"), Some("feat"));
}

#[test]
fn suggest_returns_none_for_garbage() {
    assert_eq!(suggest_category("xyzzyplugh"), None);
}

#[test]
fn filterable_fields_known_category() {
    let fields = filterable_fields("spell").unwrap();
    assert!(fields.contains(&"tradition"));
    assert!(fields.contains(&"level"));
}

#[test]
fn filterable_fields_unknown_category() {
    assert!(filterable_fields("nonexistent").is_none());
}

#[test]
fn all_group_members_are_valid_categories() {
    for group in CATEGORY_GROUPS {
        for &member in group.members {
            assert!(
                ALL_CATEGORIES.contains(&member),
                "Group '{}' has unknown category '{}'",
                group.name,
                member
            );
        }
    }
}

#[test]
fn system_only_lists_are_disjoint_and_known() {
    for &cat in PF2E_ONLY {
        assert!(ALL_CATEGORIES.contains(&cat), "{cat} missing from ALL");
        assert!(!SF2E_ONLY.contains(&cat), "{cat} in both only-lists");
    }
    for &cat in SF2E_ONLY {
        assert!(ALL_CATEGORIES.contains(&cat), "{cat} missing from ALL");
    }
}

#[test]
fn sf2e_list_excludes_pf2e_only_categories() {
    let sf = categories_for(GameSystem::Starfinder);
    for &cat in PF2E_ONLY {
        assert!(
            !sf.contains(&cat),
            "PF2e-only category '{cat}' leaked into the SF2e list"
        );
    }
    // Spot-check known PF2e-only offenders from the bug report.
    for cat in ["hellknight-order", "kingdom-event", "eidolon", "bloodline"] {
        assert!(!category_in_system(cat, GameSystem::Starfinder));
        assert!(category_in_system(cat, GameSystem::Pathfinder));
    }
}

#[test]
fn pf2e_list_excludes_sf2e_only_categories() {
    let pf = categories_for(GameSystem::Pathfinder);
    for &cat in SF2E_ONLY {
        assert!(
            !pf.contains(&cat),
            "SF2e-only category '{cat}' leaked into the PF2e list"
        );
    }
    // Spot-check known SF2e-only categories.
    for cat in [
        "starship-scene",
        "planet",
        "solar-manifestation",
        "computer",
    ] {
        assert!(!category_in_system(cat, GameSystem::Pathfinder));
        assert!(category_in_system(cat, GameSystem::Starfinder));
    }
}

#[test]
fn per_system_counts_match_live_aggregation() {
    // Measured against the live indices on 2026-07-10: PF2e 93, SF2e 52.
    assert_eq!(categories_for(GameSystem::Pathfinder).len(), 93);
    assert_eq!(categories_for(GameSystem::Starfinder).len(), 52);
}

#[test]
fn shared_categories_appear_in_both_systems() {
    // A category shared by both systems (not in either only-list).
    assert!(category_in_system("spell", GameSystem::Pathfinder));
    assert!(category_in_system("spell", GameSystem::Starfinder));
}

#[test]
fn unknown_category_not_in_any_system() {
    assert!(!category_in_system("nonexistent", GameSystem::Pathfinder));
    assert!(!category_in_system("nonexistent", GameSystem::Starfinder));
}

#[test]
fn every_sf2e_category_is_displayed_in_a_group() {
    // `wf --sf2e categories` should show the complete SF2e set.
    let grouped: std::collections::HashSet<&str> = CATEGORY_GROUPS
        .iter()
        .flat_map(|g| g.members.iter().copied())
        .filter(|&c| category_in_system(c, GameSystem::Starfinder))
        .collect();
    for &cat in &categories_for(GameSystem::Starfinder) {
        assert!(
            grouped.contains(cat),
            "SF2e category '{cat}' not in any group"
        );
    }
}
