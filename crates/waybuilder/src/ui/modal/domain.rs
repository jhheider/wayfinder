//! Simple list picker modal for domain and divine font selection.

use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Margin, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};

/// The kind of pick this modal is performing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum PickKind {
    Domain,
    DivineFont,
}

/// A simple list picker modal.
pub struct DomainModal {
    pub title: String,
    pub kind: PickKind,
    pub items: Vec<String>,
    pub cursor: usize,
}

impl DomainModal {
    pub fn new(title: &str, kind: PickKind, items: Vec<String>) -> Self {
        Self {
            title: title.to_string(),
            kind,
            items,
            cursor: 0,
        }
    }

    pub fn move_up(&mut self) {
        self.cursor = self.cursor.saturating_sub(1);
    }

    pub fn move_down(&mut self) {
        if self.cursor + 1 < self.items.len() {
            self.cursor += 1;
        }
    }

    pub fn selected(&self) -> Option<&str> {
        self.items.get(self.cursor).map(|s| s.as_str())
    }
}

pub fn render(f: &mut Frame, modal: &DomainModal) {
    let area = centered_rect(40, 50, f.area());
    f.render_widget(Clear, area);

    let outer = Block::default()
        .title(format!(" {} ", modal.title))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));
    f.render_widget(outer.clone(), area);
    let inner = area.inner(Margin::new(1, 1));

    let mut lines: Vec<Line> = Vec::new();
    for (i, item) in modal.items.iter().enumerate() {
        let is_cursor = i == modal.cursor;
        let style = if is_cursor {
            Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };
        let marker = if is_cursor { "▸ " } else { "  " };
        lines.push(Line::from(Span::styled(
            format!("{marker}{item}"),
            style,
        )));
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
