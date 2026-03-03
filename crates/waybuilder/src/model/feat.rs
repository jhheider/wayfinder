use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FeatType {
    Ancestry,
    Class,
    General,
    Skill,
    Heritage,
    Bonus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatChoice {
    pub name: String,
    pub feat_type: FeatType,
    pub level: u8,
    pub slot: String,
    pub source_id: Option<String>,
}
