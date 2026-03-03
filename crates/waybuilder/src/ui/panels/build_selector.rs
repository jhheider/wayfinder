use ratatui::Frame;
use ratatui::layout::{Margin, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{
    Block, Borders, List, ListItem, ListState, Scrollbar, ScrollbarOrientation, ScrollbarState,
};

use crate::build::progression::ProgressionEntry;
use crate::ui::app::{App, Focus};

pub fn render(f: &mut Frame, area: Rect, app: &App) {
    let focused = app.focus == Focus::BuildSelector;
    let border_style = if focused {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };
    let title = format!(" Build — Level {} (+/-) ", app.character.level);
    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(border_style);

    let char_level = app.character.level;
    let mut current_level: u8 = 0;
    let items: Vec<ListItem> = app
        .progression
        .iter()
        .enumerate()
        .map(|(i, entry)| match entry {
            ProgressionEntry::LevelHeader(lvl) => {
                current_level = *lvl;
                let beyond = *lvl > char_level;
                let style = if beyond {
                    Style::default().fg(Color::DarkGray)
                } else {
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                };
                let separator = if *lvl > 1 {
                    format!("──────────\nLevel {lvl}")
                } else {
                    format!("Level {lvl}")
                };
                ListItem::new(Line::from(Span::styled(separator, style)))
            }
            ProgressionEntry::Slot(state) => {
                let beyond = current_level > char_level;
                let is_cursor = focused && i == app.build_cursor;
                let is_filled = state.filled.is_some();
                let is_valid = state.valid;
                let style = if is_cursor {
                    Style::default().fg(Color::Black).bg(Color::Cyan)
                } else if beyond {
                    Style::default().fg(Color::DarkGray)
                } else if is_filled && !is_valid {
                    Style::default().fg(Color::Yellow)
                } else if is_filled {
                    Style::default().fg(Color::Green)
                } else {
                    Style::default().fg(Color::DarkGray)
                };
                let marker = if is_filled && !is_valid {
                    "⚠"
                } else if is_filled {
                    "✓"
                } else {
                    "○"
                };
                ListItem::new(Line::from(Span::styled(
                    format!("  {marker} {}", state.display()),
                    style,
                )))
            }
        })
        .collect();

    let list_len = items.len();
    let list = List::new(items).block(block).highlight_symbol("");
    let mut list_state = ListState::default().with_selected(Some(app.build_cursor));
    f.render_stateful_widget(list, area, &mut list_state);

    if list_len > area.height.saturating_sub(2) as usize {
        let mut sb_state = ScrollbarState::new(list_len).position(app.build_cursor);
        f.render_stateful_widget(
            Scrollbar::new(ScrollbarOrientation::VerticalRight),
            area.inner(Margin::new(0, 1)),
            &mut sb_state,
        );
    }
}
