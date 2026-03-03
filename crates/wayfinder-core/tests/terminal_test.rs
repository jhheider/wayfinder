use wayfinder_core::render::content::{ContentBlock, InlineContent};
use wayfinder_core::render::terminal::{Span, render_spans};

#[test]
fn title_renders_with_newlines() {
    let blocks = vec![ContentBlock::Title {
        level: 1,
        text: "Fireball".into(),
        right: None,
        action: None,
    }];
    let spans = render_spans(&blocks);
    assert!(spans.contains(&Span::Title {
        level: 1,
        text: "Fireball".into()
    }));
    // Titles are wrapped in newlines
    assert!(spans.iter().filter(|s| matches!(s, Span::Newline)).count() >= 2);
}

#[test]
fn title_with_action_and_right() {
    let blocks = vec![ContentBlock::Title {
        level: 2,
        text: "Shield".into(),
        right: Some("Cantrip 1".into()),
        action: Some("Single Action".into()),
    }];
    let spans = render_spans(&blocks);
    assert!(spans.contains(&Span::Action("Single Action".into())));
    assert!(
        spans
            .iter()
            .any(|s| matches!(s, Span::Text(t) if t.contains("Cantrip 1")))
    );
}

#[test]
fn paragraph_renders_inline_content() {
    let blocks = vec![ContentBlock::Paragraph {
        content: vec![
            InlineContent::Text {
                text: "Hello ".into(),
            },
            InlineContent::Bold {
                text: "world".into(),
            },
        ],
    }];
    let spans = render_spans(&blocks);
    assert!(spans.contains(&Span::Bold("world".into())));
    assert!(
        spans
            .iter()
            .any(|s| matches!(s, Span::Text(t) if t.contains("Hello")))
    );
}

#[test]
fn key_value_renders_bold_key() {
    let blocks = vec![ContentBlock::KeyValue {
        key: "Source".into(),
        value: vec![InlineContent::Text { text: "CRB".into() }],
    }];
    let spans = render_spans(&blocks);
    assert!(spans.contains(&Span::Bold("Source".into())));
}

#[test]
fn list_renders_items() {
    let blocks = vec![ContentBlock::List {
        items: vec![
            vec![InlineContent::Text {
                text: "Alpha".into(),
            }],
            vec![InlineContent::Text {
                text: "Beta".into(),
            }],
        ],
    }];
    let spans = render_spans(&blocks);
    let list_items: Vec<_> = spans
        .iter()
        .filter(|s| matches!(s, Span::ListItem(_)))
        .collect();
    assert_eq!(list_items.len(), 2);
}

#[test]
fn table_renders_header_and_rows() {
    let blocks = vec![ContentBlock::Table {
        headers: vec!["Name".into(), "Level".into()],
        rows: vec![vec!["Fireball".into(), "3".into()]],
    }];
    let spans = render_spans(&blocks);
    assert!(spans.iter().any(|s| matches!(s, Span::TableHeader(_))));
    assert!(spans.iter().any(|s| matches!(s, Span::TableRow(_))));
}

#[test]
fn hrule_renders() {
    let blocks = vec![ContentBlock::HRule];
    let spans = render_spans(&blocks);
    assert!(spans.contains(&Span::HRule));
}

#[test]
fn aside_renders_title() {
    let blocks = vec![ContentBlock::Aside {
        title: Some("Sidebar".into()),
        content: vec![ContentBlock::Paragraph {
            content: vec![InlineContent::Text {
                text: "Inner".into(),
            }],
        }],
    }];
    let spans = render_spans(&blocks);
    assert!(spans.contains(&Span::AsideTitle("Sidebar".into())));
}

#[test]
fn newline_suppression() {
    // Multiple consecutive blocks that each push newlines shouldn't produce triple+ newlines
    let blocks = vec![
        ContentBlock::HRule,
        ContentBlock::Title {
            level: 1,
            text: "A".into(),
            right: None,
            action: None,
        },
        ContentBlock::Title {
            level: 1,
            text: "B".into(),
            right: None,
            action: None,
        },
    ];
    let spans = render_spans(&blocks);
    // Check no triple newlines
    let mut consecutive = 0;
    for s in &spans {
        if matches!(s, Span::Newline) {
            consecutive += 1;
            assert!(consecutive <= 2, "Triple newline detected");
        } else {
            consecutive = 0;
        }
    }
}

#[test]
fn text_coalescing() {
    let blocks = vec![ContentBlock::Paragraph {
        content: vec![
            InlineContent::Text {
                text: "Hello ".into(),
            },
            InlineContent::Text {
                text: "world".into(),
            },
        ],
    }];
    let spans = render_spans(&blocks);
    // Adjacent Text inlines should be coalesced into one Text span
    let text_spans: Vec<_> = spans
        .iter()
        .filter(|s| matches!(s, Span::Text(_)))
        .collect();
    assert_eq!(text_spans.len(), 1);
    assert!(matches!(&text_spans[0], Span::Text(t) if t == "Hello world"));
}
