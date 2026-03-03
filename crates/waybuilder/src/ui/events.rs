use crossterm::event::{self, Event, KeyCode, KeyModifiers};

use super::app::{App, DetailTab, Focus, Modal, Screen, progression_for, sync_progression};
use crate::build::choices::{AbilityChoiceSet, FeatSelection, SpellSlotChoice};
use crate::model::types::DivineFont;
use crate::build::mechanics::equipment as equip_extract;
use crate::model::equipment::Item;
use crate::build::mechanics::{ancestry, background, class, deity, subclass};
use crate::build::progression::ProgressionEntry;
use crate::build::recalculate::recalculate;
use crate::build::slot::BuildSlot;
use crate::data::loader::LoadRequest;
use crate::model::abilities::BoostSpec;
use crate::model::proficiencies::Rank;

pub fn poll_event(app: &mut App) -> anyhow::Result<()> {
    if !event::poll(std::time::Duration::from_millis(50))? {
        return Ok(());
    }
    let Event::Key(key) = event::read()? else {
        return Ok(());
    };
    if key.kind != crossterm::event::KeyEventKind::Press {
        return Ok(());
    }

    // Character select screen
    if app.screen == Screen::CharacterSelect {
        return handle_select_key(app, key.code);
    }

    // Help overlay
    if app.show_help {
        app.show_help = false;
        return Ok(());
    }

    // Modal takes priority
    match &app.modal {
        Some(Modal::Selection(_)) => return handle_selection_key(app, key.code, key.modifiers),
        Some(Modal::Boosts(_)) => return handle_boost_key(app, key.code),
        Some(Modal::Skill(_)) => return handle_skill_key(app, key.code),
        Some(Modal::TextInput(_)) => return handle_text_input_key(app, key.code, key.modifiers),
        Some(Modal::Info(_)) => return handle_info_key(app, key.code),
        Some(Modal::Runes(_)) => return handle_rune_key(app, key.code),
        Some(Modal::Domain(_)) => return handle_domain_key(app, key.code),
        None => {}
    }

    match key.code {
        KeyCode::Char('q') => app.quit = true,
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.quit = true;
        }
        KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            save_character(app);
        }
        KeyCode::Char('e') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            export_character(app);
        }
        KeyCode::Tab => app.focus = app.focus.next(),
        KeyCode::Up => {
            if app.focus == Focus::DetailTabs && app.detail_tab == DetailTab::Spells {
                handle_spell_up(app);
            } else if app.focus == Focus::DetailTabs && app.detail_tab == DetailTab::Equipment {
                handle_equipment_up(app);
            } else {
                handle_up(app);
            }
        }
        KeyCode::Down => {
            if app.focus == Focus::DetailTabs && app.detail_tab == DetailTab::Spells {
                handle_spell_down(app);
            } else if app.focus == Focus::DetailTabs && app.detail_tab == DetailTab::Equipment {
                handle_equipment_down(app);
            } else {
                handle_down(app);
            }
        }
        KeyCode::Left if app.focus == Focus::DetailTabs => {
            app.detail_tab = app.detail_tab.prev();
        }
        KeyCode::Right if app.focus == Focus::DetailTabs => {
            app.detail_tab = app.detail_tab.next();
        }
        KeyCode::Enter if app.focus == Focus::DetailTabs && app.detail_tab == DetailTab::Spells => {
            open_spell_picker(app);
        }
        KeyCode::Backspace | KeyCode::Delete
            if app.focus == Focus::DetailTabs && app.detail_tab == DetailTab::Spells =>
        {
            clear_spell_slot(app);
        }
        KeyCode::Enter
            if app.focus == Focus::DetailTabs && app.detail_tab == DetailTab::Equipment =>
        {
            handle_equipment_enter(app);
        }
        KeyCode::Backspace | KeyCode::Delete
            if app.focus == Focus::DetailTabs && app.detail_tab == DetailTab::Equipment =>
        {
            handle_equipment_delete(app);
        }
        KeyCode::Char('r')
            if app.focus == Focus::DetailTabs && app.detail_tab == DetailTab::Equipment =>
        {
            open_equipment_rune_modal(app);
        }
        KeyCode::Enter if app.focus == Focus::BuildSelector => {
            open_modal_for_cursor(app);
        }
        KeyCode::Char('+') | KeyCode::Char('=') => level_change(app, 1),
        KeyCode::Char('-') => level_change(app, -1),
        KeyCode::Backspace | KeyCode::Delete if app.focus == Focus::BuildSelector => {
            clear_slot(app);
        }
        KeyCode::Char('?') => app.show_help = true,
        KeyCode::Esc => {
            app.refresh_saved_list();
            app.screen = Screen::CharacterSelect;
        }
        KeyCode::Char('n') => {
            let modal = crate::ui::modal::text_input::TextInputModal::new(
                "Rename Character",
                "Name:",
                &app.character.name,
            );
            app.modal = Some(Modal::TextInput(modal));
        }
        _ => {}
    }
    Ok(())
}

fn handle_selection_key(
    app: &mut App,
    code: KeyCode,
    modifiers: KeyModifiers,
) -> anyhow::Result<()> {
    let Some(Modal::Selection(modal)) = &mut app.modal else {
        return Ok(());
    };
    match code {
        KeyCode::Esc => {
            app.spell_pick_rank = None;
            app.modal = None;
        }
        KeyCode::Up => modal.move_up(),
        KeyCode::Down => modal.move_down(),
        KeyCode::PageDown => modal.scroll_detail_down(),
        KeyCode::PageUp => modal.scroll_detail_up(),
        KeyCode::Enter => {
            if let Some(doc) = modal.selected().cloned() {
                let title = modal.title.clone();
                if let Some(rank) = app.spell_pick_rank.take() {
                    apply_spell_selection(app, &doc, rank);
                } else if title.starts_with("Add ") {
                    app.modal = None;
                    apply_equipment_selection(app, &doc);
                    return Ok(());
                } else {
                    apply_selection(app, &doc);
                }
            } else {
                app.spell_pick_rank = None;
            }
            app.modal = None;
        }
        KeyCode::Backspace => modal.backspace(),
        KeyCode::Char(c) => {
            if modifiers.contains(KeyModifiers::CONTROL) {
                return Ok(());
            }
            modal.type_char(c);
        }
        _ => {}
    }
    Ok(())
}

fn handle_boost_key(app: &mut App, code: KeyCode) -> anyhow::Result<()> {
    let Some(Modal::Boosts(modal)) = &mut app.modal else {
        return Ok(());
    };
    match code {
        KeyCode::Esc => {
            app.modal = None;
        }
        KeyCode::Up => modal.move_up(),
        KeyCode::Down => modal.move_down(),
        KeyCode::Left => modal.prev_group(),
        KeyCode::Right | KeyCode::Tab => modal.next_group(),
        KeyCode::Char('a') => modal.toggle_alternate(),
        KeyCode::Char(' ') | KeyCode::Enter if !modal.is_complete() => {
            modal.toggle();
        }
        KeyCode::Enter if modal.is_complete() => {
            apply_boosts(app);
            app.modal = None;
        }
        _ => {}
    }
    Ok(())
}

fn handle_text_input_key(
    app: &mut App,
    code: KeyCode,
    modifiers: KeyModifiers,
) -> anyhow::Result<()> {
    let Some(Modal::TextInput(modal)) = &mut app.modal else {
        return Ok(());
    };
    match code {
        KeyCode::Esc => {
            app.modal = None;
        }
        KeyCode::Backspace => modal.backspace(),
        KeyCode::Enter => {
            let value = modal.value.clone();
            let field = modal.field.clone();
            app.modal = None;
            apply_text_input(app, &field, &value);
        }
        KeyCode::Char(c) => {
            if !modifiers.contains(KeyModifiers::CONTROL) {
                modal.type_char(c);
            }
        }
        _ => {}
    }
    Ok(())
}

fn apply_text_input(app: &mut App, field: &str, value: &str) {
    if field == "Name:" && !value.is_empty() {
        app.choices.name = value.to_string();
        app.character.name = value.to_string();
    }
}

fn handle_info_key(app: &mut App, code: KeyCode) -> anyhow::Result<()> {
    let Some(Modal::Info(modal)) = &mut app.modal else {
        return Ok(());
    };
    match code {
        KeyCode::Esc | KeyCode::Enter => {
            app.modal = None;
        }
        KeyCode::Down | KeyCode::PageDown => modal.scroll_down(),
        KeyCode::Up | KeyCode::PageUp => modal.scroll_up(),
        _ => {}
    }
    Ok(())
}

fn handle_skill_key(app: &mut App, code: KeyCode) -> anyhow::Result<()> {
    let Some(Modal::Skill(modal)) = &mut app.modal else {
        return Ok(());
    };
    match code {
        KeyCode::Esc => {
            app.modal = None;
        }
        KeyCode::Up => modal.move_up(),
        KeyCode::Down => modal.move_down(),
        KeyCode::Enter => {
            if let Some(skill) = modal.selected().map(String::from) {
                let slot_info = app.progression.get(app.build_cursor).and_then(|e| match e {
                    ProgressionEntry::Slot(s) => Some((s.slot, s.level)),
                    _ => None,
                });
                match slot_info {
                    Some((BuildSlot::SkillSelection, _)) => {
                        app.choices.initial_skills.push(skill);
                    }
                    _ => {
                        let level = slot_info.map(|(_, l)| l).unwrap_or(1);
                        app.choices.skill_increases.insert(level, skill);
                    }
                }
                app.character = recalculate(&app.choices);
                sync_progression(&mut app.progression, &app.choices, &app.character);
            }
            app.modal = None;
        }
        _ => {}
    }
    Ok(())
}

fn level_change(app: &mut App, delta: i32) {
    let new_level = (app.choices.level as i32 + delta).clamp(1, 20) as u8;
    if new_level == app.choices.level {
        return;
    }
    app.choices.level = new_level;
    // Trim choices above new level
    app.choices
        .skill_increases
        .retain(|&lvl, _| lvl <= new_level);
    app.choices.feats.retain(|_key, sel| sel.level <= new_level);
    app.choices.ability_choices.retain(|src, _| {
        if let Some(lvl_str) = src.strip_prefix("level_") {
            lvl_str.parse::<u8>().is_ok_and(|l| l <= new_level)
        } else {
            true // ancestry/background/class sources always kept
        }
    });
    // Trim prepared spells to match available slots at new level
    trim_prepared_spells(&mut app.choices);
    app.character = recalculate(&app.choices);
    app.progression = progression_for(&app.choices);
    sync_progression(&mut app.progression, &app.choices, &app.character);
    app.build_cursor = app
        .build_cursor
        .min(app.progression.len().saturating_sub(1));
    app.status = Some(format!("Level {new_level}"));
}

fn clear_slot(app: &mut App) {
    let Some(ProgressionEntry::Slot(state)) = app.progression.get(app.build_cursor) else {
        return;
    };
    if state.filled.is_none() {
        return;
    }
    let slot = state.slot;
    let level = state.level;

    match slot {
        BuildSlot::Ancestry => {
            app.choices.ancestry = None;
            app.choices.ancestry_data = None;
            // Cascade: heritage, ancestry feats, ancestry boosts
            app.choices.heritage = None;
            app.choices
                .feats
                .retain(|_, sel| sel.feat_type != crate::model::feat::FeatType::Ancestry);
            app.choices.ability_choices.remove("ancestry");
        }
        BuildSlot::Heritage => {
            app.choices.heritage = None;
        }
        BuildSlot::Background => {
            app.choices.background = None;
            app.choices.background_data = None;
            app.choices.ability_choices.remove("background");
            app.choices
                .feats
                .retain(|k, _| k != "background_feat");
        }
        BuildSlot::Class => {
            app.choices.class = None;
            app.choices.class_data = None;
            app.choices.subclass = None;
            app.choices.ability_choices.remove("class");
            app.choices.initial_skills.clear();
            app.choices.prepared_spells.clear();
            app.choices.focus_spells.clear();
            app.choices
                .feats
                .retain(|_, sel| sel.feat_type != crate::model::feat::FeatType::Class);
        }
        BuildSlot::Deity => {
            app.choices.deity = None;
            app.choices.deity_data = None;
            app.choices.chosen_domains.clear();
            app.choices.chosen_divine_font = None;
        }
        BuildSlot::Subclass => {
            app.choices.subclass = None;
        }
        BuildSlot::AbilityBoosts => {
            if level == 1 {
                for src in ["ancestry", "background", "class", "level_1"] {
                    app.choices.ability_choices.remove(src);
                }
            } else {
                app.choices
                    .ability_choices
                    .remove(&format!("level_{level}"));
            }
        }
        BuildSlot::AncestryFeat
        | BuildSlot::ClassFeat
        | BuildSlot::SkillFeat
        | BuildSlot::GeneralFeat => {
            let slot_key = format!("{}_{level}", slot.label());
            app.choices.feats.remove(&slot_key);
        }
        BuildSlot::SkillIncrease => {
            app.choices.skill_increases.remove(&level);
        }
        BuildSlot::ClassFeature => {}
        BuildSlot::SkillSelection => {
            // Find which index this skill selection slot is
            let idx = app
                .progression
                .iter()
                .take(app.build_cursor + 1)
                .filter(|e| {
                    matches!(e, ProgressionEntry::Slot(s) if s.slot == BuildSlot::SkillSelection)
                })
                .count()
                .saturating_sub(1);
            if idx < app.choices.initial_skills.len() {
                app.choices.initial_skills.remove(idx);
            }
        }
    }

    let rebuild = matches!(slot, BuildSlot::Class);
    app.character = recalculate(&app.choices);
    if rebuild {
        app.progression = progression_for(&app.choices);
    }
    sync_progression(&mut app.progression, &app.choices, &app.character);
}

fn handle_select_key(app: &mut App, code: KeyCode) -> anyhow::Result<()> {
    let max = app.saved_characters.len(); // 0 = new, 1..=max = saved
    match code {
        KeyCode::Char('q') => app.quit = true,
        KeyCode::Up => {
            app.select_cursor = app.select_cursor.saturating_sub(1);
        }
        KeyCode::Down => {
            if app.select_cursor < max {
                app.select_cursor += 1;
            }
        }
        KeyCode::Enter => {
            if app.select_cursor == 0 {
                app.new_character();
            } else if let Some(path) = app.saved_characters.get(app.select_cursor - 1).cloned() {
                app.load_from_file(&path);
            }
        }
        KeyCode::Char('d') => {
            // Delete saved character
            if app.select_cursor > 0 {
                if let Some(path) = app.saved_characters.get(app.select_cursor - 1) {
                    let _ = std::fs::remove_file(path);
                }
                app.saved_characters =
                    crate::persistence::save::list_characters().unwrap_or_default();
                if app.select_cursor > app.saved_characters.len() {
                    app.select_cursor = app.saved_characters.len();
                }
            }
        }
        _ => {}
    }
    Ok(())
}

fn handle_up(app: &mut App) {
    if app.focus != Focus::BuildSelector {
        return;
    }
    let mut cursor = app.build_cursor;
    while cursor > 0 {
        cursor -= 1;
        if matches!(app.progression.get(cursor), Some(ProgressionEntry::Slot(_))) {
            app.build_cursor = cursor;
            return;
        }
    }
}

fn handle_down(app: &mut App) {
    if app.focus != Focus::BuildSelector {
        return;
    }
    let max = app.progression.len();
    let mut cursor = app.build_cursor + 1;
    while cursor < max {
        if matches!(app.progression.get(cursor), Some(ProgressionEntry::Slot(_))) {
            app.build_cursor = cursor;
            return;
        }
        cursor += 1;
    }
}

fn open_modal_for_cursor(app: &mut App) {
    let Some(entry) = app.progression.get(app.build_cursor) else {
        return;
    };
    let ProgressionEntry::Slot(state) = entry else {
        return;
    };
    let slot = state.slot;
    let level = state.level;

    // ClassFeature slots show info modal (read-only)
    if slot == BuildSlot::ClassFeature {
        if let Some(name) = &state.filled {
            let modal =
                crate::ui::modal::info::InfoModal::new(name);
            app.modal = Some(Modal::Info(Box::new(modal)));
            let _ = app.loader_tx.send(LoadRequest::ShowDocument {
                name: name.clone(),
                category: Some("class-feature".to_string()),
            });
        }
        return;
    }

    // Ability boosts open a custom modal
    if slot == BuildSlot::AbilityBoosts {
        open_boost_modal(app, level);
        return;
    }

    // Skill increases open a skill picker
    if slot == BuildSlot::SkillIncrease {
        open_skill_modal(app, level);
        return;
    }

    // Skill selection opens a picker for untrained skills
    if slot == BuildSlot::SkillSelection {
        open_skill_selection_modal(app);
        return;
    }

    // Subclass has a dynamic search category
    let category: String = if slot == BuildSlot::Subclass {
        let Some(cat) = app
            .choices
            .class_data
            .as_ref()
            .and_then(|d| d.subclass_category.clone())
        else {
            return;
        };
        cat
    } else if let Some(cat) = slot.search_category() {
        cat.to_string()
    } else {
        return;
    };

    let title = if slot == BuildSlot::Subclass {
        let label = app
            .choices
            .class
            .as_deref()
            .map(subclass::subclass_label)
            .unwrap_or("Subclass");
        format!("Select {label}")
    } else {
        format!("Select {}", slot.label())
    };
    let mut modal = crate::ui::modal::selection::SelectionModal::new(&title);

    // Build filters based on slot type and character state
    let mut filters: Vec<(String, String)> = Vec::new();

    match slot {
        BuildSlot::AncestryFeat => {
            if let Some(ancestry) = &app.choices.ancestry {
                filters.push(("trait".into(), ancestry.clone()));
            }
            modal.max_level = Some(level as i32);
        }
        BuildSlot::ClassFeat => {
            if let Some(class) = &app.choices.class {
                filters.push(("trait".into(), class.clone()));
            }
            modal.max_level = Some(level as i32);
        }
        BuildSlot::SkillFeat => {
            filters.push(("trait".into(), "Skill".into()));
            modal.max_level = Some(level as i32);
        }
        BuildSlot::GeneralFeat => {
            filters.push(("trait".into(), "General".into()));
            modal.max_level = Some(level as i32);
        }
        BuildSlot::Heritage => {
            if let Some(ancestry) = &app.choices.ancestry {
                modal.ancestry_filter = Some(ancestry.clone());
            }
        }
        _ => {}
    }

    app.modal = Some(Modal::Selection(modal));
    app.status = Some(format!("Loading {category}..."));
    let _ = app.loader_tx.send(LoadRequest::SearchCategory {
        category: category.clone(),
        filters,
    });
}

fn open_boost_modal(app: &mut App, level: u8) {
    let ancestry_spec = app
        .choices
        .ancestry_data
        .as_ref()
        .map(|d| &d.boost_spec)
        .cloned()
        .unwrap_or_default();
    let background_spec = app
        .choices
        .background_data
        .as_ref()
        .map(|d| &d.boost_spec)
        .cloned()
        .unwrap_or_default();
    let class_spec = app
        .choices
        .class_data
        .as_ref()
        .map(|d| &d.boost_spec)
        .cloned()
        .unwrap_or_default();

    let modal = if level == 1 {
        crate::ui::modal::boosts::BoostModal::new(
            &ancestry_spec,
            &background_spec,
            &class_spec,
            level,
        )
    } else {
        crate::ui::modal::boosts::BoostModal::new(
            &BoostSpec::default(),
            &BoostSpec::default(),
            &BoostSpec::default(),
            level,
        )
    };
    app.modal = Some(Modal::Boosts(modal));
}

fn open_skill_selection_modal(app: &mut App) {
    let modal = crate::ui::modal::skills::SkillModal::new_selection(
        &app.character.proficiencies.skills,
        &app.choices.initial_skills,
    );
    app.modal = Some(Modal::Skill(modal));
}

fn open_skill_modal(app: &mut App, level: u8) {
    // Max rank depends on level
    let max_rank = if level >= 15 {
        Rank::Legendary
    } else if level >= 7 {
        Rank::Master
    } else {
        Rank::Expert
    };
    let modal = crate::ui::modal::skills::SkillModal::new(
        &app.character.proficiencies.skills,
        &app.character.proficiencies.lores,
        max_rank,
    );
    app.modal = Some(Modal::Skill(modal));
}

fn apply_boosts(app: &mut App) {
    let Some(Modal::Boosts(modal)) = &app.modal else {
        return;
    };
    for group in &modal.groups {
        let choice_set = AbilityChoiceSet {
            fixed: group.fixed.clone(),
            chosen: group.chosen.clone(),
            flaws: group.flaws.clone(),
        };
        app.choices
            .ability_choices
            .insert(group.source.clone(), choice_set);
    }

    app.character = recalculate(&app.choices);
    sync_progression(&mut app.progression, &app.choices, &app.character);
}

fn apply_selection(app: &mut App, doc: &wayfinder_core::aon::Document) {
    let Some(entry) = app.progression.get(app.build_cursor) else {
        return;
    };
    let ProgressionEntry::Slot(state) = entry else {
        return;
    };
    let name = doc.name.clone().unwrap_or_default();
    let slot = state.slot;
    let level = state.level;

    match slot {
        BuildSlot::Ancestry => {
            // Cascade: clear heritage, ancestry feats, ancestry boosts
            if app.choices.ancestry.is_some() {
                app.choices.heritage = None;
                app.choices
                    .feats
                    .retain(|_, sel| sel.feat_type != crate::model::feat::FeatType::Ancestry);
                app.choices.ability_choices.remove("ancestry");
            }
            app.choices.ancestry = Some(name.clone());
            app.choices.ancestry_data = Some(ancestry::extract_ancestry_data(doc));
        }
        BuildSlot::Heritage => {
            app.choices.heritage = Some(name.clone());
        }
        BuildSlot::Background => {
            // Cascade
            app.choices.ability_choices.remove("background");
            app.choices.feats.retain(|k, _| k != "background_feat");
            app.choices.background = Some(name.clone());
            app.choices.background_data = Some(background::extract_background_data(doc));
        }
        BuildSlot::Class => {
            // Cascade
            if app.choices.class.is_some() {
                app.choices
                    .feats
                    .retain(|_, sel| sel.feat_type != crate::model::feat::FeatType::Class);
                app.choices.ability_choices.remove("class");
                app.choices.subclass = None;
            }
            app.choices.class = Some(name.clone());
            app.choices.class_data = Some(class::extract_class_data(doc));
            // Rebuild progression to reflect new class's subclass/deity/feature slots
            app.character = recalculate(&app.choices);
            app.progression = progression_for(&app.choices);
            sync_progression(&mut app.progression, &app.choices, &app.character);
            return;
        }
        BuildSlot::Subclass => {
            app.choices.subclass = Some(name.clone());
        }
        BuildSlot::Deity => {
            app.choices.deity = Some(name.clone());
            app.choices.deity_data = Some(deity::extract_deity_data(doc));
            app.choices.chosen_domains.clear();
            app.choices.chosen_divine_font = None;
            // Recalculate, then open divine font modal if needed
            app.character = recalculate(&app.choices);
            sync_progression(&mut app.progression, &app.choices, &app.character);
            maybe_open_font_modal(app);
            return;
        }
        BuildSlot::AncestryFeat
        | BuildSlot::ClassFeat
        | BuildSlot::SkillFeat
        | BuildSlot::GeneralFeat => {
            let slot_key = format!("{}_{level}", slot.label());
            let feat_type = match slot {
                BuildSlot::AncestryFeat => crate::model::feat::FeatType::Ancestry,
                BuildSlot::ClassFeat => crate::model::feat::FeatType::Class,
                BuildSlot::SkillFeat => crate::model::feat::FeatType::Skill,
                BuildSlot::GeneralFeat => crate::model::feat::FeatType::General,
                _ => unreachable!(),
            };
            let source_context = match slot {
                BuildSlot::AncestryFeat => app.choices.ancestry.clone(),
                BuildSlot::ClassFeat => app.choices.class.clone(),
                _ => None,
            };
            app.choices.feats.insert(
                slot_key,
                FeatSelection {
                    name: name.clone(),
                    feat_type,
                    level,
                    source_id: doc.id.clone(),
                    source_context,
                },
            );
        }
        _ => {}
    }

    app.character = recalculate(&app.choices);
    sync_progression(&mut app.progression, &app.choices, &app.character);
}

fn save_character(app: &mut App) {
    match crate::persistence::save::save_choices(&app.choices) {
        Ok(path) => app.status = Some(format!("Saved to {}", path.display())),
        Err(e) => app.status = Some(format!("Save failed: {e}")),
    }
}

/// Trim prepared spells to match available slot counts at current level.
fn trim_prepared_spells(choices: &mut crate::build::choices::BuildChoices) {
    let Some(data) = &choices.class_data else {
        return;
    };
    let level = choices.level as usize;
    let Some(slots) = data.spell_slots.get(level.saturating_sub(1)) else {
        choices.prepared_spells.clear();
        return;
    };
    let mut to_remove = Vec::new();
    for (&rank, spells) in &mut choices.prepared_spells {
        let max = slots
            .get(rank as usize)
            .and_then(|s| s.trim().parse::<usize>().ok())
            .unwrap_or(0);
        if max == 0 {
            to_remove.push(rank);
        } else {
            spells.truncate(max);
        }
    }
    for rank in to_remove {
        choices.prepared_spells.remove(&rank);
    }
}

fn handle_spell_up(app: &mut App) {
    app.spell_cursor = app.spell_cursor.saturating_sub(1);
}

fn handle_spell_down(app: &mut App) {
    let max = app.total_spell_slots().saturating_sub(1);
    if app.spell_cursor < max {
        app.spell_cursor += 1;
    }
}

fn open_spell_picker(app: &mut App) {
    // Check if cursor is in focus region
    if let Some(_focus_idx) = app.focus_spell_index() {
        open_focus_spell_picker(app);
        return;
    }

    let Some((rank, _slot_idx)) = app.spell_cursor_rank() else {
        return;
    };
    let Some(data) = &app.choices.class_data else {
        return;
    };
    let Some(tradition) = &data.tradition else {
        return;
    };

    let rank_label = if rank == 0 {
        "Cantrip".to_string()
    } else {
        format!("Rank {rank}")
    };
    let title = format!("Select Spell — {rank_label}");
    let mut modal = crate::ui::modal::selection::SelectionModal::new(&title);
    modal.max_level = Some(rank as i32);

    let mut filters = vec![("tradition".to_string(), tradition.clone())];
    if rank == 0 {
        filters.push(("trait".to_string(), "Cantrip".to_string()));
    }

    app.spell_pick_rank = Some(rank);
    app.modal = Some(Modal::Selection(modal));
    app.status = Some("Loading spells...".to_string());
    let _ = app.loader_tx.send(LoadRequest::SearchCategory {
        category: "spell".to_string(),
        filters,
    });
}

fn open_focus_spell_picker(app: &mut App) {
    let title = "Select Focus Spell".to_string();
    let modal = crate::ui::modal::selection::SelectionModal::new(&title);

    // Use rank 255 as sentinel for focus spell picks
    app.spell_pick_rank = Some(255);
    app.modal = Some(Modal::Selection(modal));
    app.status = Some("Loading focus spells...".to_string());
    let _ = app.loader_tx.send(LoadRequest::SearchCategory {
        category: "spell".to_string(),
        filters: vec![("trait".to_string(), "Focus".to_string())],
    });
}

fn apply_spell_selection(app: &mut App, doc: &wayfinder_core::aon::Document, rank: u8) {
    let name = doc.name.clone().unwrap_or_default();

    // Sentinel rank 255 = focus spell
    if rank == 255 {
        let choice = SpellSlotChoice {
            name,
            rank: 0,
            source_id: doc.id.clone(),
        };
        app.choices.focus_spells.push(choice);
        app.character = recalculate(&app.choices);
        return;
    }

    let choice = SpellSlotChoice {
        name,
        rank,
        source_id: doc.id.clone(),
    };
    app.choices
        .prepared_spells
        .entry(rank)
        .or_default()
        .push(choice);
    app.character = recalculate(&app.choices);
}

fn clear_spell_slot(app: &mut App) {
    // Check focus region first
    if let Some(focus_idx) = app.focus_spell_index() {
        if focus_idx < app.choices.focus_spells.len() {
            app.choices.focus_spells.remove(focus_idx);
            app.character = recalculate(&app.choices);
        }
        return;
    }

    let Some((rank, slot_idx)) = app.spell_cursor_rank() else {
        return;
    };
    if let Some(spells) = app.choices.prepared_spells.get_mut(&rank)
        && slot_idx < spells.len()
    {
        spells.remove(slot_idx);
        app.character = recalculate(&app.choices);
    }
}

fn export_character(app: &mut App) {
    match crate::persistence::export::export_pathbuilder(&app.choices, &app.character) {
        Ok(path) => app.status = Some(format!("Exported to {}", path.display())),
        Err(e) => app.status = Some(format!("Export failed: {e}")),
    }
}

// --- Equipment tab helpers ---

fn handle_equipment_up(app: &mut App) {
    app.equipment_cursor = app.equipment_cursor.saturating_sub(1);
}

fn handle_equipment_down(app: &mut App) {
    let max = app.equipment_row_count().saturating_sub(1);
    if app.equipment_cursor < max {
        app.equipment_cursor += 1;
    }
}

/// Map equipment_cursor to section info.
/// Returns (section, index_within_section, is_add_button).
/// Sections: 0=weapons, 1=armor, 2=shield, 3=items.
fn equipment_cursor_info(app: &App) -> (u8, usize, bool) {
    let eq = &app.choices.equipment;
    let mut pos = app.equipment_cursor;

    // Weapons section
    let wc = eq.weapons.len();
    if pos < wc {
        return (0, pos, false);
    }
    if pos == wc {
        return (0, 0, true); // "Add Weapon"
    }
    pos -= wc + 1;

    // Armor section
    let ac = eq.armor.len();
    if pos < ac {
        return (1, pos, false);
    }
    if pos == ac {
        return (1, 0, true); // "Add Armor"
    }
    pos -= ac + 1;

    // Shield section
    let sc = if eq.shield.is_some() { 1 } else { 0 };
    if pos < sc {
        return (2, 0, false);
    }
    if pos == sc {
        return (2, 0, true); // "Add Shield"
    }
    pos -= sc + 1;

    // Items section
    let ic = eq.items.len();
    if pos < ic {
        return (3, pos, false);
    }
    (3, 0, true) // "Add Item"
}

fn handle_equipment_enter(app: &mut App) {
    let (section, idx, is_add) = equipment_cursor_info(app);

    if is_add {
        // Open AON search modal for the appropriate category
        let (category, title) = match section {
            0 => ("weapon", "Add Weapon"),
            1 => ("armor", "Add Armor"),
            2 => ("shield", "Add Shield"),
            _ => ("equipment", "Add Item"),
        };
        let modal = crate::ui::modal::selection::SelectionModal::new(title);
        app.modal = Some(Modal::Selection(modal));
        app.status = Some(format!("Loading {category}..."));
        let _ = app.loader_tx.send(LoadRequest::SearchCategory {
            category: category.to_string(),
            filters: vec![],
        });
        return;
    }

    // Open rune modal for existing weapon/armor, or toggle shield raised
    match section {
        0 => {
            let w = &app.choices.equipment.weapons[idx];
            let modal = crate::ui::modal::runes::RuneModal::for_weapon(
                idx,
                &w.name,
                w.potency,
                w.striking,
            );
            app.modal = Some(Modal::Runes(modal));
        }
        1 => {
            let a = &app.choices.equipment.armor[idx];
            let modal = crate::ui::modal::runes::RuneModal::for_armor(
                idx,
                &a.name,
                a.potency,
                a.resilient,
            );
            app.modal = Some(Modal::Runes(modal));
        }
        2 => {
            // Toggle shield raised
            if let Some(s) = &mut app.choices.equipment.shield {
                s.raised = !s.raised;
                app.character = recalculate(&app.choices);
            }
        }
        _ => {
            // Items: no action on enter for now
        }
    }
}

fn handle_equipment_delete(app: &mut App) {
    let (section, idx, is_add) = equipment_cursor_info(app);
    if is_add {
        return;
    }

    match section {
        0 => {
            if idx < app.choices.equipment.weapons.len() {
                app.choices.equipment.weapons.remove(idx);
            }
        }
        1 => {
            if idx < app.choices.equipment.armor.len() {
                app.choices.equipment.armor.remove(idx);
            }
        }
        2 => {
            app.choices.equipment.shield = None;
        }
        3 => {
            if idx < app.choices.equipment.items.len() {
                app.choices.equipment.items.remove(idx);
            }
        }
        _ => {}
    }

    let max = app.equipment_row_count().saturating_sub(1);
    if app.equipment_cursor > max {
        app.equipment_cursor = max;
    }
    app.character = recalculate(&app.choices);
}

fn open_equipment_rune_modal(app: &mut App) {
    let (section, idx, is_add) = equipment_cursor_info(app);
    if is_add {
        return;
    }
    match section {
        0 => {
            let w = &app.choices.equipment.weapons[idx];
            let modal = crate::ui::modal::runes::RuneModal::for_weapon(
                idx,
                &w.name,
                w.potency,
                w.striking,
            );
            app.modal = Some(Modal::Runes(modal));
        }
        1 => {
            let a = &app.choices.equipment.armor[idx];
            let modal = crate::ui::modal::runes::RuneModal::for_armor(
                idx,
                &a.name,
                a.potency,
                a.resilient,
            );
            app.modal = Some(Modal::Runes(modal));
        }
        2 => {
            if let Some(s) = &app.choices.equipment.shield {
                let modal = crate::ui::modal::runes::RuneModal::for_shield(
                    &s.name,
                    s.potency,
                    s.resilient,
                );
                app.modal = Some(Modal::Runes(modal));
            }
        }
        _ => {}
    }
}

fn handle_rune_key(app: &mut App, code: KeyCode) -> anyhow::Result<()> {
    let Some(Modal::Runes(modal)) = &mut app.modal else {
        return Ok(());
    };
    match code {
        KeyCode::Esc => {
            app.modal = None;
        }
        KeyCode::Up => modal.move_up(),
        KeyCode::Down => modal.move_down(),
        KeyCode::Left => modal.cycle_left(),
        KeyCode::Right => modal.cycle_right(),
        KeyCode::Enter => {
            apply_runes(app);
            app.modal = None;
        }
        _ => {}
    }
    Ok(())
}

fn handle_domain_key(app: &mut App, code: KeyCode) -> anyhow::Result<()> {
    let Some(Modal::Domain(modal)) = &mut app.modal else {
        return Ok(());
    };
    match code {
        KeyCode::Esc => {
            app.modal = None;
        }
        KeyCode::Up => modal.move_up(),
        KeyCode::Down => modal.move_down(),
        KeyCode::Enter => {
            if let Some(value) = modal.selected().map(String::from) {
                let kind = modal.kind;
                app.modal = None;
                apply_domain_pick(app, kind, &value);
            }
        }
        _ => {}
    }
    Ok(())
}

fn apply_domain_pick(
    app: &mut App,
    kind: crate::ui::modal::domain::PickKind,
    value: &str,
) {
    use crate::ui::modal::domain::PickKind;
    match kind {
        PickKind::Domain => {
            app.choices.chosen_domains.push(value.to_string());
            app.character = recalculate(&app.choices);
            sync_progression(&mut app.progression, &app.choices, &app.character);
        }
        PickKind::DivineFont => {
            app.choices.chosen_divine_font = DivineFont::parse(value);
            app.character = recalculate(&app.choices);
            sync_progression(&mut app.progression, &app.choices, &app.character);
        }
    }
    // After domain pick, check if we should open divine font picker
    maybe_open_font_modal(app);
}

/// If the deity has multiple divine font options and none is chosen yet, open the picker.
fn maybe_open_font_modal(app: &mut App) {
    if app.choices.chosen_divine_font.is_some() {
        return;
    }
    let Some(deity_data) = &app.choices.deity_data else {
        return;
    };
    if deity_data.divine_font.len() <= 1 {
        // Auto-select if only one option
        if let Some(font) = deity_data.divine_font.first() {
            app.choices.chosen_divine_font = Some(*font);
        }
        return;
    }
    let items: Vec<String> = deity_data
        .divine_font
        .iter()
        .map(|f| f.label().to_string())
        .collect();
    let modal = crate::ui::modal::domain::DomainModal::new(
        "Select Divine Font",
        crate::ui::modal::domain::PickKind::DivineFont,
        items,
    );
    app.modal = Some(Modal::Domain(modal));
}

fn apply_runes(app: &mut App) {
    let Some(Modal::Runes(modal)) = &app.modal else {
        return;
    };
    use crate::ui::modal::runes::RuneTarget;
    match modal.target {
        RuneTarget::Weapon(idx) => {
            if let Some(w) = app.choices.equipment.weapons.get_mut(idx) {
                w.potency = modal.potency;
                w.striking = modal.striking;
            }
        }
        RuneTarget::Armor(idx) => {
            if let Some(a) = app.choices.equipment.armor.get_mut(idx) {
                a.potency = modal.potency;
                a.resilient = modal.resilient;
            }
        }
        RuneTarget::Shield => {
            if let Some(s) = &mut app.choices.equipment.shield {
                s.potency = modal.potency;
                s.resilient = modal.resilient;
            }
        }
    }
    app.character = recalculate(&app.choices);
}

/// Handle equipment-specific selection results from the search modal.
/// Called when a Selection modal result is from an equipment "Add" action.
pub fn apply_equipment_selection(app: &mut App, doc: &wayfinder_core::aon::Document) {
    let name = doc.name.clone().unwrap_or_default();
    let category = doc.category.as_deref().unwrap_or("");

    match category {
        "weapon" => {
            app.choices
                .equipment
                .weapons
                .push(equip_extract::extract_weapon(doc));
        }
        "armor" => {
            for a in &mut app.choices.equipment.armor {
                a.worn = false;
            }
            app.choices
                .equipment
                .armor
                .push(equip_extract::extract_armor(doc));
        }
        "shield" => {
            app.choices.equipment.shield = Some(equip_extract::extract_shield(doc));
        }
        _ => {
            app.choices.equipment.items.push(Item {
                name,
                quantity: 1,
                invested: false,
                worn: false,
            });
        }
    }

    app.character = recalculate(&app.choices);
}
