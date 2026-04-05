use tokio::sync::mpsc;
use wayfinder_core::aon::GameSystem;

use crate::build::choices::BuildChoices;
use crate::build::progression::{ProgressionEntry, build_progression_with_class};
use crate::build::recalculate::recalculate;
use crate::data::loader::{self, LoadRequest, LoadResult};
use crate::model::character::Character;
use crate::ui::modal::boosts::BoostModal;
use crate::ui::modal::domain::DomainModal;
use crate::ui::modal::info::InfoModal;
use crate::ui::modal::runes::RuneModal;
use crate::ui::modal::selection::SelectionModal;
use crate::ui::modal::skills::SkillModal;
use crate::ui::modal::text_input::TextInputModal;

/// Which top-level screen is active.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    CharacterSelect,
    Builder,
}

/// Active modal dialog.
pub enum Modal {
    Selection(SelectionModal),
    Boosts(BoostModal),
    Skill(SkillModal),
    TextInput(TextInputModal),
    Info(Box<InfoModal>),
    Runes(RuneModal),
    Domain(DomainModal),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Focus {
    BuildSelector,
    CharacterInfo,
    DetailTabs,
}

impl Focus {
    pub fn next(self) -> Self {
        match self {
            Focus::BuildSelector => Focus::CharacterInfo,
            Focus::CharacterInfo => Focus::DetailTabs,
            Focus::DetailTabs => Focus::BuildSelector,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DetailTab {
    Equipment,
    Spells,
    Skills,
    Feats,
}

impl DetailTab {
    pub const ALL: [DetailTab; 4] = [
        DetailTab::Equipment,
        DetailTab::Spells,
        DetailTab::Skills,
        DetailTab::Feats,
    ];

    pub fn label(&self) -> &'static str {
        match self {
            DetailTab::Equipment => "Equipment",
            DetailTab::Spells => "Spells",
            DetailTab::Skills => "Skills",
            DetailTab::Feats => "Feats",
        }
    }

    pub fn next(self) -> Self {
        match self {
            DetailTab::Equipment => DetailTab::Spells,
            DetailTab::Spells => DetailTab::Skills,
            DetailTab::Skills => DetailTab::Feats,
            DetailTab::Feats => DetailTab::Equipment,
        }
    }

    pub fn prev(self) -> Self {
        match self {
            DetailTab::Equipment => DetailTab::Feats,
            DetailTab::Spells => DetailTab::Equipment,
            DetailTab::Skills => DetailTab::Spells,
            DetailTab::Feats => DetailTab::Skills,
        }
    }
}

pub struct App {
    pub screen: Screen,
    pub choices: BuildChoices,
    pub character: Character,
    pub focus: Focus,
    pub detail_tab: DetailTab,
    pub progression: Vec<ProgressionEntry>,
    pub build_cursor: usize,
    pub quit: bool,
    pub show_help: bool,
    pub modal: Option<Modal>,
    pub loader_tx: mpsc::UnboundedSender<LoadRequest>,
    pub loader_rx: mpsc::UnboundedReceiver<LoadResult>,
    pub status: Option<String>,
    pub tick: usize,
    /// Saved character files for the select screen.
    pub saved_characters: Vec<std::path::PathBuf>,
    pub select_cursor: usize,
    pub spell_cursor: usize,
    /// Pending spell rank when a spell picker modal is open.
    pub spell_pick_rank: Option<u8>,
    /// Cursor for the Equipment tab (flat index across sections).
    pub equipment_cursor: usize,
}

impl App {
    pub fn new(system: GameSystem) -> Self {
        let choices = BuildChoices::new("Adventurer");
        let character = recalculate(&choices);
        let progression = progression_for(&choices);
        let (loader_tx, loader_rx) = loader::spawn_loader(system);
        // Preload common categories in the background
        for cat in [
            "ancestry",
            "heritage",
            "background",
            "class",
            "deity",
            "feat",
        ] {
            let _ = loader_tx.send(LoadRequest::SearchCategory {
                category: cat.to_string(),
                filters: vec![],
            });
        }
        let saved_characters = crate::persistence::save::list_characters().unwrap_or_default();
        let screen = if saved_characters.is_empty() {
            Screen::Builder
        } else {
            Screen::CharacterSelect
        };
        let build_cursor = first_slot_index(&progression);
        App {
            screen,
            choices,
            character,
            focus: Focus::BuildSelector,
            detail_tab: DetailTab::Equipment,
            progression,
            build_cursor,
            quit: false,
            show_help: false,
            modal: None,
            loader_tx,
            loader_rx,
            status: None,
            tick: 0,
            saved_characters,
            select_cursor: 0,
            spell_cursor: 0,
            spell_pick_rank: None,
            equipment_cursor: 0,
        }
    }

    /// Total number of spell slots visible in the Spells tab
    /// (prepared slots + focus spell rows including one empty "add" row).
    pub fn total_spell_slots(&self) -> usize {
        let prepared = self.prepared_slot_count();
        // Focus region: existing focus spells + 1 empty add slot (only if caster)
        let focus = if self.character.spell_caster.is_some() {
            self.choices.focus_spells.len() + 1
        } else {
            0
        };
        prepared + focus
    }

    /// Count of prepared spell slots only (no focus).
    pub fn prepared_slot_count(&self) -> usize {
        let Some(data) = &self.choices.class_data else {
            return 0;
        };
        let level = self.choices.level as usize;
        let Some(slots) = data.spell_slots.get(level.saturating_sub(1)) else {
            return 0;
        };
        slots
            .iter()
            .filter_map(|s| {
                let s = s.trim();
                if s == "—" || s == "-" || s.is_empty() {
                    None
                } else {
                    s.parse::<usize>().ok()
                }
            })
            .sum()
    }

    /// Map flat spell_cursor index to (rank, slot_index).
    /// Returns `None` if cursor is in focus region (use `focus_spell_index` instead).
    pub fn spell_cursor_rank(&self) -> Option<(u8, usize)> {
        let data = self.choices.class_data.as_ref()?;
        let level = self.choices.level as usize;
        let slots = data.spell_slots.get(level.saturating_sub(1))?;
        let mut flat = 0usize;
        for (rank_idx, slot_str) in slots.iter().enumerate() {
            let s = slot_str.trim();
            if s == "—" || s == "-" || s.is_empty() {
                continue;
            }
            let count: usize = s.parse().ok()?;
            if self.spell_cursor < flat + count {
                return Some((rank_idx as u8, self.spell_cursor - flat));
            }
            flat += count;
        }
        None
    }

    /// If the spell cursor is in the focus spell region, return the index
    /// within the focus spell list. Returns `None` if cursor is in prepared region.
    /// The index may equal `choices.focus_spells.len()` meaning the "add" row.
    pub fn focus_spell_index(&self) -> Option<usize> {
        let prepared = self.prepared_slot_count();
        if self.spell_cursor >= prepared {
            Some(self.spell_cursor - prepared)
        } else {
            None
        }
    }

    /// Total number of selectable rows in the Equipment tab.
    /// Weapons + "Add Weapon" + Armor items + "Add Armor" + Shield or "Add Shield" + Items + "Add Item".
    pub fn equipment_row_count(&self) -> usize {
        let eq = &self.choices.equipment;
        let weapons = eq.weapons.len() + 1; // +1 for "Add Weapon"
        let armor = eq.armor.len() + 1; // +1 for "Add Armor"
        let shield = (if eq.shield.is_some() { 1 } else { 0 }) + 1; // item or nothing + "Add Shield"
        let items = eq.items.len() + 1; // +1 for "Add Item"
        weapons + armor + shield + items
    }

    pub fn tick(&mut self) {
        self.tick = self.tick.wrapping_add(1);
    }

    pub fn spinner(&self) -> char {
        const FRAMES: &[char] = &['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];
        FRAMES[self.tick / 2 % FRAMES.len()]
    }

    /// Start a new character and switch to builder screen.
    pub fn new_character(&mut self) {
        self.choices = BuildChoices::new("Adventurer");
        self.character = recalculate(&self.choices);
        self.progression = progression_for(&self.choices);
        self.build_cursor = first_slot_index(&self.progression);
        self.screen = Screen::Builder;
    }

    /// Load a character from file and switch to builder screen.
    pub fn load_from_file(&mut self, path: &std::path::Path) {
        match crate::persistence::save::load_choices(path) {
            Ok(choices) => {
                let character = recalculate(&choices);
                let mut progression = progression_for(&choices);
                sync_progression(&mut progression, &choices, &character);
                self.build_cursor = first_slot_index(&progression);
                self.choices = choices;
                self.character = character;
                self.progression = progression;
                self.screen = Screen::Builder;
            }
            Err(e) => {
                self.status = Some(format!("Load failed: {e}"));
            }
        }
    }

    /// Refresh saved character list (e.g., after save/delete).
    pub fn refresh_saved_list(&mut self) {
        self.saved_characters = crate::persistence::save::list_characters().unwrap_or_default();
    }

    /// Drain any pending loader results into the modal.
    pub fn poll_loader(&mut self) {
        while let Ok(result) = self.loader_rx.try_recv() {
            match result {
                LoadResult::Documents(docs) => {
                    match &mut self.modal {
                        Some(Modal::Selection(modal)) => {
                            modal.items = docs;
                        }
                        Some(Modal::Info(modal)) => {
                            modal.doc = docs.into_iter().next();
                        }
                        _ => {}
                    }
                    self.status = None;
                }
                LoadResult::Error(e) => {
                    self.status = Some(format!("Error: {e}"));
                }
            }
        }
    }
}

/// Build progression from choices, using class-aware slots.
pub fn progression_for(choices: &BuildChoices) -> Vec<ProgressionEntry> {
    let features = choices
        .class_data
        .as_ref()
        .map(|d| d.class_features.as_slice())
        .unwrap_or(&[]);
    build_progression_with_class(20, Some(choices), features)
}

/// Populate progression slot `filled` fields from `BuildChoices`,
/// then validate each filled slot against current state.
pub fn sync_progression(
    progression: &mut [ProgressionEntry],
    choices: &BuildChoices,
    character: &Character,
) {
    use crate::build::slot::BuildSlot;

    for entry in progression.iter_mut() {
        let ProgressionEntry::Slot(state) = entry else {
            continue;
        };
        state.filled = match state.slot {
            BuildSlot::Ancestry => choices.ancestry.clone(),
            BuildSlot::Heritage => choices.heritage.clone(),
            BuildSlot::Background => choices.background.clone(),
            BuildSlot::Class => choices.class.clone(),
            BuildSlot::Deity => choices.deity.clone(),
            BuildSlot::Subclass => choices.subclass.clone(),
            BuildSlot::AbilityBoosts => {
                if state.level == 1 {
                    let has = choices.ability_choices.contains_key("level_1")
                        || choices.ability_choices.contains_key("ancestry")
                        || choices.ability_choices.contains_key("background")
                        || choices.ability_choices.contains_key("class");
                    if has { Some("Done".to_string()) } else { None }
                } else {
                    let key = format!("level_{}", state.level);
                    if choices.ability_choices.contains_key(&key) {
                        Some("Done".to_string())
                    } else {
                        None
                    }
                }
            }
            BuildSlot::AncestryFeat
            | BuildSlot::ClassFeat
            | BuildSlot::SkillFeat
            | BuildSlot::GeneralFeat => {
                let slot_key = format!("{}_{}", state.slot.label(), state.level);
                choices.feats.get(&slot_key).map(|f| f.name.clone())
            }
            BuildSlot::SkillIncrease => choices.skill_increases.get(&state.level).cloned(),
            // ClassFeature slots are pre-filled by build_progression_with_class
            BuildSlot::ClassFeature => state.filled.clone(),
            BuildSlot::SkillSelection => {
                // Match SkillSelection slots to initial_skills by index
                None // filled below
            }
        };
    }

    // Fill SkillSelection slots from initial_skills by index
    let mut skill_idx = 0;
    for entry in progression.iter_mut() {
        if let ProgressionEntry::Slot(state) = entry
            && state.slot == BuildSlot::SkillSelection
        {
            state.filled = choices.initial_skills.get(skill_idx).cloned();
            skill_idx += 1;
        }
    }

    crate::build::validate::validate_progression(progression, choices, character);
}

/// Find the index of the first Slot entry in the progression.
fn first_slot_index(progression: &[ProgressionEntry]) -> usize {
    progression
        .iter()
        .position(|e| matches!(e, ProgressionEntry::Slot(_)))
        .unwrap_or(0)
}
