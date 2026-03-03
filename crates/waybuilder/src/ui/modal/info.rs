use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};
use wayfinder_core::aon::Document;
use wayfinder_core::render::{parse_content, render_spans};

use crate::ui::convert::spans_to_text;

pub struct InfoModal {
    pub title: String,
    pub doc: Option<Document>,
    pub scroll: u16,
}

impl InfoModal {
    pub fn new(title: &str) -> Self {
        Self {
            title: title.to_string(),
            doc: None,
            scroll: 0,
        }
    }

    pub fn scroll_down(&mut self) {
        self.scroll = self.scroll.saturating_add(1);
    }

    pub fn scroll_up(&mut self) {
        self.scroll = self.scroll.saturating_sub(1);
    }
}

pub fn render(f: &mut Frame, modal: &InfoModal) {
    let area = centered_rect(70, 70, f.area());
    f.render_widget(Clear, area);

    let block = Block::default()
        .title(format!(" {} ", modal.title))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let Some(doc) = &modal.doc else {
        let loading = Paragraph::new("Loading...").block(block);
        f.render_widget(loading, area);
        return;
    };

    let name = doc.name.as_deref().unwrap_or("???");
    let mut lines: Vec<Line> = Vec::new();

    lines.push(Line::from(Span::styled(
        name,
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(""));

    if let Some(md) = doc.markdown.as_deref() {
        let base = "https://2e.aonprd.com";
        let blocks = parse_content(md, base);
        let spans = render_spans(&blocks);
        let text = spans_to_text(&spans);
        lines.extend(text.lines);
    } else if let Some(text) = &doc.text {
        lines.push(Line::from(text.as_str()));
    }

    let paragraph = Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: false })
        .scroll((modal.scroll, 0));

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
