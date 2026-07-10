use wayfinder_core::render::content::{ContentBlock, InlineContent};
use wayfinder_core::render::markdown::render_markdown;

#[test]
fn title_renders_as_heading() {
    let blocks = vec![ContentBlock::Title {
        level: 2,
        text: "Fireball".into(),
        right: Some("Spell 3".into()),
        action: Some("Two Actions".into()),
    }];
    let md = render_markdown(&blocks);
    assert_eq!(md, "## Fireball (Two Actions) - Spell 3");
}

#[test]
fn paragraph_renders_inline() {
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
    let md = render_markdown(&blocks);
    assert_eq!(md, "Hello **world**");
}

#[test]
fn key_value_renders() {
    let blocks = vec![ContentBlock::KeyValue {
        key: "Source".into(),
        value: vec![InlineContent::Text {
            text: "CRB pg. 341".into(),
        }],
    }];
    let md = render_markdown(&blocks);
    assert_eq!(md, "**Source** CRB pg. 341");
}

#[test]
fn list_renders() {
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
    let md = render_markdown(&blocks);
    assert_eq!(md, "- Alpha\n- Beta");
}

#[test]
fn table_renders_gfm() {
    let blocks = vec![ContentBlock::Table {
        headers: vec!["Name".into(), "Level".into()],
        rows: vec![vec!["Fireball".into(), "3".into()]],
    }];
    let md = render_markdown(&blocks);
    assert!(md.contains("| Name | Level |"));
    assert!(md.contains("| --- | --- |"));
    assert!(md.contains("| Fireball | 3 |"));
}

#[test]
fn aside_renders_blockquote() {
    let blocks = vec![ContentBlock::Aside {
        title: Some("Sidebar".into()),
        content: vec![ContentBlock::Paragraph {
            content: vec![InlineContent::Text {
                text: "Inner text".into(),
            }],
        }],
    }];
    let md = render_markdown(&blocks);
    assert!(md.contains("> **Sidebar**"));
    assert!(md.contains("> Inner text"));
}

#[test]
fn hrule_renders() {
    let blocks = vec![ContentBlock::HRule];
    let md = render_markdown(&blocks);
    assert_eq!(md, "---");
}

#[test]
fn inline_formatting() {
    let blocks = vec![ContentBlock::Paragraph {
        content: vec![
            InlineContent::Italic {
                text: "emphasis".into(),
            },
            InlineContent::Text {
                text: " and ".into(),
            },
            InlineContent::Link {
                text: "link".into(),
                url: "https://example.com".into(),
            },
            InlineContent::Text {
                text: " with ".into(),
            },
            InlineContent::Trait {
                label: "Fire".into(),
            },
            InlineContent::Text { text: " ".into() },
            InlineContent::Action {
                text: "Single Action".into(),
            },
        ],
    }];
    let md = render_markdown(&blocks);
    assert!(md.contains("_emphasis_"));
    assert!(md.contains("[link](https://example.com)"));
    assert!(md.contains("[Fire]"));
    assert!(md.contains("(Single Action)"));
}
