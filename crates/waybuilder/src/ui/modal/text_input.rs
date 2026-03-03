use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Margin, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};

/// Simple text input modal with a title and editable value.
pub struct TextInputModal {
    pub title: String,
    pub value: String,
    pub field: String,
}

impl TextInputModal {
    pub fn new(title: &str, field: &str, initial: &str) -> Self {
        Self {
            title: title.to_string(),
            value: initial.to_string(),
            field: field.to_string(),
        }
    }

    pub fn type_char(&mut self, c: char) {
        self.value.push(c);
    }

    pub fn backspace(&mut self) {
        self.value.pop();
    }
}

pub fn render(f: &mut Frame, modal: &TextInputModal) {
    let area = centered_rect(50, 20, f.area());
    f.render_widget(Clear, area);

    let outer = Block::default()
        .title(format!(" {} ", modal.title))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));
    f.render_widget(outer.clone(), area);
    let inner = area.inner(Margin::new(1, 1));

    let lines = vec![
        Line::from(Span::styled(&modal.field, Style::default().fg(Color::Gray))),
        Line::from(vec![
            Span::raw(&modal.value),
            Span::styled("_", Style::default().fg(Color::DarkGray)),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "Enter to confirm, Esc to cancel",
            Style::default().fg(Color::DarkGray),
        )),
    ];

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
