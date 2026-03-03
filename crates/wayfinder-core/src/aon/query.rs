use serde_json::{Value, json};

/// All valid filter field names (sorted union of filterable_fields across categories).
pub const ALLOWED_FILTER_FIELDS: &[&str] = &[
    "actions",
    "alignment",
    "archetype",
    "archetype_category",
    "area_of_concern",
    "aspect",
    "attack_proficiency",
    "attribute",
    "attribute_flaw",
    "bloodline",
    "cleric_spell",
    "component",
    "creature_ability",
    "damage_type",
    "defense_proficiency",
    "deity",
    "divine_font",
    "domain",
    "domain_alternate",
    "domain_primary",
    "element",
    "familiar_ability",
    "favored_weapon",
    "feat",
    "follower_alignment",
    "heighten_group",
    "hp",
    "immunity",
    "language",
    "level",
    "pantheon",
    "patron_theme",
    "rarity",
    "sanctification",
    "saving_throw",
    "school",
    "size",
    "skill",
    "skill_proficiency",
    "spell",
    "spell_type",
    "strongest_save",
    "tradition",
    "trait",
    "trait_group",
    "weakest_save",
];

/// Check whether a field name is a known filterable field.
pub fn is_valid_filter_field(field: &str) -> bool {
    ALLOWED_FILTER_FIELDS.binary_search(&field).is_ok()
}

/// Check whether a field is valid for a specific category.
/// Falls back to the global whitelist if the category has no field info.
pub fn is_valid_filter_for_category(field: &str, category: &str) -> bool {
    use super::categories::filterable_fields;
    match filterable_fields(category) {
        Some(fields) => fields.contains(&field),
        None => is_valid_filter_field(field),
    }
}

/// Builder for AON Elasticsearch queries.
#[derive(Debug, Clone, Default)]
pub struct SearchQuery {
    category: Option<String>,
    name: Option<String>,
    filters: Vec<(String, String)>,
    text: Option<String>,
    broad: Option<String>,
    size: u32,
    from: u32,
}

impl SearchQuery {
    pub fn new() -> Self {
        Self {
            size: 50,
            ..Default::default()
        }
    }

    pub fn category(mut self, cat: &str) -> Self {
        self.category = Some(cat.to_string());
        self
    }

    pub fn name(mut self, name: &str) -> Self {
        self.name = Some(name.to_string());
        self
    }

    pub fn filter(mut self, field: &str, value: &str) -> Self {
        self.filters.push((field.to_string(), value.to_string()));
        self
    }

    pub fn text(mut self, q: &str) -> Self {
        self.text = Some(q.to_string());
        self
    }

    /// Broad search across name and text fields (OR semantics).
    pub fn broad(mut self, q: &str) -> Self {
        self.broad = Some(q.to_string());
        self
    }

    pub fn size(mut self, n: u32) -> Self {
        self.size = n;
        self
    }

    pub fn from(mut self, n: u32) -> Self {
        self.from = n;
        self
    }

    /// Build the Elasticsearch JSON query body.
    pub fn build(&self) -> Value {
        let mut musts: Vec<Value> = Vec::new();

        if let Some(cat) = &self.category {
            musts.push(json!({ "term": { "category": cat } }));
        }

        if let Some(name) = &self.name {
            musts.push(json!({ "match_phrase": { "name": name } }));
        }

        if let Some(text) = &self.text {
            musts.push(json!({ "match": { "text": text } }));
        }

        if let Some(q) = &self.broad {
            musts.push(json!({
                "multi_match": {
                    "query": q,
                    "fields": ["name^3", "text"],
                    "type": "best_fields"
                }
            }));
        }

        for (field, value) in self.filters.iter().take(20) {
            musts.push(json!({ "term": { field: value } }));
        }

        json!({
            "query": { "bool": { "must": musts } },
            "size": self.size,
            "from": self.from,
        })
    }
}
