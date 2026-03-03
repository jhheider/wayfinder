//! Colored terminal rendering of spans and documents.

use colored::Colorize;

use super::terminal::{Span, action_icon};
use crate::aon::Document;
use crate::aon::categories::category_icon;
use crate::search::is_legacy;

/// Render spans to colored terminal output with enhanced styling.
pub fn render_spans_colored(spans: &[Span]) -> String {
    let mut out = String::new();
    for span in spans {
        match span {
            Span::Title { level, text } => {
                let styled = match level {
                    1 => text.bold().underline().bright_white().to_string(),
                    2 => text.bold().cyan().to_string(),
                    _ => text.bold().yellow().to_string(),
                };
                out.push_str(&format!("\n{styled}"));
            }
            Span::Bold(t) => out.push_str(&t.bold().green().to_string()),
            Span::Italic(t) => out.push_str(&t.italic().to_string()),
            Span::Text(t) => out.push_str(t),
            Span::Trait(t) => {
                out.push_str(&format!(" {}", t.on_yellow().black()));
            }
            Span::Link { text, .. } => {
                out.push_str(&text.cyan().underline().to_string());
            }
            Span::Action(t) => {
                let icon = action_icon(t);
                out.push_str(&format!(" {icon}"));
            }
            Span::HRule => {
                out.push_str(&"───────────────────────────────────\n".dimmed().to_string());
            }
            Span::Newline => out.push('\n'),
            Span::ListItem(inner) => {
                out.push_str(&format!("  {} ", "•".dimmed()));
                out.push_str(&render_spans_colored(inner));
                out.push('\n');
            }
            Span::TableHeader(cells) => {
                let formatted: Vec<String> =
                    cells.iter().map(|c| format!("{:>12}", c.bold())).collect();
                out.push_str(&format!("  {}\n", formatted.join("  ")));
            }
            Span::TableRow(cells) => {
                let formatted: Vec<String> = cells.iter().map(|c| format!("{c:>12}")).collect();
                out.push_str(&format!("  {}\n", formatted.join("  ")));
            }
            Span::AsideTitle(t) => {
                out.push_str(&format!("  {}", t.bold().magenta()));
            }
        }
    }
    out
}

/// Format a rarity string with color.
pub fn rarity_colored(rarity: &str) -> String {
    match rarity.to_lowercase().as_str() {
        "common" => rarity.dimmed().to_string(),
        "uncommon" => rarity.yellow().to_string(),
        "rare" => rarity.blue().bold().to_string(),
        "unique" => rarity.magenta().bold().to_string(),
        _ => rarity.to_string(),
    }
}

/// Display a document in short list format with colors and emoji.
pub fn display_short_colored(doc: &Document) -> String {
    let name = doc.name.as_deref().unwrap_or("Unknown");
    let cat = doc.category.as_deref().unwrap_or("");
    let cat_icon = category_icon(cat);
    let rarity = doc.rarity.as_deref().unwrap_or("");

    let mut line = format!(
        "{} {} {}",
        cat_icon,
        name.bold().white(),
        format!("[{cat}]").dimmed()
    );

    if let Some(lvl) = doc.level {
        line.push_str(&format!(" {}", format!("Lvl {lvl}").cyan()));
    }

    if is_legacy(doc) {
        line.push_str(&format!(" {}", "legacy".dimmed().italic()));
    }

    if !rarity.is_empty() && rarity != "common" {
        line.push_str(&format!(" {}", rarity_colored(rarity)));
    }

    if !doc.traits.is_empty() {
        let traits: Vec<String> = doc.traits.iter().map(|t| t.yellow().to_string()).collect();
        line.push_str(&format!(" {}", traits.join(", ")));
    }

    line
}
