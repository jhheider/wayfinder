//! Fuzzy category suggestion.

use super::ALL_CATEGORIES;

/// Find the closest matching category for a given input.
/// Returns `None` if no reasonable match exists.
pub fn suggest_category(input: &str) -> Option<&'static str> {
    suggest_from(input, ALL_CATEGORIES).copied()
}

/// Find the closest match for `input` among `candidates` (exact, then a unique
/// prefix, then a unique substring, then edit distance ≤ 3).
pub fn suggest_from<'a, S: AsRef<str>>(input: &str, candidates: &'a [S]) -> Option<&'a S> {
    let input = input.to_lowercase();

    // Exact match
    if let Some(cat) = candidates.iter().find(|c| c.as_ref() == input) {
        return Some(cat);
    }

    // Prefix match
    let prefix_matches: Vec<&S> = candidates
        .iter()
        .filter(|c| c.as_ref().starts_with(&input))
        .collect();
    if prefix_matches.len() == 1 {
        return Some(prefix_matches[0]);
    }

    // Substring match
    let substr_matches: Vec<&S> = candidates
        .iter()
        .filter(|c| c.as_ref().contains(input.as_str()))
        .collect();
    if substr_matches.len() == 1 {
        return Some(substr_matches[0]);
    }

    // Edit distance (simple Levenshtein)
    let mut best = None;
    let mut best_dist = usize::MAX;
    for cat in candidates {
        let d = edit_distance(&input, cat.as_ref());
        if d < best_dist {
            best_dist = d;
            best = Some(cat);
        }
    }
    // Only suggest if distance is reasonable (≤ 3 edits)
    if best_dist <= 3 {
        return best;
    }

    None
}

fn edit_distance(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let mut dp = vec![vec![0usize; b.len() + 1]; a.len() + 1];
    for (i, row) in dp.iter_mut().enumerate().take(a.len() + 1) {
        row[0] = i;
    }
    for (j, val) in dp[0].iter_mut().enumerate().take(b.len() + 1) {
        *val = j;
    }
    for i in 1..=a.len() {
        for j in 1..=b.len() {
            let cost = if a[i - 1] == b[j - 1] { 0 } else { 1 };
            dp[i][j] = (dp[i - 1][j] + 1)
                .min(dp[i][j - 1] + 1)
                .min(dp[i - 1][j - 1] + cost);
        }
    }
    dp[a.len()][b.len()]
}
