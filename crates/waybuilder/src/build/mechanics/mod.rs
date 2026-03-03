//! Mechanics extractor: AON document data -> structured game data.
//!
//! Each submodule handles one domain (ancestry, background, class, etc.)
//! and provides `extract_*` functions that take an AON `Document` and
//! return typed game data.

pub mod ancestry;
pub mod background;
pub mod boosts;
pub mod class;
pub mod deity;
pub mod equipment;
pub mod proficiency;
pub mod skills;
pub mod subclass;

use crate::model::proficiencies::Rank;

/// A grant of proficiency rank in a named target.
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)]
pub struct ProficiencyGrant {
    pub target: String,
    pub rank: Rank,
}

/// Error type for mechanics extraction failures.
#[derive(Debug, thiserror::Error)]
#[allow(dead_code)]
pub enum MechanicsError {
    #[error("missing field: {0}")]
    MissingField(String),
    #[error("parse error: {0}")]
    Parse(String),
}
