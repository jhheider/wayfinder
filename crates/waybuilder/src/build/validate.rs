use crate::build::choices::BuildChoices;
use crate::build::progression::ProgressionEntry;
use crate::build::slot::BuildSlot;
use crate::model::character::Character;
use crate::model::proficiencies::Rank;

/// Mark filled slots as invalid when their prerequisites no longer hold.
pub fn validate_progression(
    progression: &mut [ProgressionEntry],
    choices: &BuildChoices,
    character: &Character,
) {
    for entry in progression.iter_mut() {
        let ProgressionEntry::Slot(state) = entry else {
            continue;
        };
        if state.filled.is_none() {
            state.valid = true;
            continue;
        }
        state.valid = match state.slot {
            BuildSlot::Heritage => choices.ancestry.is_some(),
            BuildSlot::AncestryFeat => {
                let slot_key = format!("{}_{}", state.slot.label(), state.level);
                choices
                    .feats
                    .get(&slot_key)
                    .and_then(|f| f.source_context.as_ref())
                    .is_none_or(|ctx| choices.ancestry.as_ref() == Some(ctx))
            }
            BuildSlot::ClassFeat => {
                let slot_key = format!("{}_{}", state.slot.label(), state.level);
                choices
                    .feats
                    .get(&slot_key)
                    .and_then(|f| f.source_context.as_ref())
                    .is_none_or(|ctx| choices.class.as_ref() == Some(ctx))
            }
            BuildSlot::SkillIncrease => {
                if let Some(skill) = choices.skill_increases.get(&state.level) {
                    is_skill_trained(character, skill)
                } else {
                    true
                }
            }
            BuildSlot::AbilityBoosts if state.level == 1 => {
                let needs_ancestry = choices.ability_choices.contains_key("ancestry");
                let needs_bg = choices.ability_choices.contains_key("background");
                let needs_class = choices.ability_choices.contains_key("class");
                (!needs_ancestry || choices.ancestry_data.is_some())
                    && (!needs_bg || choices.background_data.is_some())
                    && (!needs_class || choices.class_data.is_some())
            }
            BuildSlot::Subclass => choices
                .class_data
                .as_ref()
                .and_then(|d| d.subclass_category.as_ref())
                .is_some(),
            _ => true,
        };
    }
}

fn is_skill_trained(character: &Character, skill: &str) -> bool {
    character
        .proficiencies
        .skills
        .iter()
        .any(|(name, rank)| name == skill && *rank >= Rank::Trained)
        || character
            .proficiencies
            .lores
            .iter()
            .any(|(name, rank)| name == skill && *rank >= Rank::Trained)
}
