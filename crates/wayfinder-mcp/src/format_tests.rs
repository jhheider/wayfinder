use super::{format_alternatives, format_detail, format_summary};
use serde_json::json;
use wayfinder_core::aon::Document;

fn doc(v: serde_json::Value) -> Document {
    serde_json::from_value(v).unwrap()
}

#[test]
fn summary_includes_type_level_traits_rarity_summary_url() {
    let d = doc(json!({
        "name": "Fireball", "type": "Spell", "category": "spell", "level": 3,
        "trait": ["Fire", "Concentrate"], "rarity": "uncommon",
        "summary": "Boom.", "url": "/Spells.aspx?ID=1"
    }));
    let s = format_summary(1, &d, "https://2e.aonprd.com");
    assert!(s.contains("1. Fireball - Spell 3"), "{s}");
    assert!(s.contains("[Fire, Concentrate]"));
    assert!(s.contains("(uncommon)"));
    assert!(s.contains("Boom."));
    assert!(s.contains("https://2e.aonprd.com/Spells.aspx?ID=1"));
}

#[test]
fn summary_does_not_repeat_a_rarity_already_in_traits() {
    let d = doc(json!({
        "name": "X", "category": "spell", "rarity": "uncommon",
        "trait": ["Fire", "Uncommon"]
    }));
    let s = format_summary(1, &d, "b");
    assert!(!s.contains("(uncommon)"), "{s}");
}

#[test]
fn summary_marks_legacy_and_formerly_named_entries() {
    let legacy = doc(json!({"name": "Heal", "category": "spell", "remaster_id": ["spell-1554"]}));
    assert!(format_summary(1, &legacy, "b").contains("Heal - spell (legacy)"));
    let renamed = doc(json!({
        "name": "Force Barrage", "category": "spell", "legacy_name": ["Magic Missile"]
    }));
    assert!(format_summary(1, &renamed, "b").contains("(formerly Magic Missile)"));
}

#[test]
fn summary_hides_common_rarity_and_keeps_absolute_url() {
    let d = doc(json!({
        "name": "X", "category": "feat", "rarity": "common",
        "url": "https://example.test/y"
    }));
    let s = format_summary(2, &d, "https://2e.aonprd.com");
    assert!(!s.contains("(common)"));
    assert!(s.contains("https://example.test/y"));
}

#[test]
fn summary_unknown_name_and_bare_level() {
    let d = doc(json!({"category": "", "level": 2}));
    let s = format_summary(3, &d, "b");
    assert!(s.contains("3. Unknown (level 2)"), "{s}");
}

#[test]
fn detail_renders_markdown_with_structure_and_without_links() {
    // Regression: `get` used the flat `text` field, so a creature stat block
    // arrived as one run-on line.
    let d = doc(json!({
        "name": "Goblin Warrior", "category": "creature", "pfs": "Standard",
        "url": "/Monsters.aspx?ID=1", "legacy_name": ["Goblin Warrior (Legacy)"],
        "markdown": "**Perception** +2; [darkvision](/MonsterAbilities.aspx?ID=59)\n\n**AC** 16\n\n**HP** 6",
        "text": "Perception +2; darkvision AC 16 HP 6"
    }));
    let s = format_detail(&d, "https://2e.aonprd.com");
    assert!(
        s.starts_with("URL: https://2e.aonprd.com/Monsters.aspx?ID=1\n"),
        "{s}"
    );
    assert!(s.contains("Formerly: Goblin Warrior (Legacy)\n"), "{s}");
    assert!(s.contains("PFS: Standard"));
    assert!(s.contains("**Perception** +2; darkvision\n"), "{s}");
    assert!(s.contains("**AC** 16\n"), "{s}");
    assert!(
        !s.contains("MonsterAbilities"),
        "links should be stripped: {s}"
    );
}

#[test]
fn detail_falls_back_to_text_without_markdown() {
    let d = doc(json!({
        "name": "Z", "category": "feat", "source": ["Core"], "summary": "Short.",
        "source_raw": ["Core pg. 3"], "actions": "Reaction", "trait": ["General"]
    }));
    let s = format_detail(&d, "b");
    assert!(s.contains("# Z (feat)"), "{s}");
    assert!(s.contains("Actions: Reaction"));
    assert!(s.contains("Traits: General"));
    assert!(s.contains("Source: Core pg. 3"));
    assert!(s.contains("Short."));
    assert!(!s.contains("URL:"));
}

#[test]
fn alternatives_list_name_category_and_url() {
    let docs = vec![doc(
        json!({"name": "Shield", "category": "spell", "url": "/S.aspx?ID=1"}),
    )];
    assert_eq!(
        format_alternatives(&docs, "https://x"),
        "- Shield (category: spell) https://x/S.aspx?ID=1\n"
    );
}

#[test]
fn detail_flags_legacy_entries() {
    let d = doc(json!({"name": "Heal", "remaster_id": ["spell-1554"], "markdown": "Heal."}));
    assert!(format_detail(&d, "b").contains("Edition: legacy (pre-remaster)"));
}
