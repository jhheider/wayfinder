use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};

use crate::ui::app::App;

pub fn render(f: &mut Frame, app: &App) {
    let area = f.area();

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(area);

    // Title
    let title = Paragraph::new(Line::from(vec![
        Span::styled(
            " Waybuilder ",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            " — PF2e Character Builder",
            Style::default().fg(Color::DarkGray),
        ),
    ]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan)),
    );
    f.render_widget(title, layout[0]);

    // Character list
    let list_block = Block::default()
        .title(" Characters ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));
    let inner = list_block.inner(layout[1]);
    f.render_widget(list_block, layout[1]);

    let mut lines: Vec<Line> = Vec::new();

    // "New Character" option at top
    let new_selected = app.select_cursor == 0;
    lines.push(Line::from(Span::styled(
        if new_selected {
            " ▸ New Character"
        } else {
            "   New Character"
        },
        if new_selected {
            Style::default().fg(Color::Black).bg(Color::Cyan)
        } else {
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD)
        },
    )));
    lines.push(Line::from(""));

    // Saved characters
    for (i, path) in app.saved_characters.iter().enumerate() {
        let selected = app.select_cursor == i + 1;
        let name = path.file_stem().and_then(|s| s.to_str()).unwrap_or("???");
        let style = if selected {
            Style::default().fg(Color::Black).bg(Color::Cyan)
        } else {
            Style::default()
        };
        let marker = if selected { " ▸ " } else { "   " };
        lines.push(Line::from(Span::styled(format!("{marker}{name}"), style)));
    }

    let content_area = Rect {
        x: inner.x + 1,
        y: inner.y,
        width: inner.width.saturating_sub(2),
        height: inner.height,
    };
    f.render_widget(Paragraph::new(lines), content_area);

    // Footer
    let footer = Paragraph::new(Line::from(vec![
        Span::styled("↑/↓ ", Style::default().fg(Color::Cyan)),
        Span::styled("Navigate  ", Style::default().fg(Color::Gray)),
        Span::styled("Enter ", Style::default().fg(Color::Cyan)),
        Span::styled("Select  ", Style::default().fg(Color::Gray)),
        Span::styled("d ", Style::default().fg(Color::Cyan)),
        Span::styled("Delete  ", Style::default().fg(Color::Gray)),
        Span::styled("q ", Style::default().fg(Color::Cyan)),
        Span::styled("Quit", Style::default().fg(Color::Gray)),
    ]));
    f.render_widget(footer, layout[2]);
}
