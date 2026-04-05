//! Rune assignment modal for weapons and armor.

use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};

use crate::model::equipment::{ResilientRune, StrikingRune};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum RuneTarget {
    Weapon(usize),
    Armor(usize),
    Shield,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuneField {
    Potency,
    Striking,
    Resilient,
}

pub struct RuneModal {
    pub target: RuneTarget,
    pub title: String,
    pub potency: u8,
    pub striking: StrikingRune,
    pub resilient: ResilientRune,
    pub cursor: usize,
    pub fields: Vec<RuneField>,
}

impl RuneModal {
    pub fn for_weapon(idx: usize, name: &str, potency: u8, striking: StrikingRune) -> Self {
        Self {
            target: RuneTarget::Weapon(idx),
            title: format!("Runes — {name}"),
            potency,
            striking,
            resilient: ResilientRune::None,
            cursor: 0,
            fields: vec![RuneField::Potency, RuneField::Striking],
        }
    }

    pub fn for_armor(idx: usize, name: &str, potency: u8, resilient: ResilientRune) -> Self {
        Self {
            target: RuneTarget::Armor(idx),
            title: format!("Runes — {name}"),
            potency,
            striking: StrikingRune::None,
            resilient,
            cursor: 0,
            fields: vec![RuneField::Potency, RuneField::Resilient],
        }
    }

    pub fn for_shield(name: &str, potency: u8, resilient: ResilientRune) -> Self {
        Self {
            target: RuneTarget::Shield,
            title: format!("Runes — {name}"),
            potency,
            striking: StrikingRune::None,
            resilient,
            cursor: 0,
            fields: vec![RuneField::Potency, RuneField::Resilient],
        }
    }

    pub fn move_up(&mut self) {
        self.cursor = self.cursor.saturating_sub(1);
    }

    pub fn move_down(&mut self) {
        if self.cursor + 1 < self.fields.len() {
            self.cursor += 1;
        }
    }

    pub fn cycle_right(&mut self) {
        match self.fields[self.cursor] {
            RuneField::Potency => {
                self.potency = if self.potency >= 3 {
                    0
                } else {
                    self.potency + 1
                };
            }
            RuneField::Striking => {
                self.striking = self.striking.next();
            }
            RuneField::Resilient => {
                self.resilient = self.resilient.next();
            }
        }
    }

    pub fn cycle_left(&mut self) {
        match self.fields[self.cursor] {
            RuneField::Potency => {
                self.potency = if self.potency == 0 {
                    3
                } else {
                    self.potency - 1
                };
            }
            RuneField::Striking => {
                self.striking = self.striking.prev();
            }
            RuneField::Resilient => {
                self.resilient = self.resilient.prev();
            }
        }
    }

    pub fn field_label(&self, idx: usize) -> &'static str {
        match self.fields[idx] {
            RuneField::Potency => "Potency",
            RuneField::Striking => "Striking",
            RuneField::Resilient => "Resilient",
        }
    }

    pub fn field_value(&self, idx: usize) -> String {
        match self.fields[idx] {
            RuneField::Potency => {
                if self.potency == 0 {
                    "None".to_string()
                } else {
                    format!("+{}", self.potency)
                }
            }
            RuneField::Striking => {
                let l = self.striking.label();
                if l.is_empty() {
                    "None".to_string()
                } else {
                    l.to_string()
                }
            }
            RuneField::Resilient => {
                let l = self.resilient.label();
                if l.is_empty() {
                    "None".to_string()
                } else {
                    l.to_string()
                }
            }
        }
    }
}

pub fn render(f: &mut Frame, modal: &RuneModal) {
    let area = centered_rect(40, 30, f.area());
    f.render_widget(Clear, area);

    let block = Block::default()
        .title(format!(" {} ", modal.title))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let mut lines: Vec<Line> = Vec::new();
    lines.push(Line::from("  ←/→ to change, Enter to confirm"));
    lines.push(Line::from(""));

    for (i, _field) in modal.fields.iter().enumerate() {
        let is_selected = i == modal.cursor;
        let label = modal.field_label(i);
        let value = modal.field_value(i);
        let style = if is_selected {
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };
        let marker = if is_selected { ">" } else { " " };
        lines.push(Line::from(vec![
            Span::styled(format!(" {marker} {label:<12}"), style),
            Span::styled(format!("◂ {value} ▸"), style),
        ]));
    }

    f.render_widget(Paragraph::new(lines), inner);
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
