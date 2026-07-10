use serde_json::json;
use wayfinder_core::aon::Document;
use wayfinder_core::render::{Span, display_short_colored, rarity_colored, render_spans_colored};

fn doc(v: serde_json::Value) -> Document {
    serde_json::from_value(v).unwrap()
}

#[test]
fn renders_every_span_variant() {
    let spans = vec![
        Span::Title {
            level: 1,
            text: "H1".into(),
        },
        Span::Title {
            level: 2,
            text: "H2".into(),
        },
        Span::Title {
            level: 3,
            text: "H3".into(),
        },
        Span::Bold("bold".into()),
        Span::Italic("ital".into()),
        Span::Text("plain".into()),
        Span::Trait("Fire".into()),
        Span::Link {
            text: "link".into(),
            url: "http://x".into(),
        },
        Span::Action("Single Action".into()),
        Span::HRule,
        Span::Newline,
        Span::ListItem(vec![Span::Text("item".into())]),
        Span::TableHeader(vec!["A".into(), "B".into()]),
        Span::TableRow(vec!["1".into(), "2".into()]),
        Span::AsideTitle("Aside".into()),
    ];
    let out = render_spans_colored(&spans);
    for t in [
        "H1", "H2", "H3", "bold", "ital", "plain", "Fire", "link", "item", "A", "B", "1", "2",
        "Aside",
    ] {
        assert!(out.contains(t), "missing {t:?} in output");
    }
    assert!(out.contains('\n'));
}

#[test]
fn rarity_colors_every_tier() {
    for r in ["common", "uncommon", "rare", "unique", "weird"] {
        assert!(rarity_colored(r).contains(r));
    }
}

#[test]
fn display_short_includes_name_category_level_rarity_traits() {
    let d = doc(json!({
        "name": "Fireball", "category": "spell", "level": 3,
        "rarity": "uncommon", "trait": ["Fire", "Evocation"]
    }));
    let line = display_short_colored(&d);
    assert!(line.contains("Fireball"));
    assert!(line.contains("[spell]"));
    assert!(line.contains("Lvl 3"));
    assert!(line.contains("Fire"));
    assert!(line.contains("uncommon"));
}

#[test]
fn display_short_handles_missing_fields_and_hides_common_rarity() {
    let d = doc(json!({ "rarity": "common" }));
    let line = display_short_colored(&d);
    assert!(line.contains("Unknown"));
    assert!(!line.contains("common"));
}
