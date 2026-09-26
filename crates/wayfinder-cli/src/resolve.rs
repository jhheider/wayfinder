//! Turning user input into categories and names.

use anyhow::{Result, bail};
use colored::Colorize;

use wayfinder_core::Wayfinder;
use wayfinder_core::aon::parse::{CategoryError, parse_compound, resolve_category};

/// Resolve a category against the game's live list (see
/// [`Wayfinder::resolve_category`]). A near miss is used with a warning; an
/// unknown category is an error.
pub async fn cli_resolve_category(wf: &Wayfinder, input: &str) -> Result<String> {
    match wf.resolve_category(input).await {
        Ok(cat) => Ok(cat),
        Err(CategoryError::Suggested { input, suggestion }) => {
            eprintln!(
                "{} Unknown category '{}', using '{}'",
                "⚠".yellow(),
                input.red(),
                suggestion.green()
            );
            Ok(suggestion)
        }
        Err(CategoryError::Unknown(input)) => {
            bail!(
                "Unknown category '{}'. Run {} to see all available categories.",
                input.red(),
                "wf categories".cyan()
            );
        }
    }
}

/// Whether `s` names a known AON category (normalizing plural/case).
fn is_known_category(s: &str) -> bool {
    resolve_category(s).is_ok()
}

/// Resolve a `show` query into `(category, name)`, supporting all documented
/// forms: `spell/Fireball` (slash), `spell Fireball` (first token is a known
/// category), `Grab an Edge` (multi-word name), and `Fireball` (bare name).
pub fn resolve_show_query(query: &[String]) -> (Option<String>, Option<String>) {
    let joined = query.join(" ");
    let (cat, name) = parse_compound(&joined);
    // Only treat a space-separated first token as a category when it actually
    // is one, so multi-word names like "Grab an Edge" stay intact.
    if cat.is_none() && query.len() >= 2 && is_known_category(&query[0]) {
        return (Some(query[0].clone()), Some(query[1..].join(" ")));
    }
    (cat, name)
}

#[cfg(test)]
mod tests {
    use super::{is_known_category, resolve_show_query};

    fn q(parts: &[&str]) -> Vec<String> {
        parts.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn is_known_category_normalizes_plurals_and_case() {
        assert!(is_known_category("spell"));
        assert!(is_known_category("Spells"));
        assert!(is_known_category("deities"));
        // Regression: "rules" de-pluralized to "rule", so `wf show rules Flanking`
        // looked up the name "rules Flanking" in every category.
        assert!(is_known_category("rules"));
        assert!(!is_known_category("wizardry"));
    }

    #[test]
    fn slash_form_splits_category_and_name() {
        assert_eq!(
            resolve_show_query(&q(&["spell/Fireball"])),
            (Some("spell".into()), Some("Fireball".into()))
        );
    }

    #[test]
    fn space_form_splits_when_first_token_is_a_category() {
        assert_eq!(
            resolve_show_query(&q(&["spell", "Fireball"])),
            (Some("spell".into()), Some("Fireball".into()))
        );
    }

    #[test]
    fn multi_word_name_without_category_stays_intact() {
        assert_eq!(
            resolve_show_query(&q(&["Grab", "an", "Edge"])),
            (None, Some("Grab an Edge".into()))
        );
    }

    #[test]
    fn bare_name() {
        assert_eq!(
            resolve_show_query(&q(&["Fireball"])),
            (None, Some("Fireball".into()))
        );
    }
}
