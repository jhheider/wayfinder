use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Tabs, Wrap};

use crate::model::proficiencies::{Rank, SKILLS};
use crate::ui::app::{App, DetailTab, Focus};

pub fn render(f: &mut Frame, area: Rect, app: &App) {
    let focused = app.focus == Focus::DetailTabs;
    let border_style = if focused {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(0)])
        .split(area);

    let titles: Vec<Span> = DetailTab::ALL
        .iter()
        .map(|t| {
            let style = if *t == app.detail_tab {
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::DarkGray)
            };
            Span::styled(t.label(), style)
        })
        .collect();

    let tabs = Tabs::new(titles)
        .select(
            DetailTab::ALL
                .iter()
                .position(|t| *t == app.detail_tab)
                .unwrap_or(0),
        )
        .divider("|");
    f.render_widget(tabs, chunks[0]);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(border_style);

    let paragraph = match app.detail_tab {
        DetailTab::Skills => render_skills(app, block),
        DetailTab::Feats => render_feats(app, block),
        DetailTab::Spells => render_spells(app, block),
        DetailTab::Equipment => render_equipment(app, block),
    };
    f.render_widget(paragraph, chunks[1]);
}

fn rank_style(rank: Rank) -> Style {
    match rank {
        Rank::Legendary => Style::default()
            .fg(Color::Magenta)
            .add_modifier(Modifier::BOLD),
        Rank::Master => Style::default().fg(Color::Yellow),
        Rank::Expert => Style::default().fg(Color::Cyan),
        Rank::Trained => Style::default().fg(Color::Green),
        Rank::Untrained => Style::default().fg(Color::DarkGray),
    }
}

fn render_skills<'a>(app: &App, block: Block<'a>) -> Paragraph<'a> {
    let ch = &app.character;
    let mut lines: Vec<Line> = Vec::new();

    // Proficiency summary
    lines.push(Line::from(vec![
        Span::styled("Perception ", Style::default().fg(Color::Gray)),
        Span::styled(
            ch.proficiencies.perception.label(),
            rank_style(ch.proficiencies.perception),
        ),
    ]));
    lines.push(Line::from(""));

    // Saves
    lines.push(Line::from(Span::styled(
        "Saving Throws",
        Style::default()
            .fg(Color::White)
            .add_modifier(Modifier::BOLD),
    )));
    for (name, rank) in [
        ("Fortitude", ch.proficiencies.fortitude),
        ("Reflex", ch.proficiencies.reflex),
        ("Will", ch.proficiencies.will),
    ] {
        lines.push(Line::from(vec![
            Span::styled(format!("  {name:<12}"), Style::default().fg(Color::Gray)),
            Span::styled(rank.label(), rank_style(rank)),
        ]));
    }
    lines.push(Line::from(""));

    // Skills
    lines.push(Line::from(Span::styled(
        "Skills",
        Style::default()
            .fg(Color::White)
            .add_modifier(Modifier::BOLD),
    )));
    for &skill in SKILLS {
        let rank = ch.proficiencies.skill_rank(skill);
        if rank == Rank::Untrained {
            continue;
        }
        let modifier = ch.abilities.modifier(skill_ability(skill));
        let total = modifier + rank.bonus() + ch.level as i32;
        let sign = if total >= 0 { "+" } else { "" };
        lines.push(Line::from(vec![
            Span::styled(format!("  {skill:<16}"), Style::default().fg(Color::Gray)),
            Span::styled(rank.label(), rank_style(rank)),
            Span::styled(
                format!("  {sign}{total}"),
                Style::default().fg(Color::White),
            ),
        ]));
    }

    // Lores
    for (lore, rank) in &ch.proficiencies.lores {
        let total = ch
            .abilities
            .modifier(crate::model::abilities::Ability::Intelligence)
            + rank.bonus()
            + ch.level as i32;
        let sign = if total >= 0 { "+" } else { "" };
        lines.push(Line::from(vec![
            Span::styled(
                format!("  {lore} Lore     "),
                Style::default().fg(Color::Gray),
            ),
            Span::styled(rank.label(), rank_style(*rank)),
            Span::styled(
                format!("  {sign}{total}"),
                Style::default().fg(Color::White),
            ),
        ]));
    }

    if lines.len() <= 5 {
        lines.push(Line::from("  No trained skills."));
    }

    lines.push(Line::from(""));

    // Armor/weapon proficiencies
    lines.push(Line::from(Span::styled(
        "Combat",
        Style::default()
            .fg(Color::White)
            .add_modifier(Modifier::BOLD),
    )));
    for (name, rank) in [
        ("Unarmed", ch.proficiencies.unarmed),
        ("Simple", ch.proficiencies.simple_weapons),
        ("Martial", ch.proficiencies.martial_weapons),
        ("Unarmored", ch.proficiencies.unarmored),
        ("Light Armor", ch.proficiencies.light_armor),
        ("Medium Armor", ch.proficiencies.medium_armor),
        ("Heavy Armor", ch.proficiencies.heavy_armor),
    ] {
        if rank == Rank::Untrained {
            continue;
        }
        lines.push(Line::from(vec![
            Span::styled(format!("  {name:<14}"), Style::default().fg(Color::Gray)),
            Span::styled(rank.label(), rank_style(rank)),
        ]));
    }

    Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: false })
}

fn render_spells<'a>(app: &App, block: Block<'a>) -> Paragraph<'a> {
    let ch = &app.character;
    let Some(caster) = &ch.spell_caster else {
        return Paragraph::new("  No spellcasting.").block(block);
    };

    let tradition = caster.tradition.as_deref().unwrap_or("Unknown");
    let ability = caster
        .ability
        .as_deref()
        .and_then(|a| a.get(..3))
        .unwrap_or("???");
    let ability_upper = ability.to_uppercase();

    let mut lines: Vec<Line> = Vec::new();
    lines.push(Line::from(Span::styled(
        format!("Spells — {tradition} ({ability_upper})"),
        Style::default()
            .fg(Color::White)
            .add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(""));

    let class_data = app.choices.class_data.as_ref();
    let level = app.choices.level as usize;
    let slots = class_data.and_then(|d| d.spell_slots.get(level.saturating_sub(1)));

    let Some(slots) = slots else {
        lines.push(Line::from("  No spell slots at this level."));
        return Paragraph::new(lines)
            .block(block)
            .wrap(Wrap { trim: false });
    };

    let rank_labels = [
        "Cantrips",
        "1st Rank",
        "2nd Rank",
        "3rd Rank",
        "4th Rank",
        "5th Rank",
        "6th Rank",
        "7th Rank",
        "8th Rank",
        "9th Rank",
        "10th Rank",
    ];

    let focused = app.focus == Focus::DetailTabs && app.detail_tab == DetailTab::Spells;
    let mut flat_idx: usize = 0;

    for (rank_idx, slot_str) in slots.iter().enumerate() {
        let slot_str = slot_str.trim();
        if slot_str == "—" || slot_str == "-" || slot_str.is_empty() {
            continue;
        }
        let count: usize = slot_str.parse().unwrap_or(0);
        if count == 0 {
            continue;
        }
        let rank = rank_idx as u8;
        let label = rank_labels.get(rank_idx).unwrap_or(&"???");

        lines.push(Line::from(vec![
            Span::styled(
                format!("{label}  "),
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(format!("[{count}]"), Style::default().fg(Color::DarkGray)),
        ]));

        let prepared = app.choices.prepared_spells.get(&rank);
        for slot_i in 0..count {
            let spell_name = prepared
                .and_then(|v| v.get(slot_i))
                .map(|s| s.name.as_str());
            let is_cursor = focused && flat_idx == app.spell_cursor;
            let (marker, name_str) = match spell_name {
                Some(name) => ("✓", name.to_string()),
                None => ("○", "────".to_string()),
            };
            let style = if is_cursor {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else if spell_name.is_some() {
                Style::default().fg(Color::White)
            } else {
                Style::default().fg(Color::DarkGray)
            };
            lines.push(Line::from(vec![
                Span::styled(format!("  {marker} "), style),
                Span::styled(name_str, style),
            ]));
            flat_idx += 1;
        }
        lines.push(Line::from(""));
    }

    // Focus Spells section
    let focus_count = ch.focus_points;
    let focus_spells = &app.choices.focus_spells;
    lines.push(Line::from(vec![
        Span::styled(
            "Focus Spells  ",
            Style::default()
                .fg(Color::Magenta)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!("[{focus_count}]"),
            Style::default().fg(Color::DarkGray),
        ),
    ]));

    for (i, fs) in focus_spells.iter().enumerate() {
        let is_cursor = focused && flat_idx == app.spell_cursor;
        let style = if is_cursor {
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };
        lines.push(Line::from(vec![
            Span::styled("  ✓ ", style),
            Span::styled(fs.name.clone(), style),
        ]));
        let _ = i;
        flat_idx += 1;
    }

    // Empty "add" slot for focus spells
    {
        let is_cursor = focused && flat_idx == app.spell_cursor;
        let style = if is_cursor {
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::DarkGray)
        };
        lines.push(Line::from(vec![
            Span::styled("  ○ ", style),
            Span::styled("────", style),
        ]));
        // flat_idx += 1;
    }
    lines.push(Line::from(""));

    Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: false })
}

fn render_feats<'a>(app: &'a App, block: Block<'a>) -> Paragraph<'a> {
    let ch = &app.character;
    let mut lines: Vec<Line> = Vec::new();

    if ch.feats.is_empty() {
        lines.push(Line::from("  No feats selected."));
    } else {
        let mut sorted: Vec<_> = ch.feats.iter().collect();
        sorted.sort_by_key(|f| (f.level, f.feat_type as u8));
        for feat in sorted {
            let type_label = match feat.feat_type {
                crate::model::feat::FeatType::Ancestry => "Anc",
                crate::model::feat::FeatType::Class => "Cls",
                crate::model::feat::FeatType::General => "Gen",
                crate::model::feat::FeatType::Skill => "Skl",
                crate::model::feat::FeatType::Heritage => "Her",
                crate::model::feat::FeatType::Bonus => "Bns",
            };
            let type_color = match feat.feat_type {
                crate::model::feat::FeatType::Ancestry => Color::Yellow,
                crate::model::feat::FeatType::Class => Color::Cyan,
                crate::model::feat::FeatType::Skill => Color::Green,
                crate::model::feat::FeatType::General => Color::White,
                _ => Color::DarkGray,
            };
            lines.push(Line::from(vec![
                Span::styled(
                    format!("{:>2} ", feat.level),
                    Style::default().fg(Color::DarkGray),
                ),
                Span::styled(format!("[{type_label}] "), Style::default().fg(type_color)),
                Span::raw(&feat.name),
            ]));
        }
    }

    Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: false })
}

fn render_equipment<'a>(app: &'a App, block: Block<'a>) -> Paragraph<'a> {
    let eq = &app.choices.equipment;
    let ch = &app.character;
    let focused = app.focus == Focus::DetailTabs && app.detail_tab == DetailTab::Equipment;
    let mut lines: Vec<Line> = Vec::new();
    let mut flat_idx: usize = 0;

    let cursor_style = |idx: usize| -> Style {
        if focused && idx == app.equipment_cursor {
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        }
    };
    let add_style = |idx: usize| -> Style {
        if focused && idx == app.equipment_cursor {
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::DarkGray)
        }
    };

    // Weapons
    lines.push(Line::from(Span::styled(
        "Weapons",
        Style::default()
            .fg(Color::White)
            .add_modifier(Modifier::BOLD),
    )));
    for (i, w) in eq.weapons.iter().enumerate() {
        let style = cursor_style(flat_idx);
        if let Some(wa) = ch.weapon_attacks.get(i) {
            let sign = if wa.attack_bonus >= 0 { "+" } else { "" };
            lines.push(Line::from(vec![
                Span::styled(format!("  {:<26}", wa.display), style),
                Span::styled(format!("{sign}{}  ", wa.attack_bonus), style),
                Span::styled(&wa.damage, style),
            ]));
        } else {
            lines.push(Line::from(Span::styled(
                format!("  {}", w.display_name()),
                style,
            )));
        }
        flat_idx += 1;
    }
    lines.push(Line::from(Span::styled(
        "  + Add Weapon",
        add_style(flat_idx),
    )));
    flat_idx += 1;
    lines.push(Line::from(""));

    // Armor
    lines.push(Line::from(Span::styled(
        "Armor",
        Style::default()
            .fg(Color::White)
            .add_modifier(Modifier::BOLD),
    )));
    for a in &eq.armor {
        let style = cursor_style(flat_idx);
        let tag = if a.worn { "  (worn)" } else { "" };
        lines.push(Line::from(Span::styled(
            format!("  {}{tag}", a.display_name()),
            style,
        )));
        flat_idx += 1;
    }
    lines.push(Line::from(Span::styled(
        "  + Add Armor",
        add_style(flat_idx),
    )));
    flat_idx += 1;
    lines.push(Line::from(""));

    // Shield
    lines.push(Line::from(Span::styled(
        "Shield",
        Style::default()
            .fg(Color::White)
            .add_modifier(Modifier::BOLD),
    )));
    if let Some(s) = &eq.shield {
        let style = cursor_style(flat_idx);
        let tag = if s.raised { "  (raised)" } else { "" };
        lines.push(Line::from(Span::styled(
            format!("  {}{tag}", s.display_name()),
            style,
        )));
        flat_idx += 1;
    }
    lines.push(Line::from(Span::styled(
        "  + Add Shield",
        add_style(flat_idx),
    )));
    flat_idx += 1;
    lines.push(Line::from(""));

    // Items
    let item_count = eq.items.len();
    lines.push(Line::from(Span::styled(
        format!("Items ({item_count})"),
        Style::default()
            .fg(Color::White)
            .add_modifier(Modifier::BOLD),
    )));
    for item in &eq.items {
        let style = cursor_style(flat_idx);
        let qty = if item.quantity > 1 {
            format!(" x{}", item.quantity)
        } else {
            String::new()
        };
        lines.push(Line::from(Span::styled(
            format!("  {}{qty}", item.name),
            style,
        )));
        flat_idx += 1;
    }
    lines.push(Line::from(Span::styled(
        "  + Add Item",
        add_style(flat_idx),
    )));
    // flat_idx += 1; // not needed, last item
    lines.push(Line::from(""));

    // Money
    let m = &eq.money;
    if m.gp > 0 || m.sp > 0 || m.cp > 0 || m.pp > 0 {
        let mut money_parts = Vec::new();
        if m.pp > 0 {
            money_parts.push(format!("{} pp", m.pp));
        }
        if m.gp > 0 {
            money_parts.push(format!("{} gp", m.gp));
        }
        if m.sp > 0 {
            money_parts.push(format!("{} sp", m.sp));
        }
        if m.cp > 0 {
            money_parts.push(format!("{} cp", m.cp));
        }
        lines.push(Line::from(Span::styled(
            format!("  {}", money_parts.join(", ")),
            Style::default().fg(Color::Yellow),
        )));
    }

    Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: false })
}

/// Map skill name to its key ability.
fn skill_ability(skill: &str) -> crate::model::abilities::Ability {
    use crate::model::abilities::Ability;
    match skill {
        "Acrobatics" | "Stealth" | "Thievery" => Ability::Dexterity,
        "Arcana" | "Crafting" | "Occultism" | "Society" => Ability::Intelligence,
        "Athletics" => Ability::Strength,
        "Deception" | "Diplomacy" | "Intimidation" | "Performance" => Ability::Charisma,
        "Medicine" | "Nature" | "Religion" | "Survival" => Ability::Wisdom,
        _ => Ability::Intelligence, // Lores default to INT
    }
}
