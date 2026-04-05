use std::collections::HashMap;

use waybuilder::build::mechanics::boosts::{extract_boost_spec, parse_ability};
use waybuilder::build::mechanics::proficiency::{
    extract_proficiency_advances, extract_proficiency_grants, parse_rank,
};
use waybuilder::build::mechanics::skills::{extract_class_skill_grants, extract_skill_grants};
use waybuilder::build::mechanics::{ancestry, background, class, subclass};
use waybuilder::build::rules::combat;
use waybuilder::build::rules::proficiency::{
    apply_proficiency_advances, class_proficiency_advances,
};
use waybuilder::model::abilities::{Abilities, Ability, BoostSource};
use waybuilder::model::character::Character;
use waybuilder::model::proficiencies::Rank;
use wayfinder_core::aon::Document;

fn make_doc() -> Document {
    Document {
        id: None,
        name: None,
        category: None,
        doc_type: None,
        url: None,
        text: None,
        markdown: None,
        summary: None,
        source: vec![],
        rarity: None,
        traits: vec![],
        trait_group: vec![],
        pfs: None,
        level: None,
        tradition: vec![],
        domain: vec![],
        favored_weapon: vec![],
        sanctification: vec![],
        attribute: vec![],
        deity: vec![],
        remaster_id: vec![],
        legacy_id: vec![],
        extra: HashMap::new(),
    }
}

#[test]
fn parse_ability_full_names() {
    assert_eq!(parse_ability("Strength"), Some(Ability::Strength));
    assert_eq!(parse_ability("dexterity"), Some(Ability::Dexterity));
    assert_eq!(parse_ability("CON"), Some(Ability::Constitution));
    assert_eq!(parse_ability("garbage"), None);
}

#[test]
fn parse_rank_values() {
    assert_eq!(parse_rank("trained"), Some(Rank::Trained));
    assert_eq!(parse_rank("Expert"), Some(Rank::Expert));
    assert_eq!(parse_rank("MASTER"), Some(Rank::Master));
    assert_eq!(parse_rank("legendary"), Some(Rank::Legendary));
    assert_eq!(parse_rank("untrained"), None);
}

#[test]
fn extract_boost_spec_mixed() {
    let mut doc = make_doc();
    doc.attribute = vec!["Dexterity".into(), "Free".into()];
    doc.extra
        .insert("attribute_flaw".into(), serde_json::json!(["Constitution"]));
    let spec = extract_boost_spec(&doc);
    assert_eq!(spec.fixed, vec![Ability::Dexterity]);
    assert_eq!(spec.free, 1);
    assert_eq!(spec.flaws, vec![Ability::Constitution]);
}

#[test]
fn extract_ancestry_data_dwarf() {
    let mut doc = make_doc();
    doc.attribute = vec!["Constitution".into(), "Wisdom".into(), "Free".into()];
    doc.extra
        .insert("size".into(), serde_json::json!(["Medium"]));
    doc.extra
        .insert("speed".into(), serde_json::json!({"land": 20}));
    doc.extra.insert("hp".into(), serde_json::json!(10));
    doc.extra
        .insert("attribute_flaw".into(), serde_json::json!(["Charisma"]));

    let data = ancestry::extract_ancestry_data(&doc);
    assert_eq!(data.size.as_deref(), Some("Medium"));
    assert_eq!(data.speed, 20);
    assert_eq!(data.hp, 10);
    assert_eq!(data.boost_spec.fixed.len(), 2);
    assert_eq!(data.boost_spec.free, 1);
    assert_eq!(data.boost_spec.flaws, vec![Ability::Charisma]);
}

#[test]
fn extract_background_data_with_lore() {
    let mut doc = make_doc();
    doc.attribute = vec!["Strength".into(), "Constitution".into()];
    doc.extra.insert(
        "skill".into(),
        serde_json::json!(["Athletics", "Warfare Lore"]),
    );
    doc.extra
        .insert("feat".into(), serde_json::json!(["Titan Wrestler"]));

    let data = background::extract_background_data(&doc);
    assert_eq!(data.granted_skills, vec!["Athletics"]);
    assert_eq!(data.granted_lores, vec!["Warfare"]);
    assert_eq!(data.granted_feat.as_deref(), Some("Titan Wrestler"));
    // 2 fixed + 1 free (background rule)
    assert_eq!(data.boost_spec.free, 1);
}

#[test]
fn extract_class_data_fighter() {
    let mut doc = make_doc();
    doc.attribute = vec!["Strength".into(), "Dexterity".into()];
    doc.extra.insert("hp".into(), serde_json::json!(10));
    doc.extra
        .insert("perception_proficiency".into(), serde_json::json!("expert"));
    doc.extra
        .insert("fortitude_proficiency".into(), serde_json::json!("expert"));
    doc.extra
        .insert("reflex_proficiency".into(), serde_json::json!("trained"));
    doc.extra
        .insert("will_proficiency".into(), serde_json::json!("trained"));
    doc.extra.insert(
        "attack_proficiency".into(),
        serde_json::json!(["Simple Weapons", "Martial Weapons", "Unarmed Attacks"]),
    );
    doc.extra.insert(
        "defense_proficiency".into(),
        serde_json::json!([
            "Unarmored Defense",
            "Light Armor",
            "Medium Armor",
            "Heavy Armor"
        ]),
    );
    doc.extra.insert(
        "skill_proficiency".into(),
        serde_json::json!(["Athletics", "3 additional skills"]),
    );

    let data = class::extract_class_data(&doc);
    assert_eq!(data.hp, 10);
    // Multiple attributes => 1 free boost
    assert_eq!(data.boost_spec.free, 1);
    assert!(data.boost_spec.fixed.is_empty());
    assert_eq!(data.perception, Rank::Expert);
    assert_eq!(data.fortitude, Rank::Expert);
    assert_eq!(data.reflex, Rank::Trained);
    assert_eq!(data.will, Rank::Trained);
    assert_eq!(data.unarmed, Rank::Trained);
    assert_eq!(data.simple_weapons, Rank::Trained);
    assert_eq!(data.martial_weapons, Rank::Trained);
    assert_eq!(data.unarmored, Rank::Trained);
    assert_eq!(data.heavy_armor, Rank::Trained);
    assert_eq!(data.granted_skills, vec!["Athletics"]);
}

#[test]
fn extract_skill_grants_separates_lores() {
    let mut doc = make_doc();
    doc.extra.insert(
        "skill".into(),
        serde_json::json!(["Arcana", "Dragon Lore", "Stealth"]),
    );
    let grants = extract_skill_grants(&doc);
    assert_eq!(grants.skills, vec!["Arcana", "Stealth"]);
    assert_eq!(grants.lores, vec!["Dragon"]);
}

#[test]
fn extract_class_skill_grants_filters_meta() {
    let mut doc = make_doc();
    doc.extra.insert(
        "skill_proficiency".into(),
        serde_json::json!(["Religion", "2 additional skills equal to 2 + INT"]),
    );
    let skills = extract_class_skill_grants(&doc);
    assert_eq!(skills, vec!["Religion"]);
}

#[test]
fn proficiency_grants_from_text() {
    let text = "You are trained in Perception and expert in Fortitude saves.";
    let grants = extract_proficiency_grants(text);
    assert_eq!(grants.len(), 2);
    assert_eq!(grants[0].target, "Perception and expert in Fortitude saves");
    // The text parser is simple; it takes until punctuation
}

#[test]
fn compute_ac_level_1_trained() {
    let abilities = Abilities {
        boosts: vec![BoostSource {
            source: "test".into(),
            ability: Ability::Dexterity,
        }],
        flaws: vec![],
    };
    // DEX 12 => +1 mod, Trained = +2, level 1, no armor (ac_bonus=0, no dex_cap, potency=0)
    let ac = combat::compute_ac(&abilities, Rank::Trained, 1, 0, None, 0);
    assert_eq!(ac, 10 + 1 + 2 + 1); // 14
}

#[test]
fn compute_save_with_expert() {
    let abilities = Abilities {
        boosts: vec![
            BoostSource {
                source: "a".into(),
                ability: Ability::Constitution,
            },
            BoostSource {
                source: "b".into(),
                ability: Ability::Constitution,
            },
        ],
        flaws: vec![],
    };
    // CON 14 => +2 mod, Expert = +4, level 5
    let fort = combat::compute_save(&abilities, Ability::Constitution, Rank::Expert, 5);
    assert_eq!(fort, 2 + 4 + 5); // 11
}

#[test]
fn compute_class_dc() {
    let abilities = Abilities {
        boosts: vec![
            BoostSource {
                source: "a".into(),
                ability: Ability::Charisma,
            },
            BoostSource {
                source: "b".into(),
                ability: Ability::Charisma,
            },
            BoostSource {
                source: "c".into(),
                ability: Ability::Charisma,
            },
        ],
        flaws: vec![],
    };
    // CHA 16 => +3, Trained = +2, level 3
    let dc = combat::compute_class_dc(&abilities, Ability::Charisma, Rank::Trained, 3);
    assert_eq!(dc, 10 + 3 + 2 + 3); // 18
}

// Phase 2 tests

#[test]
fn fighter_has_proficiency_advances() {
    let advances = class_proficiency_advances("Fighter");
    assert!(!advances.is_empty());
    // Fighter gets expert saves at 3
    let expert_saves_3: Vec<_> = advances
        .iter()
        .filter(|a| a.level == 3 && a.rank == Rank::Expert)
        .collect();
    assert!(!expert_saves_3.is_empty());
}

#[test]
fn unknown_class_returns_empty_advances() {
    let advances = class_proficiency_advances("Necromancer");
    assert!(advances.is_empty());
}

#[test]
fn apply_advances_upgrades_proficiency() {
    let mut ch = Character::new("Test");
    ch.level = 7;
    ch.proficiencies.perception = Rank::Trained;
    ch.proficiencies.fortitude = Rank::Trained;

    let advances = class_proficiency_advances("Fighter");
    apply_proficiency_advances(&mut ch, &advances);

    // Fighter at 7: perception should be Master, fort should be Expert (via all saves at 3)
    assert_eq!(ch.proficiencies.perception, Rank::Master);
    assert!(ch.proficiencies.fortitude >= Rank::Expert);
}

#[test]
fn advances_respect_level_cap() {
    let mut ch = Character::new("Test");
    ch.level = 1;
    ch.proficiencies.perception = Rank::Trained;

    let advances = class_proficiency_advances("Fighter");
    apply_proficiency_advances(&mut ch, &advances);

    // At level 1, no advances should apply
    assert_eq!(ch.proficiencies.perception, Rank::Trained);
}

#[test]
fn class_extraction_populates_advances() {
    let mut doc = make_doc();
    doc.name = Some("Fighter".into());
    doc.attribute = vec!["Strength".into()];
    doc.extra.insert("hp".into(), serde_json::json!(10));

    let data = class::extract_class_data(&doc);
    assert!(!data.proficiency_advances.is_empty());
}

#[test]
fn extract_advances_from_text_become_pattern() {
    let text = "You become an expert in Fortitude saves.";
    let advances = extract_proficiency_advances(text, 3);
    assert_eq!(advances.len(), 1);
    assert_eq!(advances[0].level, 3);
    assert_eq!(advances[0].rank, Rank::Expert);
    assert_eq!(advances[0].target, "Fortitude saves");
}

#[test]
fn extract_advances_from_text_increases_pattern() {
    let text = "Your Reflex proficiency increases to master.";
    let advances = extract_proficiency_advances(text, 9);
    assert_eq!(advances.len(), 1);
    assert_eq!(advances[0].level, 9);
    assert_eq!(advances[0].rank, Rank::Master);
}

// Fixture-based tests

fn load_fixture(name: &str) -> Document {
    let path = format!("{}/tests/fixtures/{name}", env!("CARGO_MANIFEST_DIR"));
    let data = std::fs::read_to_string(&path).expect(&format!("fixture {name} not found"));
    serde_json::from_str(&data).expect(&format!("fixture {name} invalid JSON"))
}

#[test]
fn fixture_fighter_class_data() {
    let doc = load_fixture("fighter.json");
    let data = class::extract_class_data(&doc);
    assert_eq!(data.hp, 10);
    assert_eq!(data.perception, Rank::Expert);
    assert_eq!(data.fortitude, Rank::Expert);
    assert_eq!(data.simple_weapons, Rank::Trained);
    assert_eq!(data.martial_weapons, Rank::Trained);
    assert!(!data.proficiency_advances.is_empty());
}

#[test]
fn fixture_wizard_class_data() {
    let doc = load_fixture("wizard.json");
    let data = class::extract_class_data(&doc);
    assert_eq!(data.hp, 6);
    assert_eq!(data.will, Rank::Expert);
    assert!(!data.proficiency_advances.is_empty());
}

#[test]
fn fixture_dwarf_ancestry() {
    let doc = load_fixture("dwarf.json");
    let data = ancestry::extract_ancestry_data(&doc);
    assert_eq!(data.hp, 10);
    assert_eq!(data.speed, 20);
    assert_eq!(data.size.as_deref(), Some("Medium"));
    // Dwarf has CON, WIS, Free + CHA flaw
    assert_eq!(data.boost_spec.fixed.len(), 2);
    assert_eq!(data.boost_spec.free, 1);
    assert_eq!(data.boost_spec.flaws.len(), 1);
}

#[test]
fn fixture_farmhand_background() {
    let doc = load_fixture("farmhand.json");
    let data = background::extract_background_data(&doc);
    assert!(data.granted_feat.is_some());
    assert!(!data.granted_skills.is_empty() || !data.granted_lores.is_empty());
}

#[test]
fn fixture_sorcerer_class_data() {
    let doc = load_fixture("sorcerer.json");
    let data = class::extract_class_data(&doc);
    assert_eq!(data.hp, 6);
    // Sorcerer has Expert Will at level 1
    assert_eq!(data.will, Rank::Expert);
    assert!(!data.proficiency_advances.is_empty());
}

#[test]
fn fixture_fighter_level_20_all_advances() {
    let doc = load_fixture("fighter.json");
    let data = class::extract_class_data(&doc);

    let mut ch = Character::new("Test Fighter");
    ch.level = 20;
    ch.proficiencies.perception = data.perception;
    ch.proficiencies.fortitude = data.fortitude;
    ch.proficiencies.reflex = data.reflex;
    ch.proficiencies.will = data.will;
    ch.proficiencies.simple_weapons = data.simple_weapons;
    ch.proficiencies.martial_weapons = data.martial_weapons;
    ch.proficiencies.unarmed = data.unarmed;
    ch.proficiencies.unarmored = data.unarmored;
    ch.proficiencies.light_armor = data.light_armor;
    ch.proficiencies.medium_armor = data.medium_armor;
    ch.proficiencies.heavy_armor = data.heavy_armor;

    apply_proficiency_advances(&mut ch, &data.proficiency_advances);

    // Fighter 20 should have Legendary weapons, Master armor, Master perception
    assert_eq!(ch.proficiencies.perception, Rank::Master);
    assert_eq!(ch.proficiencies.simple_weapons, Rank::Legendary);
    assert_eq!(ch.proficiencies.martial_weapons, Rank::Legendary);
    assert_eq!(ch.proficiencies.unarmed, Rank::Legendary);
    assert_eq!(ch.proficiencies.fortitude, Rank::Master);
    assert_eq!(ch.proficiencies.reflex, Rank::Master);
    assert_eq!(ch.proficiencies.will, Rank::Master);
    assert_eq!(ch.proficiencies.unarmored, Rank::Master);
}

// Phase 3 tests

#[test]
fn subclass_category_for_sorcerer() {
    assert_eq!(subclass::subclass_category("Sorcerer"), Some("bloodline"));
    assert_eq!(subclass::subclass_category("Fighter"), None);
    assert_eq!(subclass::subclass_category("Champion"), Some("cause"));
}

#[test]
fn subclass_label_for_classes() {
    assert_eq!(subclass::subclass_label("Sorcerer"), "Bloodline");
    assert_eq!(subclass::subclass_label("Wizard"), "Arcane School");
    assert_eq!(subclass::subclass_label("Unknown"), "Subclass");
}

#[test]
fn requires_deity_for_champion_cleric() {
    assert!(subclass::requires_deity("Champion"));
    assert!(subclass::requires_deity("Cleric"));
    assert!(!subclass::requires_deity("Fighter"));
}

#[test]
fn fixture_sorcerer_has_subclass_category() {
    let doc = load_fixture("sorcerer.json");
    let data = class::extract_class_data(&doc);
    assert_eq!(data.subclass_category.as_deref(), Some("bloodline"));
}

#[test]
fn fixture_fighter_no_subclass_category() {
    let doc = load_fixture("fighter.json");
    let data = class::extract_class_data(&doc);
    assert!(data.subclass_category.is_none());
}

#[test]
fn fixture_sorcerer_class_features_extracted() {
    let doc = load_fixture("sorcerer.json");
    let data = class::extract_class_data(&doc);
    assert!(!data.class_features.is_empty());
    // Should have "bloodline" at level 1
    let l1: Vec<_> = data
        .class_features
        .iter()
        .filter(|f| f.level == 1)
        .collect();
    assert!(!l1.is_empty());
    let names: Vec<&str> = l1.iter().map(|f| f.name.as_str()).collect();
    assert!(
        names.iter().any(|n| n.to_lowercase().contains("bloodline")),
        "Expected bloodline in L1 features: {names:?}"
    );
}

#[test]
fn fixture_fighter_class_features_extracted() {
    let doc = load_fixture("fighter.json");
    let data = class::extract_class_data(&doc);
    assert!(!data.class_features.is_empty());
    // Should have features at multiple levels
    let levels: Vec<u8> = data.class_features.iter().map(|f| f.level).collect();
    assert!(levels.contains(&1));
    assert!(levels.iter().any(|&l| l > 1));
}
