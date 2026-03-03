use crate::build::choices::BuildChoices;
use crate::build::mechanics::boosts::parse_ability;
use crate::build::rules::abilities::{add_boost, add_flaw};
use crate::build::rules::combat;
use crate::build::rules::proficiency::apply_proficiency_advances;
use crate::model::abilities::Ability;
use crate::model::character::{Character, WeaponAttack};
use crate::model::feat::{FeatChoice, FeatType};
use crate::model::proficiencies::Rank;
use crate::model::spell::{SpellCaster, SpellEntry};

/// Derive a full `Character` from declarative `BuildChoices`.
pub fn recalculate(choices: &BuildChoices) -> Character {
    let mut ch = Character::new(&choices.name);
    ch.level = choices.level;

    // Identity fields
    ch.ancestry = choices.ancestry.clone();
    ch.heritage = choices.heritage.clone();
    ch.background = choices.background.clone();
    ch.class = choices.class.clone();
    ch.deity = choices.deity.clone();

    // Ancestry data
    if let Some(data) = &choices.ancestry_data {
        ch.size = data.size.clone();
        ch.speed = data.speed;
        ch.ancestry_hp = data.hp;
    }

    // Background data
    if let Some(data) = &choices.background_data {
        for skill in &data.granted_skills {
            ch.proficiencies.train_skill(skill, Rank::Trained);
        }
        for lore in &data.granted_lores {
            ch.proficiencies.add_lore(lore);
        }
        if let Some(feat_name) = &data.granted_feat {
            ch.feats.push(FeatChoice {
                name: feat_name.clone(),
                feat_type: FeatType::Bonus,
                level: 1,
                slot: "background_feat".to_string(),
                source_id: None,
            });
        }
    }

    // Class data
    if let Some(data) = &choices.class_data {
        ch.class_hp = data.hp;
        ch.key_ability = data.key_ability.clone();
        apply_class_proficiencies(&mut ch, data);
        apply_proficiency_advances(&mut ch, &data.proficiency_advances);
        for skill in &data.granted_skills {
            ch.proficiencies.train_skill(skill, Rank::Trained);
        }
    }

    // Initial skill selections at L1
    for skill in &choices.initial_skills {
        ch.proficiencies.train_skill(skill, Rank::Trained);
    }

    // Ability boosts/flaws from all sources
    for (source, choice_set) in &choices.ability_choices {
        for &ability in &choice_set.fixed {
            add_boost(&mut ch.abilities, source, ability);
        }
        for &ability in &choice_set.chosen {
            add_boost(&mut ch.abilities, source, ability);
        }
        let flaw_source = if choice_set.flaws.is_empty() {
            String::new()
        } else {
            format!("{source}_flaw")
        };
        for &ability in &choice_set.flaws {
            add_flaw(&mut ch.abilities, &flaw_source, ability);
        }
    }

    // Skill increases (apply in level order)
    for skill in choices.skill_increases.values() {
        ch.proficiencies.increase_skill(skill);
    }

    // Feats
    for (slot_key, sel) in &choices.feats {
        ch.feats.push(FeatChoice {
            name: sel.name.clone(),
            feat_type: sel.feat_type,
            level: sel.level,
            slot: slot_key.clone(),
            source_id: sel.source_id.clone(),
        });
    }

    // Languages
    recalculate_languages(&mut ch, choices);

    // Deity mechanics
    recalculate_deity(&mut ch, choices);

    // Spellcasting
    recalculate_spells(&mut ch, choices);

    // HP
    recalculate_hp(&mut ch);

    // Derived combat stats
    recalculate_combat(&mut ch, choices);

    ch
}

fn recalculate_languages(ch: &mut Character, choices: &BuildChoices) {
    if let Some(data) = &choices.ancestry_data {
        ch.languages.extend(data.granted_languages.clone());
    }
    let int_mod = ch.abilities.modifier(Ability::Intelligence).max(0) as usize;
    for lang in choices.bonus_languages.iter().take(int_mod) {
        ch.languages.push(lang.clone());
    }
}

fn recalculate_deity(ch: &mut Character, choices: &BuildChoices) {
    let Some(deity_data) = &choices.deity_data else {
        return;
    };
    ch.favored_weapon = deity_data.favored_weapon.clone();

    // If cleric and deity has a divine skill, train it
    let is_cleric = choices
        .class
        .as_deref()
        .is_some_and(|c| c.eq_ignore_ascii_case("cleric"));
    if is_cleric
        && let Some(skill) = &deity_data.divine_skill
    {
        ch.proficiencies.train_skill(skill, Rank::Trained);
    }
}

fn recalculate_spells(ch: &mut Character, choices: &BuildChoices) {
    let Some(data) = &choices.class_data else {
        return;
    };
    let Some(tradition) = &data.tradition else {
        return;
    };

    let mut spells = Vec::new();
    for (rank, slot_choices) in &choices.prepared_spells {
        for sc in slot_choices {
            spells.push(SpellEntry {
                name: sc.name.clone(),
                rank: *rank,
                source_id: sc.source_id.clone(),
            });
        }
    }

    let focus_spells: Vec<SpellEntry> = choices
        .focus_spells
        .iter()
        .map(|sc| SpellEntry {
            name: sc.name.clone(),
            rank: sc.rank,
            source_id: sc.source_id.clone(),
        })
        .collect();
    ch.focus_points = focus_spells.len().min(3) as u8;

    ch.spell_caster = Some(SpellCaster {
        tradition: Some(tradition.clone()),
        ability: data.casting_ability.clone(),
        spells,
        focus_spells,
    });

    // Spell attack and DC: casting ability + trained + level
    let casting_ability = data
        .casting_ability
        .as_deref()
        .and_then(parse_ability)
        .unwrap_or(Ability::Intelligence);
    let spell_rank = Rank::Trained;
    let level = ch.level;
    ch.spell_attack =
        ch.abilities.modifier(casting_ability) + spell_rank.bonus() + level as i32;
    ch.spell_dc = 10 + ch.spell_attack;
}

fn recalculate_hp(ch: &mut Character) {
    let con_mod = ch.abilities.modifier(Ability::Constitution);
    let per_level = (ch.class_hp as i32 + con_mod).max(1);
    ch.hp_max = ch.ancestry_hp + (per_level * ch.level as i32) as u32;
}

fn recalculate_combat(ch: &mut Character, choices: &BuildChoices) {
    let level = ch.level;
    let eq = &choices.equipment;

    // --- AC from equipped armor ---
    let worn_armor = eq.armor.iter().find(|a| a.worn);
    let (ac_bonus, dex_cap, armor_rank, armor_potency, resilient) = match worn_armor {
        Some(a) => {
            let rank = armor_prof_rank(&a.armor_category, &ch.proficiencies);
            (a.ac_bonus, a.dex_cap, rank, a.potency, a.resilient)
        }
        None => (0, None, ch.proficiencies.unarmored, 0, Default::default()),
    };
    ch.ac = combat::compute_ac(&ch.abilities, armor_rank, level, ac_bonus, dex_cap, armor_potency);

    // Shield AC bonus (when raised)
    ch.shield_ac_bonus = eq
        .shield
        .as_ref()
        .filter(|s| s.raised)
        .map(|s| s.ac_bonus)
        .unwrap_or(0);

    ch.perception_bonus =
        combat::compute_perception(&ch.abilities, ch.proficiencies.perception, level);

    // Saves with resilient bonus
    let res_bonus = resilient.bonus();
    ch.fortitude_bonus = combat::compute_save(
        &ch.abilities,
        Ability::Constitution,
        ch.proficiencies.fortitude,
        level,
    ) + res_bonus;
    ch.reflex_bonus = combat::compute_save(
        &ch.abilities,
        Ability::Dexterity,
        ch.proficiencies.reflex,
        level,
    ) + res_bonus;
    ch.will_bonus = combat::compute_save(
        &ch.abilities,
        Ability::Wisdom,
        ch.proficiencies.will,
        level,
    ) + res_bonus;

    // Class DC
    let key = ch
        .key_ability
        .as_deref()
        .and_then(parse_ability)
        .unwrap_or(Ability::Strength);
    let dc_rank = if ch.class.is_some() {
        Rank::Trained
    } else {
        Rank::Untrained
    };
    ch.class_dc = combat::compute_class_dc(&ch.abilities, key, dc_rank, level);

    // Best weapon proficiency for generic melee/ranged (kept for backward compat)
    let best_melee = [
        ch.proficiencies.unarmed,
        ch.proficiencies.simple_weapons,
        ch.proficiencies.martial_weapons,
    ]
    .into_iter()
    .max()
    .unwrap_or(Rank::Untrained);
    ch.melee_attack = combat::compute_melee_attack(&ch.abilities, best_melee, level);
    ch.ranged_attack = combat::compute_ranged_attack(&ch.abilities, best_melee, level);

    // Per-weapon attacks
    ch.weapon_attacks = eq
        .weapons
        .iter()
        .map(|w| compute_weapon_attack(w, ch))
        .collect();
}

fn armor_prof_rank(
    category: &str,
    profs: &crate::model::proficiencies::Proficiencies,
) -> Rank {
    match category.to_lowercase().as_str() {
        "light" => profs.light_armor,
        "medium" => profs.medium_armor,
        "heavy" => profs.heavy_armor,
        _ => profs.unarmored,
    }
}

fn weapon_prof_rank(
    category: &str,
    profs: &crate::model::proficiencies::Proficiencies,
) -> Rank {
    match category.to_lowercase().as_str() {
        "simple" => profs.simple_weapons,
        "martial" => profs.martial_weapons,
        "unarmed" => profs.unarmed,
        _ => Rank::Untrained, // advanced, unknown
    }
}

fn compute_weapon_attack(
    weapon: &crate::model::equipment::Weapon,
    ch: &Character,
) -> WeaponAttack {
    let is_ranged = weapon.is_ranged();
    let is_finesse = weapon.has_trait("Finesse");

    let ability_mod = if is_ranged {
        ch.abilities.modifier(Ability::Dexterity)
    } else if is_finesse {
        ch.abilities
            .modifier(Ability::Strength)
            .max(ch.abilities.modifier(Ability::Dexterity))
    } else {
        ch.abilities.modifier(Ability::Strength)
    };

    let prof_rank = weapon_prof_rank(&weapon.weapon_category, &ch.proficiencies);
    let attack_bonus =
        ability_mod + prof_rank.bonus() + ch.level as i32 + weapon.potency as i32;

    let num_dice = 1 + weapon.striking.extra_dice();
    let die = weapon.damage_die_str();
    let dmg_mod = if is_ranged { 0 } else { ability_mod };
    let sign = if dmg_mod >= 0 { "+" } else { "" };
    let damage = if dmg_mod == 0 {
        format!("{num_dice}{die} {}", weapon.damage_type)
    } else {
        format!("{num_dice}{die}{sign}{dmg_mod} {}", weapon.damage_type)
    };

    WeaponAttack {
        display: weapon.display_name(),
        attack_bonus,
        damage,
    }
}

fn apply_class_proficiencies(
    ch: &mut Character,
    data: &crate::build::choices::ClassData,
) {
    ch.proficiencies.perception = data.perception;
    ch.proficiencies.fortitude = data.fortitude;
    ch.proficiencies.reflex = data.reflex;
    ch.proficiencies.will = data.will;
    ch.proficiencies.unarmored = data.unarmored;
    ch.proficiencies.light_armor = data.light_armor;
    ch.proficiencies.medium_armor = data.medium_armor;
    ch.proficiencies.heavy_armor = data.heavy_armor;
    ch.proficiencies.simple_weapons = data.simple_weapons;
    ch.proficiencies.martial_weapons = data.martial_weapons;
    ch.proficiencies.unarmed = data.unarmed;
}
