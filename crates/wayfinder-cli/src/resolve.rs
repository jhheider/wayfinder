//! Turning user input into categories, names, and paths.

use anyhow::{Result, bail};
use colored::Colorize;
use std::path::PathBuf;

use wayfinder_core::aon::categories::ALL_CATEGORIES;
use wayfinder_core::aon::client::GameSystem;
use wayfinder_core::aon::parse::{
    CategoryError, normalize_category, parse_compound, resolve_category,
};

use crate::cli::Cli;

/// Resolve a category string, printing warnings/errors with color.
pub fn cli_resolve_category(input: &str) -> Result<String> {
    match resolve_category(input) {
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

pub fn cache_path() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("wayfinder")
        .join("wayfinder_cache.db")
}

pub fn game_system(cli: &Cli) -> GameSystem {
    if cli.sf2e {
        GameSystem::Starfinder
    } else {
        GameSystem::Pathfinder
    }
}

/// Whether `s` names a known AON category (normalizing plural/case).
fn is_known_category(s: &str) -> bool {
    ALL_CATEGORIES.contains(&normalize_category(s).as_str())
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
