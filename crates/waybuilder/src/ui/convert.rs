use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span, Text};
use wayfinder_core::render::terminal::{self, action_icon};

/// Convert wayfinder-core `Span`s into ratatui `Text` for rendering.
pub fn spans_to_text(spans: &[terminal::Span]) -> Text<'static> {
    let mut lines: Vec<Line<'static>> = Vec::new();
    let mut current: Vec<Span<'static>> = Vec::new();

    for span in spans {
        match span {
            terminal::Span::Text(t) => {
                current.push(Span::raw(t.clone()));
            }
            terminal::Span::Bold(t) => {
                current.push(Span::styled(
                    t.clone(),
                    Style::default().add_modifier(Modifier::BOLD),
                ));
            }
            terminal::Span::Italic(t) => {
                current.push(Span::styled(
                    t.clone(),
                    Style::default().add_modifier(Modifier::ITALIC),
                ));
            }
            terminal::Span::Title { text, level } => {
                flush_line(&mut current, &mut lines);
                let style = match level {
                    1 => Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                    2 => Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                    _ => Style::default().add_modifier(Modifier::BOLD),
                };
                current.push(Span::styled(text.clone(), style));
            }
            terminal::Span::Trait(label) => {
                current.push(Span::styled(
                    format!(" {label} "),
                    Style::default().fg(Color::Yellow).bg(Color::DarkGray),
                ));
                current.push(Span::raw(" "));
            }
            terminal::Span::Link { text, .. } => {
                current.push(Span::styled(
                    text.clone(),
                    Style::default()
                        .fg(Color::Blue)
                        .add_modifier(Modifier::UNDERLINED),
                ));
            }
            terminal::Span::Action(a) => {
                let icon = action_icon(a);
                current.push(Span::styled(
                    format!(" {icon} "),
                    Style::default().fg(Color::Magenta),
                ));
            }
            terminal::Span::HRule => {
                flush_line(&mut current, &mut lines);
                lines.push(Line::from(Span::styled(
                    "─".repeat(40),
                    Style::default().fg(Color::DarkGray),
                )));
            }
            terminal::Span::Newline => {
                flush_line(&mut current, &mut lines);
            }
            terminal::Span::ListItem(items) => {
                flush_line(&mut current, &mut lines);
                current.push(Span::raw("  • "));
                for item in items {
                    convert_single(&mut current, item);
                }
            }
            terminal::Span::TableHeader(cells) => {
                flush_line(&mut current, &mut lines);
                let header = cells.join(" │ ");
                current.push(Span::styled(
                    header,
                    Style::default().add_modifier(Modifier::BOLD),
                ));
            }
            terminal::Span::TableRow(cells) => {
                flush_line(&mut current, &mut lines);
                current.push(Span::raw(cells.join(" │ ")));
            }
            terminal::Span::AsideTitle(t) => {
                flush_line(&mut current, &mut lines);
                current.push(Span::styled(
                    t.clone(),
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ));
            }
        }
    }
    flush_line(&mut current, &mut lines);
    Text::from(lines)
}

fn flush_line(current: &mut Vec<Span<'static>>, lines: &mut Vec<Line<'static>>) {
    if !current.is_empty() {
        lines.push(Line::from(std::mem::take(current)));
    }
}

fn convert_single(current: &mut Vec<Span<'static>>, span: &terminal::Span) {
    match span {
        terminal::Span::Text(t) => current.push(Span::raw(t.clone())),
        terminal::Span::Bold(t) => current.push(Span::styled(
            t.clone(),
            Style::default().add_modifier(Modifier::BOLD),
        )),
        terminal::Span::Italic(t) => current.push(Span::styled(
            t.clone(),
            Style::default().add_modifier(Modifier::ITALIC),
        )),
        terminal::Span::Link { text, .. } => {
            current.push(Span::styled(text.clone(), Style::default().fg(Color::Blue)))
        }
        terminal::Span::Trait(label) => {
            current.push(Span::styled(
                format!(" {label} "),
                Style::default().fg(Color::Yellow).bg(Color::DarkGray),
            ));
        }
        _ => {}
    }
}
