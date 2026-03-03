//! Map classes to their subclass AON categories and extract subclass data.

/// Return the AON search category for a class's subclass, if any.
pub fn subclass_category(class_name: &str) -> Option<&'static str> {
    match class_name.to_lowercase().as_str() {
        "sorcerer" => Some("bloodline"),
        "champion" => Some("cause"),
        "cleric" => Some("doctrine"),
        "druid" => Some("druidic-order"),
        "wizard" => Some("arcane-school"),
        "barbarian" => Some("instinct"),
        "ranger" => Some("hunters-edge"),
        "rogue" => Some("racket"),
        "investigator" => Some("methodology"),
        "oracle" => Some("mystery"),
        "swashbuckler" => Some("style"),
        "witch" => Some("patron"),
        "magus" => Some("hybrid-study"),
        "summoner" => Some("eidolon"),
        "psychic" => Some("conscious-mind"),
        "thaumaturge" => Some("implement"),
        "gunslinger" => Some("way"),
        "inventor" => Some("innovation"),
        "kineticist" => Some("element"),
        _ => None,
    }
}

/// Human-readable label for a subclass slot.
pub fn subclass_label(class_name: &str) -> &'static str {
    match class_name.to_lowercase().as_str() {
        "sorcerer" => "Bloodline",
        "champion" => "Cause",
        "cleric" => "Doctrine",
        "druid" => "Druidic Order",
        "wizard" => "Arcane School",
        "barbarian" => "Instinct",
        "ranger" => "Hunter's Edge",
        "rogue" => "Racket",
        "investigator" => "Methodology",
        "oracle" => "Mystery",
        "swashbuckler" => "Style",
        "witch" => "Patron",
        "magus" => "Hybrid Study",
        "summoner" => "Eidolon",
        "psychic" => "Conscious Mind",
        "thaumaturge" => "Implement",
        "gunslinger" => "Way",
        "inventor" => "Innovation",
        "kineticist" => "Element",
        _ => "Subclass",
    }
}

/// Does this class require a deity selection?
pub fn requires_deity(class_name: &str) -> bool {
    matches!(
        class_name.to_lowercase().as_str(),
        "champion" | "cleric" | "oracle"
    )
}
