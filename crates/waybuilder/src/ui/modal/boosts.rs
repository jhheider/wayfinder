use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Margin, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};

use crate::model::abilities::{Ability, BoostSpec};

/// A single boost group the user must assign (ancestry, background, class, free).
#[derive(Debug, Clone)]
pub struct BoostGroup {
    pub label: String,
    pub source: String,
    /// Fixed boosts auto-applied (not chooseable).
    pub fixed: Vec<Ability>,
    /// Number of free picks remaining.
    pub free_count: u8,
    /// User's free picks so far.
    pub chosen: Vec<Ability>,
    /// Flaws from this source.
    pub flaws: Vec<Ability>,
    /// Whether this group supports the alternative ancestry rule
    /// (replace fixed+flaws with 2 free boosts).
    pub can_use_alternate: bool,
    /// Whether the alternate mode is active.
    pub alternate_active: bool,
    /// Stashed original values when toggling alternate mode.
    alternate_fixed: Vec<Ability>,
    alternate_free: u8,
    alternate_flaws: Vec<Ability>,
}

pub struct BoostModal {
    pub groups: Vec<BoostGroup>,
    /// Which group is focused.
    pub group_cursor: usize,
    /// Which ability is highlighted within the focused group.
    pub ability_cursor: usize,
}

impl BoostModal {
    pub fn new(
        ancestry_boosts: &BoostSpec,
        background_boosts: &BoostSpec,
        class_boosts: &BoostSpec,
        level: u8,
    ) -> Self {
        let mut groups = Vec::new();

        if !ancestry_boosts.fixed.is_empty() || ancestry_boosts.free > 0 {
            groups.push(BoostGroup {
                label: "Ancestry".into(),
                source: "ancestry".into(),
                fixed: ancestry_boosts.fixed.clone(),
                free_count: ancestry_boosts.free,
                chosen: Vec::new(),
                flaws: ancestry_boosts.flaws.clone(),
                can_use_alternate: !ancestry_boosts.fixed.is_empty(),
                alternate_active: false,
                alternate_fixed: ancestry_boosts.fixed.clone(),
                alternate_free: ancestry_boosts.free,
                alternate_flaws: ancestry_boosts.flaws.clone(),
            });
        }

        if !background_boosts.fixed.is_empty() || background_boosts.free > 0 {
            groups.push(new_group("Background", "background", background_boosts));
        }

        if !class_boosts.fixed.is_empty() || class_boosts.free > 0 {
            groups.push(new_group("Class", "class", class_boosts));
        }

        // Level 1 always gets 4 free boosts
        let (label, source) = if level == 1 {
            ("Free".to_string(), "level_1".to_string())
        } else {
            (format!("Level {level}"), format!("level_{level}"))
        };
        groups.push(new_group_free(&label, &source, 4));

        BoostModal {
            groups,
            group_cursor: 0,
            ability_cursor: 0,
        }
    }

    pub fn move_up(&mut self) {
        self.ability_cursor = self.ability_cursor.saturating_sub(1);
    }

    pub fn move_down(&mut self) {
        if self.ability_cursor < 5 {
            self.ability_cursor += 1;
        }
    }

    pub fn next_group(&mut self) {
        if self.group_cursor + 1 < self.groups.len() {
            self.group_cursor += 1;
            self.ability_cursor = 0;
        }
    }

    pub fn prev_group(&mut self) {
        if self.group_cursor > 0 {
            self.group_cursor -= 1;
            self.ability_cursor = 0;
        }
    }

    /// Toggle the alternate ancestry rule on the focused group (if applicable).
    pub fn toggle_alternate(&mut self) {
        let Some(group) = self.groups.get_mut(self.group_cursor) else {
            return;
        };
        if !group.can_use_alternate {
            return;
        }
        group.chosen.clear();
        if group.alternate_active {
            // Restore original fixed/free/flaws
            group.fixed = group.alternate_fixed.clone();
            group.free_count = group.alternate_free;
            group.flaws = group.alternate_flaws.clone();
            group.alternate_active = false;
        } else {
            // PF2e alternate ancestry boosts: 2 free, no fixed, no flaws
            group.fixed.clear();
            group.free_count = 2;
            group.flaws.clear();
            group.alternate_active = true;
        }
    }

    /// Toggle the currently highlighted ability in the focused group.
    pub fn toggle(&mut self) {
        let ability = Ability::ALL[self.ability_cursor];
        let Some(group) = self.groups.get_mut(self.group_cursor) else {
            return;
        };
        // Can't toggle fixed boosts
        if group.fixed.contains(&ability) {
            return;
        }
        if let Some(pos) = group.chosen.iter().position(|&a| a == ability) {
            group.chosen.remove(pos);
        } else if group.chosen.len() < group.free_count as usize {
            group.chosen.push(ability);
        }
    }

    /// Check if all groups are fully assigned.
    pub fn is_complete(&self) -> bool {
        self.groups
            .iter()
            .all(|g| g.chosen.len() == g.free_count as usize)
    }
}

fn new_group(label: &str, source: &str, spec: &BoostSpec) -> BoostGroup {
    BoostGroup {
        label: label.into(),
        source: source.into(),
        fixed: spec.fixed.clone(),
        free_count: spec.free,
        chosen: Vec::new(),
        flaws: spec.flaws.clone(),
        can_use_alternate: false,
        alternate_active: false,
        alternate_fixed: Vec::new(),
        alternate_free: 0,
        alternate_flaws: Vec::new(),
    }
}

fn new_group_free(label: &str, source: &str, free: u8) -> BoostGroup {
    BoostGroup {
        label: label.into(),
        source: source.into(),
        fixed: Vec::new(),
        free_count: free,
        chosen: Vec::new(),
        flaws: Vec::new(),
        can_use_alternate: false,
        alternate_active: false,
        alternate_fixed: Vec::new(),
        alternate_free: 0,
        alternate_flaws: Vec::new(),
    }
}

pub fn render(f: &mut Frame, modal: &BoostModal) {
    let area = centered_rect(70, 60, f.area());
    f.render_widget(Clear, area);

    let outer = Block::default()
        .title(" Ability Boosts ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));
    f.render_widget(outer.clone(), area);
    let inner = area.inner(Margin::new(1, 1));

    // Split into group columns
    let constraints: Vec<Constraint> = modal
        .groups
        .iter()
        .map(|_| Constraint::Ratio(1, modal.groups.len() as u32))
        .collect();
    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(constraints)
        .split(inner);

    for (gi, group) in modal.groups.iter().enumerate() {
        let focused = gi == modal.group_cursor;
        render_group(f, columns[gi], group, focused, modal.ability_cursor);
    }

    // Running totals row at the bottom of the modal
    render_totals(f, inner, modal);
}

fn render_totals(f: &mut Frame, area: Rect, modal: &BoostModal) {
    use crate::model::abilities::Ability;
    // Count net boosts per ability across all groups
    let mut net = [0i32; 6];
    for group in &modal.groups {
        for (i, &ability) in Ability::ALL.iter().enumerate() {
            if group.fixed.contains(&ability) {
                net[i] += 1;
            }
            if group.chosen.contains(&ability) {
                net[i] += 1;
            }
            if group.flaws.contains(&ability) {
                net[i] -= 1;
            }
        }
    }
    let mut spans: Vec<Span> = vec![Span::styled(
        " Totals: ",
        Style::default()
            .fg(Color::White)
            .add_modifier(Modifier::BOLD),
    )];
    for (i, &ability) in Ability::ALL.iter().enumerate() {
        let modifier = net[i]; // Each boost = +1 modifier (from +2 score / 2)
        let sign = if modifier >= 0 { "+" } else { "" };
        let color = if modifier > 0 {
            Color::Cyan
        } else if modifier < 0 {
            Color::Red
        } else {
            Color::DarkGray
        };
        spans.push(Span::styled(
            format!("{} {sign}{modifier}  ", ability.abbr()),
            Style::default().fg(color),
        ));
    }
    let totals_area = Rect {
        x: area.x,
        y: area.y + area.height.saturating_sub(1),
        width: area.width,
        height: 1,
    };
    f.render_widget(Paragraph::new(Line::from(spans)), totals_area);
}

fn render_group(
    f: &mut Frame,
    area: Rect,
    group: &BoostGroup,
    focused: bool,
    ability_cursor: usize,
) {
    let border_color = if focused {
        Color::Cyan
    } else {
        Color::DarkGray
    };
    let remaining = group.free_count as usize - group.chosen.len();
    let title = format!(
        " {} ({}/{}) ",
        group.label,
        group.chosen.len(),
        group.free_count
    );
    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color));

    let mut lines: Vec<Line> = Vec::new();

    for (i, &ability) in Ability::ALL.iter().enumerate() {
        let is_fixed = group.fixed.contains(&ability);
        let is_chosen = group.chosen.contains(&ability);
        let is_flaw = group.flaws.contains(&ability);
        let is_cursor = focused && i == ability_cursor;

        let marker = if is_fixed {
            "●"
        } else if is_chosen {
            "◉"
        } else {
            "○"
        };

        let flaw_marker = if is_flaw { " ▼" } else { "" };

        let style = if is_cursor {
            Style::default().fg(Color::Black).bg(Color::Cyan)
        } else if is_fixed {
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD)
        } else if is_chosen {
            Style::default().fg(Color::Cyan)
        } else if is_flaw {
            Style::default().fg(Color::Red)
        } else if remaining == 0 && !is_chosen {
            Style::default().fg(Color::DarkGray)
        } else {
            Style::default()
        };

        let text = format!(" {marker} {}{flaw_marker}", ability.abbr());
        lines.push(Line::from(Span::styled(text, style)));
    }

    lines.push(Line::from(""));
    if !group.fixed.is_empty() {
        lines.push(Line::from(Span::styled(
            " ● = fixed",
            Style::default().fg(Color::DarkGray),
        )));
    }
    if !group.flaws.is_empty() {
        lines.push(Line::from(Span::styled(
            " ▼ = flaw",
            Style::default().fg(Color::DarkGray),
        )));
    }
    if group.can_use_alternate {
        let hint = if group.alternate_active {
            " [a] use standard"
        } else {
            " [a] use alternate"
        };
        lines.push(Line::from(Span::styled(
            hint,
            Style::default().fg(Color::Yellow),
        )));
    }

    let paragraph = Paragraph::new(lines).block(block);
    f.render_widget(paragraph, area);
}

fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(vertical[1])[1]
}
