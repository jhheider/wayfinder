use crate::build::choices::{BuildChoices, ClassFeatureEntry};

use super::slot::{BuildSlot, SlotState};

/// A level header or a build slot entry in the progression list.
#[derive(Debug, Clone)]
pub enum ProgressionEntry {
    LevelHeader(u8),
    Slot(SlotState),
}

/// Generate the full level-by-level progression for a character.
pub fn build_progression(max_level: u8) -> Vec<ProgressionEntry> {
    build_progression_with_class(max_level, None, &[])

}

/// Generate progression with class-aware slots (subclass, deity, class features).
pub fn build_progression_with_class(
    max_level: u8,
    choices: Option<&BuildChoices>,
    class_features: &[ClassFeatureEntry],
) -> Vec<ProgressionEntry> {
    let has_subclass = choices
        .and_then(|c| c.class_data.as_ref())
        .and_then(|d| d.subclass_category.as_ref())
        .is_some();
    let needs_deity = choices
        .and_then(|c| c.class.as_deref())
        .is_some_and(|name| {
            crate::build::mechanics::subclass::requires_deity(name)
        });

    let mut entries = Vec::new();
    for lvl in 1..=max_level.max(1) {
        entries.push(ProgressionEntry::LevelHeader(lvl));
        if lvl == 1 {
            level_1_slots(&mut entries, has_subclass, needs_deity);
            skill_selection_slots(&mut entries, choices);
            // Class features at level 1
            for feat in class_features.iter().filter(|f| f.level == 1) {
                if feat.is_feat_slot {
                    entries.push(ProgressionEntry::Slot(SlotState::new(
                        BuildSlot::ClassFeat,
                        1,
                    )));
                } else {
                    let mut state = SlotState::new(BuildSlot::ClassFeature, 1);
                    state.filled = Some(feat.name.clone());
                    entries.push(ProgressionEntry::Slot(state));
                }
            }
        }
        if lvl >= 2 {
            level_n_slots(&mut entries, lvl, class_features);
            // Class features at this level
            for feat in class_features
                .iter()
                .filter(|f| f.level == lvl && !f.is_feat_slot)
            {
                let mut state = SlotState::new(BuildSlot::ClassFeature, lvl);
                state.filled = Some(feat.name.clone());
                entries.push(ProgressionEntry::Slot(state));
            }
        }
    }
    entries
}

fn level_1_slots(
    entries: &mut Vec<ProgressionEntry>,
    has_subclass: bool,
    needs_deity: bool,
) {
    entries.push(ProgressionEntry::Slot(SlotState::new(
        BuildSlot::Ancestry,
        1,
    )));
    entries.push(ProgressionEntry::Slot(SlotState::new(
        BuildSlot::Heritage,
        1,
    )));
    entries.push(ProgressionEntry::Slot(SlotState::new(
        BuildSlot::Background,
        1,
    )));
    entries.push(ProgressionEntry::Slot(SlotState::new(
        BuildSlot::Class,
        1,
    )));
    if has_subclass {
        entries.push(ProgressionEntry::Slot(SlotState::new(
            BuildSlot::Subclass,
            1,
        )));
    }
    if needs_deity {
        entries.push(ProgressionEntry::Slot(SlotState::new(
            BuildSlot::Deity,
            1,
        )));
    }
    entries.push(ProgressionEntry::Slot(SlotState::new(
        BuildSlot::AbilityBoosts,
        1,
    )));
    entries.push(ProgressionEntry::Slot(SlotState::new(
        BuildSlot::AncestryFeat,
        1,
    )));
}

fn skill_selection_slots(
    entries: &mut Vec<ProgressionEntry>,
    choices: Option<&BuildChoices>,
) {
    let count = choices
        .and_then(|c| c.class_data.as_ref())
        .map(|d| d.additional_skill_count)
        .unwrap_or(0);
    for _ in 0..count {
        entries.push(ProgressionEntry::Slot(SlotState::new(
            BuildSlot::SkillSelection,
            1,
        )));
    }
}

fn level_n_slots(
    entries: &mut Vec<ProgressionEntry>,
    level: u8,
    class_features: &[ClassFeatureEntry],
) {
    // Class feat at even levels, or whenever the class table has a feat slot at this level
    let has_feat_slot = class_features
        .iter()
        .any(|f| f.level == level && f.is_feat_slot);
    if level.is_multiple_of(2) || has_feat_slot {
        entries.push(ProgressionEntry::Slot(SlotState::new(
            BuildSlot::ClassFeat,
            level,
        )));
    }
    // Skill feat at 2 and even levels
    if level.is_multiple_of(2) {
        entries.push(ProgressionEntry::Slot(SlotState::new(
            BuildSlot::SkillFeat,
            level,
        )));
    }
    // General feat at odd levels >= 3
    if level >= 3 && level % 2 == 1 {
        entries.push(ProgressionEntry::Slot(SlotState::new(
            BuildSlot::GeneralFeat,
            level,
        )));
    }
    // Skill increase every level >= 2
    entries.push(ProgressionEntry::Slot(SlotState::new(
        BuildSlot::SkillIncrease,
        level,
    )));
    // Ability boosts at 5, 10, 15, 20
    if level.is_multiple_of(5) {
        entries.push(ProgressionEntry::Slot(SlotState::new(
            BuildSlot::AbilityBoosts,
            level,
        )));
    }
    // Ancestry feat at 5, 9, 13, 17
    if [5, 9, 13, 17].contains(&level) {
        entries.push(ProgressionEntry::Slot(SlotState::new(
            BuildSlot::AncestryFeat,
            level,
        )));
    }
}
