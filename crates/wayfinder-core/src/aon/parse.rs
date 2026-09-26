/// Parse "category/name" or plain "name" from a compound input string.
pub fn parse_compound(input: &str) -> (Option<String>, Option<String>) {
    if let Some((cat, name)) = input.split_once('/') {
        let cat = if cat.is_empty() {
            None
        } else {
            Some(cat.to_string())
        };
        let name = if name.is_empty() {
            None
        } else {
            Some(name.to_string())
        };
        (cat, name)
    } else {
        (None, Some(input.to_string()))
    }
}

/// Error from resolving a user-supplied category string.
#[derive(Debug, PartialEq, Eq)]
pub enum CategoryError {
    /// The input was unknown but a close match exists.
    Suggested { input: String, suggestion: String },
    /// No match or suggestion found.
    Unknown(String),
}

impl std::fmt::Display for CategoryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Suggested { input, suggestion } => {
                write!(f, "Unknown category '{input}', using '{suggestion}'")
            }
            Self::Unknown(input) => {
                write!(f, "Unknown category '{input}'")
            }
        }
    }
}

impl std::error::Error for CategoryError {}

/// Resolve a user-supplied category string to a known category.
///
/// Normalizes, checks for exact match, then tries fuzzy suggestion.
pub fn resolve_category(input: &str) -> Result<String, CategoryError> {
    resolve_category_in(input, super::categories::ALL_CATEGORIES)
}

/// Resolve a category string against a caller-supplied set of categories (e.g.
/// the live list for one game), like [`resolve_category`].
///
/// The input is matched as given (case-insensitively) before it is
/// de-pluralized, so categories that end in `s` (`rules`) resolve to
/// themselves.
pub fn resolve_category_in<S: AsRef<str>>(
    input: &str,
    known: &[S],
) -> Result<String, CategoryError> {
    let lower = input.trim().to_lowercase();
    let normalized = normalize_category(&lower);
    for candidate in [&lower, &normalized] {
        if known.iter().any(|k| k.as_ref() == candidate) {
            return Ok(candidate.clone());
        }
    }
    if let Some(suggestion) = super::categories::suggest_from(&normalized, known) {
        return Err(CategoryError::Suggested {
            input: input.to_string(),
            suggestion: suggestion.as_ref().to_string(),
        });
    }
    Err(CategoryError::Unknown(input.to_string()))
}

/// Normalize a category string: lowercase and de-pluralize.
pub fn normalize_category(input: &str) -> String {
    let mut s = input.to_lowercase();
    if s.ends_with("ies") {
        s.truncate(s.len() - 3);
        s.push('y');
    } else if s.ends_with('s') && !s.ends_with("ss") {
        s.truncate(s.len() - 1);
    }
    s
}
