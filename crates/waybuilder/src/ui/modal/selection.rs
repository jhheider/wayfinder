use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Margin, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{
    Block, Borders, Clear, List, ListItem, ListState, Paragraph, Scrollbar, ScrollbarOrientation,
    ScrollbarState, Wrap,
};
use wayfinder_core::aon::Document;
use wayfinder_core::render::content::ContentBlock;
use wayfinder_core::render::{parse_content, render_spans};

use crate::ui::convert::spans_to_text;

pub struct SelectionModal {
    pub title: String,
    pub items: Vec<Document>,
    pub cursor: usize,
    pub filter: String,
    pub detail_scroll: u16,
    /// Client-side max level filter (feats only show if level <= this).
    pub max_level: Option<i32>,
    /// Client-side ancestry filter for heritages (match name suffix).
    pub ancestry_filter: Option<String>,
}

impl SelectionModal {
    pub fn new(title: &str) -> Self {
        Self {
            title: title.to_string(),
            items: Vec::new(),
            cursor: 0,
            filter: String::new(),
            detail_scroll: 0,
            max_level: None,
            ancestry_filter: None,
        }
    }

    pub fn filtered_items(&self) -> Vec<&Document> {
        let lower = self.filter.to_lowercase();
        let mut items: Vec<&Document> = self
            .items
            .iter()
            .filter(|d| {
                // Name filter
                if !self.filter.is_empty() {
                    let name_match = d
                        .name
                        .as_deref()
                        .is_some_and(|n| n.to_lowercase().contains(&lower));
                    if !name_match {
                        return false;
                    }
                }
                // Max level filter
                if let Some(max) = self.max_level
                    && let Some(lvl) = d.level
                    && lvl > max
                {
                    return false;
                }
                // Ancestry filter for heritages: show ancestry-specific + versatile
                if let Some(ancestry) = &self.ancestry_filter {
                    let name = d.name.as_deref().unwrap_or("");
                    let name_lower = name.to_lowercase();
                    let anc_lower = ancestry.to_lowercase();
                    let traits_lower: Vec<String> =
                        d.traits.iter().map(|t| t.to_lowercase()).collect();
                    // Match if name/traits reference the ancestry
                    let ancestry_match = name_lower.contains(&anc_lower)
                        || traits_lower.iter().any(|t| t == &anc_lower);
                    // Versatile heritages have no ancestry-specific trait
                    let is_versatile = traits_lower
                        .iter()
                        .any(|t| t == "versatile heritage");
                    if !ancestry_match && !is_versatile {
                        return false;
                    }
                }
                true
            })
            .collect();
        items.sort_by(|a, b| {
            let a_name = a.name.as_deref().unwrap_or("");
            let b_name = b.name.as_deref().unwrap_or("");
            a_name.to_lowercase().cmp(&b_name.to_lowercase())
        });
        items
    }

    pub fn selected(&self) -> Option<&Document> {
        let filtered = self.filtered_items();
        filtered.get(self.cursor).copied()
    }

    pub fn move_up(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
            self.detail_scroll = 0;
        }
    }

    pub fn move_down(&mut self) {
        let max = self.filtered_items().len().saturating_sub(1);
        if self.cursor < max {
            self.cursor += 1;
            self.detail_scroll = 0;
        }
    }

    pub fn type_char(&mut self, c: char) {
        self.filter.push(c);
        self.cursor = 0;
        self.detail_scroll = 0;
    }

    pub fn backspace(&mut self) {
        self.filter.pop();
        self.cursor = 0;
        self.detail_scroll = 0;
    }

    pub fn scroll_detail_down(&mut self) {
        self.detail_scroll = self.detail_scroll.saturating_add(1);
    }

    pub fn scroll_detail_up(&mut self) {
        self.detail_scroll = self.detail_scroll.saturating_sub(1);
    }
}

pub fn render(f: &mut Frame, modal: &SelectionModal) {
    let area = centered_rect(80, 80, f.area());
    f.render_widget(Clear, area);

    let outer = Block::default()
        .title(format!(" {} ", modal.title))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));
    f.render_widget(outer.clone(), area);
    let inner = area.inner(Margin::new(1, 1));

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(0)])
        .split(inner);

    // Filter bar
    let filter_line = Line::from(vec![
        Span::styled("/ ", Style::default().fg(Color::Cyan)),
        Span::raw(&modal.filter),
        Span::styled("_", Style::default().fg(Color::DarkGray)),
    ]);
    f.render_widget(Paragraph::new(filter_line), layout[0]);

    // Two-panel: list | detail
    let panels = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(layout[1]);

    render_list(f, panels[0], modal);
    render_detail(f, panels[1], modal);
}

fn rarity_style(rarity: &str) -> Option<Style> {
    match rarity.to_lowercase().as_str() {
        "uncommon" => Some(Style::default().fg(Color::Rgb(255, 140, 0))),
        "rare" => Some(Style::default().fg(Color::Blue)),
        "unique" => Some(Style::default().fg(Color::Magenta)),
        _ => None,
    }
}

fn rarity_badge(rarity: Option<&str>) -> Option<(Style, &'static str)> {
    let r = rarity?;
    let style = rarity_style(r)?;
    let abbr = match r.to_lowercase().as_str() {
        "uncommon" => "U",
        "rare" => "R",
        "unique" => "!",
        _ => return None,
    };
    Some((style, abbr))
}

fn render_list(f: &mut Frame, area: Rect, modal: &SelectionModal) {
    let filtered = modal.filtered_items();
    let inner_w = area.width.saturating_sub(2) as usize;

    let items: Vec<ListItem> = filtered
        .iter()
        .enumerate()
        .map(|(i, doc)| {
            let selected = i == modal.cursor;
            let name = doc.name.as_deref().unwrap_or("???");
            let is_legacy = !doc.remaster_id.is_empty();

            let mut right_spans: Vec<Span> = Vec::new();
            if is_legacy {
                right_spans.push(Span::styled(
                    " legacy ",
                    Style::default().fg(Color::DarkGray),
                ));
            }
            if let Some((style, abbr)) = rarity_badge(doc.rarity.as_deref()) {
                right_spans.push(Span::styled(format!(" {abbr} "), style));
            }
            if let Some(level) = doc.level {
                right_spans.push(Span::styled(
                    format!(" {level:>2}"),
                    Style::default().fg(Color::DarkGray),
                ));
            }

            let right_w: usize = right_spans.iter().map(|s| s.width()).sum();
            let name_max = inner_w.saturating_sub(right_w);
            let display_name: String = if name.len() > name_max {
                format!("{}…", &name[..name_max.saturating_sub(1)])
            } else {
                name.to_string()
            };
            let pad = inner_w.saturating_sub(display_name.len() + right_w);

            let name_style = if selected {
                Style::default().fg(Color::Black).bg(Color::Cyan)
            } else if is_legacy {
                Style::default().fg(Color::DarkGray)
            } else {
                Style::default()
            };

            let mut spans = vec![
                Span::styled(display_name, name_style),
                Span::raw(" ".repeat(pad)),
            ];
            spans.extend(right_spans);
            ListItem::new(Line::from(spans))
        })
        .collect();

    let block = Block::default()
        .title(" Options ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    let list_len = items.len();
    let list = List::new(items).block(block).highlight_symbol("");
    let mut list_state = ListState::default().with_selected(Some(modal.cursor));
    f.render_stateful_widget(list, area, &mut list_state);

    if list_len > area.height.saturating_sub(2) as usize {
        let mut sb_state = ScrollbarState::new(list_len).position(modal.cursor);
        f.render_stateful_widget(
            Scrollbar::new(ScrollbarOrientation::VerticalRight),
            area.inner(Margin::new(0, 1)),
            &mut sb_state,
        );
    }
}

fn render_detail(f: &mut Frame, area: Rect, modal: &SelectionModal) {
    let block = Block::default()
        .title(" Detail ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    let Some(doc) = modal.selected() else {
        let empty = Paragraph::new("No selection").block(block);
        f.render_widget(empty, area);
        return;
    };

    let mut header_lines: Vec<Line> = Vec::new();

    // Name + level
    let name = doc.name.as_deref().unwrap_or("???");
    let mut title_spans = vec![Span::styled(
        name,
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    )];
    if let Some(level) = doc.level {
        title_spans.push(Span::styled(
            format!("  Level {level}"),
            Style::default().fg(Color::DarkGray),
        ));
    }
    header_lines.push(Line::from(title_spans));

    // Rarity + traits
    let mut tag_spans: Vec<Span> = Vec::new();
    if let Some((style, _)) = rarity_badge(doc.rarity.as_deref()) {
        let rarity = doc.rarity.as_deref().unwrap();
        tag_spans.push(Span::styled(
            format!(" {rarity} "),
            style.bg(Color::DarkGray),
        ));
        tag_spans.push(Span::raw(" "));
    }
    for t in &doc.traits {
        tag_spans.push(Span::styled(
            format!(" {t} "),
            Style::default().fg(Color::Yellow).bg(Color::DarkGray),
        ));
        tag_spans.push(Span::raw(" "));
    }
    if !tag_spans.is_empty() {
        header_lines.push(Line::from(tag_spans));
    }

    // Source
    if !doc.source.is_empty() {
        header_lines.push(Line::from(Span::styled(
            doc.source.join(", "),
            Style::default().fg(Color::DarkGray),
        )));
    }

    header_lines.push(Line::from(""));

    // Rendered markdown content — skip leading title + traits (already in header)
    let content_text = if let Some(md) = doc.markdown.as_deref() {
        let base = "https://2e.aonprd.com";
        let blocks = parse_content(md, base);
        let trimmed = skip_leading_header(&blocks);
        let spans = render_spans(trimmed);
        spans_to_text(&spans)
    } else {
        ratatui::text::Text::raw(doc.text.as_deref().unwrap_or(""))
    };

    let mut all_lines = header_lines;
    all_lines.extend(content_text.lines);

    let paragraph = Paragraph::new(all_lines)
        .block(block)
        .wrap(Wrap { trim: false })
        .scroll((modal.detail_scroll, 0));

    f.render_widget(paragraph, area);
}

/// Skip the leading title (level 1), any trait paragraphs, and
/// the first source KeyValue — these are already shown in the header.
fn skip_leading_header(blocks: &[ContentBlock]) -> &[ContentBlock] {
    let mut i = 0;
    // Skip leading level-1 title
    if let Some(ContentBlock::Title { level: 1, .. }) = blocks.first() {
        i += 1;
    }
    // Skip trait/source paragraphs that follow
    while i < blocks.len() {
        match &blocks[i] {
            // Paragraphs with just traits/actions/links (the trait line + source line)
            ContentBlock::Paragraph { content } => {
                let all_meta = content.iter().all(|ic| {
                    matches!(
                        ic,
                        wayfinder_core::render::content::InlineContent::Trait { .. }
                            | wayfinder_core::render::content::InlineContent::Action { .. }
                    )
                });
                if all_meta {
                    i += 1;
                } else {
                    break;
                }
            }
            // Source line rendered as KeyValue with bold "Source"
            ContentBlock::KeyValue { key, .. } if key == "Source" => {
                i += 1;
            }
            ContentBlock::HRule => {
                i += 1;
                break; // content starts after the hrule
            }
            _ => break,
        }
    }
    &blocks[i..]
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
