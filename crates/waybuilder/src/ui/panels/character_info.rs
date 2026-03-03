use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

use crate::model::abilities::Ability;
use crate::model::proficiencies::{Rank, SKILLS};
use crate::ui::app::{App, Focus};

pub fn render(f: &mut Frame, area: Rect, app: &App) {
    let focused = app.focus == Focus::CharacterInfo;
    let border_style = if focused {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };
    let block = Block::default()
        .title(" Character ")
        .borders(Borders::ALL)
        .border_style(border_style);

    let ch = &app.character;
    let sign = |v: i32| if v >= 0 { format!("+{v}") } else { format!("{v}") };

    let mut lines = vec![
        Line::from(Span::styled(
            &ch.name,
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(ch.summary_line()),
    ];

    if let Some(deity) = &ch.deity {
        lines.push(Line::from(vec![
            Span::styled("Deity: ", Style::default().fg(Color::Gray)),
            Span::raw(deity.as_str()),
        ]));
        if let Some(deity_data) = &app.choices.deity_data {
            // Domains: primary bold/cyan, alternate gray
            if !deity_data.domains.is_empty() {
                let mut domain_spans = vec![Span::styled(
                    "  Domains: ",
                    Style::default().fg(Color::DarkGray),
                )];
                for (i, d) in deity_data.domains.iter().enumerate() {
                    if i > 0 {
                        domain_spans
                            .push(Span::styled(", ", Style::default().fg(Color::DarkGray)));
                    }
                    let is_primary = deity_data.primary_domains.contains(d);
                    let style = if is_primary {
                        Style::default().fg(Color::Cyan)
                    } else {
                        Style::default().fg(Color::DarkGray)
                    };
                    domain_spans.push(Span::styled(d.as_str(), style));
                }
                lines.push(Line::from(domain_spans));
            }
            // Favored weapon
            if let Some(fw) = &deity_data.favored_weapon {
                lines.push(Line::from(vec![
                    Span::styled("  Weapon: ", Style::default().fg(Color::DarkGray)),
                    Span::raw(fw.as_str()),
                ]));
            }
            // Edicts / Anathema (compact)
            if let Some(edicts) = &deity_data.edicts {
                lines.push(Line::from(Span::styled(
                    format!("  Edicts: {edicts}"),
                    Style::default().fg(Color::DarkGray),
                )));
            }
            if let Some(anathema) = &deity_data.anathema {
                lines.push(Line::from(Span::styled(
                    format!("  Anathema: {anathema}"),
                    Style::default().fg(Color::DarkGray),
                )));
            }
        }
    }

    if !ch.languages.is_empty() {
        lines.push(Line::from(vec![
            Span::styled("Languages: ", Style::default().fg(Color::Gray)),
            Span::raw(ch.languages.join(", ")),
        ]));
    }

    let size = ch.size.as_deref().unwrap_or("Medium");
    lines.push(Line::from(vec![
        Span::styled(
            format!("HP {} ", ch.hp_max),
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(format!("Speed {} ", ch.speed), Style::default().fg(Color::Cyan)),
        Span::styled(format!("Size {size}"), Style::default().fg(Color::Gray)),
    ]));

    lines.push(Line::from(""));

    // Abilities: header row + modifier row
    let label_style = Style::default().fg(Color::Gray);
    let val_style = Style::default().fg(Color::White).add_modifier(Modifier::BOLD);

    let header: Vec<Span> = Ability::ALL
        .iter()
        .map(|a| {
            let is_key = ch
                .key_ability
                .as_deref()
                .is_some_and(|k| k.eq_ignore_ascii_case(a.abbr()));
            let style = if is_key {
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else {
                label_style
            };
            Span::styled(format!("{:<6}", a.abbr()), style)
        })
        .collect();
    lines.push(Line::from(header));

    let values: Vec<Span> = Ability::ALL
        .iter()
        .map(|a| Span::styled(format!("{:<6}", sign(ch.abilities.modifier(*a))), val_style))
        .collect();
    lines.push(Line::from(values));

    lines.push(Line::from(""));

    // Saves + Perception
    let save_header: Vec<Span> = vec![
        Span::styled(format!("{:<8}", "Fort"), label_style),
        Span::styled(format!("{:<8}", "Ref"), label_style),
        Span::styled(format!("{:<8}", "Will"), label_style),
        Span::styled(format!("{:<8}", "Per"), label_style),
    ];
    lines.push(Line::from(save_header));

    let save_values: Vec<Span> = vec![
        Span::styled(format!("{:<8}", sign(ch.fortitude_bonus)), val_style),
        Span::styled(format!("{:<8}", sign(ch.reflex_bonus)), val_style),
        Span::styled(format!("{:<8}", sign(ch.will_bonus)), val_style),
        Span::styled(format!("{:<8}", sign(ch.perception_bonus)), val_style),
    ];
    lines.push(Line::from(save_values));

    lines.push(Line::from(""));

    // AC / Class DC
    let mut ac_line = vec![
        Span::styled("AC ", Style::default().fg(Color::Yellow)),
        Span::styled(format!("{:<6}", ch.ac), val_style),
        Span::styled("Class DC ", label_style),
        Span::styled(format!("{:<6}", ch.class_dc), val_style),
    ];
    if ch.spell_caster.is_some() {
        ac_line.push(Span::styled("Spell DC ", label_style));
        ac_line.push(Span::styled(format!("{:<4}", ch.spell_dc), val_style));
        let sign_val = if ch.spell_attack >= 0 {
            format!("+{}", ch.spell_attack)
        } else {
            format!("{}", ch.spell_attack)
        };
        ac_line.push(Span::styled("Atk ", label_style));
        ac_line.push(Span::styled(sign_val, val_style));
    }
    lines.push(Line::from(ac_line));

    lines.push(Line::from(""));

    // Trained skills (compact)
    let mut skill_spans: Vec<Span> = Vec::new();
    for &skill in SKILLS {
        let rank = ch.proficiencies.skill_rank(skill);
        if rank == Rank::Untrained {
            continue;
        }
        let modifier = ch.abilities.modifier(skill_ability(skill));
        let total = modifier + rank.bonus() + ch.level as i32;
        let rank_color = rank_color(rank);
        if !skill_spans.is_empty() {
            skill_spans.push(Span::styled(" │ ", Style::default().fg(Color::DarkGray)));
        }
        skill_spans.push(Span::styled(skill, Style::default().fg(rank_color)));
        skill_spans.push(Span::styled(
            format!(" {}", sign(total)),
            Style::default().fg(Color::White),
        ));
    }
    for (lore, rank) in &ch.proficiencies.lores {
        let total = ch.abilities.modifier(Ability::Intelligence) + rank.bonus() + ch.level as i32;
        if !skill_spans.is_empty() {
            skill_spans.push(Span::styled(" │ ", Style::default().fg(Color::DarkGray)));
        }
        skill_spans.push(Span::styled(
            format!("{lore} Lore"),
            Style::default().fg(rank_color(*rank)),
        ));
        skill_spans.push(Span::styled(
            format!(" {}", sign(total)),
            Style::default().fg(Color::White),
        ));
    }
    if !skill_spans.is_empty() {
        lines.push(Line::from(skill_spans));
    }

    let paragraph = Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: false });
    f.render_widget(paragraph, area);
}

fn rank_color(rank: Rank) -> Color {
    match rank {
        Rank::Legendary => Color::Magenta,
        Rank::Master => Color::Yellow,
        Rank::Expert => Color::Cyan,
        Rank::Trained => Color::Green,
        Rank::Untrained => Color::DarkGray,
    }
}

fn skill_ability(skill: &str) -> Ability {
    match skill {
        "Acrobatics" | "Stealth" | "Thievery" => Ability::Dexterity,
        "Arcana" | "Crafting" | "Occultism" | "Society" => Ability::Intelligence,
        "Athletics" => Ability::Strength,
        "Deception" | "Diplomacy" | "Intimidation" | "Performance" => Ability::Charisma,
        "Medicine" | "Nature" | "Religion" | "Survival" => Ability::Wisdom,
        _ => Ability::Intelligence,
    }
}
