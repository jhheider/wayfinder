use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Margin, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};

use crate::model::proficiencies::{Rank, SKILLS};

/// Modal for selecting a skill to increase by one rank.
pub struct SkillModal {
    pub cursor: usize,
    /// (skill_name, current_rank, can_increase).
    pub skills: Vec<(String, Rank, bool)>,
    /// Lore skills at the end.
    pub lores: Vec<(String, Rank, bool)>,
}

impl SkillModal {
    /// Create a skill selection modal for initial L1 skill picks (select untrained skills).
    pub fn new_selection(
        skills: &[(String, Rank)],
        already_selected: &[String],
    ) -> Self {
        let skill_entries: Vec<(String, Rank, bool)> = SKILLS
            .iter()
            .map(|&s| {
                let rank = skills
                    .iter()
                    .find(|(n, _)| n.eq_ignore_ascii_case(s))
                    .map(|(_, r)| *r)
                    .unwrap_or(Rank::Untrained);
                let already = already_selected.iter().any(|a| a.eq_ignore_ascii_case(s));
                // Can select if untrained and not already selected
                let can = rank == Rank::Untrained && !already;
                (s.to_string(), rank, can)
            })
            .collect();
        SkillModal {
            cursor: 0,
            skills: skill_entries,
            lores: Vec::new(),
        }
    }

    pub fn new(skills: &[(String, Rank)], lores: &[(String, Rank)], max_rank: Rank) -> Self {
        let skill_entries: Vec<(String, Rank, bool)> = SKILLS
            .iter()
            .map(|&s| {
                let rank = skills
                    .iter()
                    .find(|(n, _)| n.eq_ignore_ascii_case(s))
                    .map(|(_, r)| *r)
                    .unwrap_or(Rank::Untrained);
                let can = rank >= Rank::Trained && rank < max_rank && rank.next().is_some();
                (s.to_string(), rank, can)
            })
            .collect();
        let lore_entries: Vec<(String, Rank, bool)> = lores
            .iter()
            .map(|(name, rank)| {
                let can = *rank >= Rank::Trained && *rank < max_rank && rank.next().is_some();
                (name.clone(), *rank, can)
            })
            .collect();
        SkillModal {
            cursor: 0,
            skills: skill_entries,
            lores: lore_entries,
        }
    }

    fn all_entries(&self) -> Vec<&(String, Rank, bool)> {
        self.skills.iter().chain(self.lores.iter()).collect()
    }

    pub fn move_up(&mut self) {
        self.cursor = self.cursor.saturating_sub(1);
    }

    pub fn move_down(&mut self) {
        let max = self.skills.len() + self.lores.len();
        if self.cursor + 1 < max {
            self.cursor += 1;
        }
    }

    /// Returns the selected skill name if it can be increased.
    pub fn selected(&self) -> Option<&str> {
        let entries = self.all_entries();
        entries.get(self.cursor).and_then(
            |(name, _, can)| {
                if *can { Some(name.as_str()) } else { None }
            },
        )
    }
}

pub fn render(f: &mut Frame, modal: &SkillModal) {
    let area = centered_rect(40, 70, f.area());
    f.render_widget(Clear, area);

    let outer = Block::default()
        .title(" Skill Increase ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));
    f.render_widget(outer.clone(), area);
    let inner = area.inner(Margin::new(1, 1));

    let mut lines: Vec<Line> = Vec::new();
    let entries: Vec<&(String, Rank, bool)> =
        modal.skills.iter().chain(modal.lores.iter()).collect();

    for (i, (name, rank, can)) in entries.iter().enumerate() {
        let is_cursor = i == modal.cursor;
        let rank_label = rank.label();

        let style = if is_cursor {
            Style::default().fg(Color::Black).bg(Color::Cyan)
        } else if !can {
            Style::default().fg(Color::DarkGray)
        } else {
            Style::default()
        };

        let rank_style = match rank {
            Rank::Legendary => Style::default()
                .fg(Color::Magenta)
                .add_modifier(Modifier::BOLD),
            Rank::Master => Style::default().fg(Color::Yellow),
            Rank::Expert => Style::default().fg(Color::Cyan),
            Rank::Trained => Style::default().fg(Color::Green),
            Rank::Untrained => Style::default().fg(Color::DarkGray),
        };

        let arrow = if *can { " ↑" } else { "  " };
        let line = Line::from(vec![
            Span::styled(format!(" {name:<18}"), style),
            Span::styled(
                format!("{rank_label:<10}"),
                if is_cursor { style } else { rank_style },
            ),
            Span::styled(arrow, Style::default().fg(Color::Green)),
        ]);
        lines.push(line);
    }

    let paragraph = Paragraph::new(lines);
    f.render_widget(paragraph, inner);
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
