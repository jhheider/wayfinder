use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Margin, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};

use super::app::{App, Modal, Screen};
use super::modal::{boosts, domain, info, runes, selection, skills, text_input};
use super::panels::{build_selector, character_info, detail_tabs};
use super::screens::character_select;

pub fn draw(f: &mut Frame, app: &App) {
    match app.screen {
        Screen::CharacterSelect => {
            character_select::render(f, app);
            return;
        }
        Screen::Builder => {}
    }

    let outer = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
        .split(f.area());

    let right = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
        .split(outer[1]);

    build_selector::render(f, outer[0], app);
    character_info::render(f, right[0], app);
    detail_tabs::render(f, right[1], app);

    // Status bar (bottom of build panel)
    if let Some(status) = &app.status {
        let display = if status.starts_with("Loading") {
            format!("{} {status}", app.spinner())
        } else {
            status.clone()
        };
        let status_area = Rect {
            x: outer[0].x,
            y: outer[0].y + outer[0].height.saturating_sub(1),
            width: outer[0].width,
            height: 1,
        };
        f.render_widget(
            Paragraph::new(display).style(Style::default().fg(Color::DarkGray)),
            status_area,
        );
    }

    // Modal overlay
    match &app.modal {
        Some(Modal::Selection(modal)) => selection::render(f, modal),
        Some(Modal::Boosts(modal)) => boosts::render(f, modal),
        Some(Modal::Skill(modal)) => skills::render(f, modal),
        Some(Modal::TextInput(modal)) => text_input::render(f, modal),
        Some(Modal::Info(modal)) => info::render(f, modal),
        Some(Modal::Runes(modal)) => runes::render(f, modal),
        Some(Modal::Domain(modal)) => domain::render(f, modal),
        None => {}
    }

    // Help overlay
    if app.show_help {
        render_help(f);
    }
}

fn render_help(f: &mut Frame) {
    let area = centered_rect(50, 60, f.area());
    f.render_widget(Clear, area);

    let block = Block::default()
        .title(" Keybindings ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));
    f.render_widget(block.clone(), area);
    let inner = area.inner(Margin::new(1, 1));

    let key_style = Style::default()
        .fg(Color::Cyan)
        .add_modifier(Modifier::BOLD);
    let desc_style = Style::default().fg(Color::Gray);
    let header_style = Style::default()
        .fg(Color::White)
        .add_modifier(Modifier::BOLD);

    let lines = vec![
        Line::from(Span::styled("Navigation", header_style)),
        help_line("Tab", "Cycle panels", key_style, desc_style),
        help_line("↑/↓", "Navigate build slots", key_style, desc_style),
        help_line("←/→", "Switch detail tabs", key_style, desc_style),
        help_line("Enter", "Open selection modal", key_style, desc_style),
        help_line("Backspace", "Clear slot selection", key_style, desc_style),
        help_line("+/-", "Level up/down", key_style, desc_style),
        Line::from(""),
        Line::from(Span::styled("Selection Modal", header_style)),
        help_line("Type", "Filter by name", key_style, desc_style),
        help_line("↑/↓", "Navigate list", key_style, desc_style),
        help_line("PgUp/Dn", "Scroll detail", key_style, desc_style),
        help_line("Enter", "Confirm selection", key_style, desc_style),
        help_line("Esc", "Cancel", key_style, desc_style),
        Line::from(""),
        Line::from(Span::styled("Ability Boosts", header_style)),
        help_line("←/→/Tab", "Switch groups", key_style, desc_style),
        help_line("↑/↓", "Navigate abilities", key_style, desc_style),
        help_line("Space", "Toggle boost", key_style, desc_style),
        help_line("a", "Toggle alternate ancestry", key_style, desc_style),
        help_line("Enter", "Confirm (when complete)", key_style, desc_style),
        Line::from(""),
        Line::from(Span::styled("General", header_style)),
        help_line("n", "Rename character", key_style, desc_style),
        help_line("Ctrl+S", "Save character", key_style, desc_style),
        help_line("Ctrl+E", "Export (Pathbuilder)", key_style, desc_style),
        help_line("Esc", "Character select", key_style, desc_style),
        help_line("q", "Quit", key_style, desc_style),
        help_line("?", "This help", key_style, desc_style),
    ];

    let paragraph = Paragraph::new(lines);
    f.render_widget(paragraph, inner);
}

fn help_line<'a>(key: &'a str, desc: &'a str, key_style: Style, desc_style: Style) -> Line<'a> {
    Line::from(vec![
        Span::styled(format!("  {key:<12}"), key_style),
        Span::styled(desc, desc_style),
    ])
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
