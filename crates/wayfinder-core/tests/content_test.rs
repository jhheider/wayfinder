use wayfinder_core::render::{ContentBlock, InlineContent, parse_content};

const BASE: &str = "https://2e.aonprd.com";

#[test]
fn plain_text() {
    let blocks = parse_content("Hello world", BASE);
    assert_eq!(
        blocks,
        vec![ContentBlock::Paragraph {
            content: vec![InlineContent::Text {
                text: "Hello world".into()
            }],
        },]
    );
}

#[test]
fn bold_only_becomes_key_value() {
    // A paragraph starting with bold is detected as KeyValue
    let blocks = parse_content("**bold**", BASE);
    assert_eq!(
        blocks,
        vec![ContentBlock::KeyValue {
            key: "bold".into(),
            value: vec![],
        },]
    );
}

#[test]
fn bold_with_text() {
    let blocks = parse_content("Some **bold** text", BASE);
    assert_eq!(
        blocks,
        vec![ContentBlock::Paragraph {
            content: vec![
                InlineContent::Text {
                    text: "Some ".into()
                },
                InlineContent::Bold {
                    text: "bold".into()
                },
                InlineContent::Text {
                    text: " text".into()
                },
            ],
        },]
    );
}

#[test]
fn italic_text() {
    let blocks = parse_content("_italic_", BASE);
    assert_eq!(
        blocks,
        vec![ContentBlock::Paragraph {
            content: vec![InlineContent::Italic {
                text: "italic".into()
            }],
        },]
    );
}

#[test]
fn absolute_link() {
    let blocks = parse_content("[Fireball](https://example.com/spell)", BASE);
    assert_eq!(
        blocks,
        vec![ContentBlock::Paragraph {
            content: vec![InlineContent::Link {
                text: "Fireball".into(),
                url: "https://example.com/spell".into(),
            }],
        },]
    );
}

#[test]
fn relative_link_resolved() {
    let blocks = parse_content("[Fireball](/Spells.aspx?ID=119)", BASE);
    assert_eq!(
        blocks,
        vec![ContentBlock::Paragraph {
            content: vec![InlineContent::Link {
                text: "Fireball".into(),
                url: "https://2e.aonprd.com/Spells.aspx?ID=119".into(),
            }],
        },]
    );
}

#[test]
fn title_tag() {
    let blocks = parse_content(r#"<title level="2" right="Spell 3">Fireball</title>"#, BASE);
    assert_eq!(
        blocks,
        vec![ContentBlock::Title {
            level: 2,
            text: "Fireball".into(),
            right: Some("Spell 3".into()),
            action: None,
        },]
    );
}

#[test]
fn title_with_action() {
    let input = r#"<title level="1">Shield<actions string="Single Action" /></title>"#;
    let blocks = parse_content(input, BASE);
    assert_eq!(blocks.len(), 1);
    match &blocks[0] {
        ContentBlock::Title { text, action, .. } => {
            assert_eq!(text, "Shield");
            assert_eq!(action.as_deref(), Some("Single Action"));
        }
        _ => panic!("Expected Title"),
    }
}

#[test]
fn trait_tag() {
    let blocks = parse_content(r#"<trait label="Fire" />"#, BASE);
    assert_eq!(
        blocks,
        vec![ContentBlock::Paragraph {
            content: vec![InlineContent::Trait {
                label: "Fire".into()
            }],
        },]
    );
}

#[test]
fn actions_tag() {
    let blocks = parse_content(r#"<actions string="Two Actions" />"#, BASE);
    assert_eq!(
        blocks,
        vec![ContentBlock::Paragraph {
            content: vec![InlineContent::Action {
                text: "Two Actions".into()
            }],
        },]
    );
}

#[test]
fn hr_tag() {
    let blocks = parse_content("<hr>", BASE);
    assert_eq!(blocks, vec![ContentBlock::HRule]);
}

#[test]
fn br_tag_splits_paragraphs() {
    let blocks = parse_content("Hello<br>World", BASE);
    // <br> flushes the first paragraph, then "World" becomes a second
    assert_eq!(blocks.len(), 2);
    match &blocks[0] {
        ContentBlock::Paragraph { content } => {
            assert_eq!(
                content[0],
                InlineContent::Text {
                    text: "Hello".into()
                }
            );
        }
        _ => panic!("Expected Paragraph"),
    }
}

#[test]
fn aside_tag() {
    let input = r#"<aside title="Sidebar">Inner text</aside>"#;
    let blocks = parse_content(input, BASE);
    assert_eq!(blocks.len(), 1);
    match &blocks[0] {
        ContentBlock::Aside { title, content } => {
            assert_eq!(title.as_deref(), Some("Sidebar"));
            assert_eq!(content.len(), 1);
        }
        _ => panic!("Expected Aside"),
    }
}

#[test]
fn key_value_detection() {
    let blocks = parse_content("**Source** Core Rulebook pg. 341", BASE);
    assert_eq!(
        blocks,
        vec![ContentBlock::KeyValue {
            key: "Source".into(),
            value: vec![InlineContent::Text {
                text: "Core Rulebook pg. 341".into()
            }],
        },]
    );
}

#[test]
fn unordered_list() {
    let input = "<ul><li>Alpha</li><li>Beta</li></ul>";
    let blocks = parse_content(input, BASE);
    assert_eq!(blocks.len(), 1);
    match &blocks[0] {
        ContentBlock::List { items } => {
            assert_eq!(items.len(), 2);
            assert_eq!(
                items[0],
                vec![InlineContent::Text {
                    text: "Alpha".into()
                }]
            );
        }
        _ => panic!("Expected List"),
    }
}

#[test]
fn bare_li() {
    let blocks = parse_content("<li>Solo item</li>", BASE);
    assert_eq!(blocks.len(), 1);
    match &blocks[0] {
        ContentBlock::List { items } => {
            assert_eq!(items.len(), 1);
        }
        _ => panic!("Expected List"),
    }
}

#[test]
fn table_with_headers() {
    let input =
        "<table><tr><th>Name</th><th>Level</th></tr><tr><td>Fireball</td><td>3</td></tr></table>";
    let blocks = parse_content(input, BASE);
    assert_eq!(blocks.len(), 1);
    match &blocks[0] {
        ContentBlock::Table { headers, rows } => {
            assert_eq!(headers, &["Name", "Level"]);
            assert_eq!(rows.len(), 1);
            assert_eq!(rows[0], vec!["Fireball", "3"]);
        }
        _ => panic!("Expected Table"),
    }
}

#[test]
fn bare_tr() {
    let blocks = parse_content("<tr><td>Cell1</td><td>Cell2</td></tr>", BASE);
    assert_eq!(blocks.len(), 1);
    match &blocks[0] {
        ContentBlock::Table { headers, rows } => {
            assert!(headers.is_empty());
            assert_eq!(rows[0], vec!["Cell1", "Cell2"]);
        }
        _ => panic!("Expected Table"),
    }
}

#[test]
fn double_newline_paragraph_break() {
    let blocks = parse_content("First\n\nSecond", BASE);
    assert_eq!(blocks.len(), 2);
}

#[test]
fn hrule_markdown() {
    let blocks = parse_content("---", BASE);
    assert_eq!(blocks, vec![ContentBlock::HRule]);
}

#[test]
fn empty_input() {
    let blocks = parse_content("", BASE);
    assert!(blocks.is_empty());
}

#[test]
fn unclosed_tags_handled() {
    // Should not panic
    let blocks = parse_content("<title level=\"1\">Unclosed", BASE);
    // Title content collected until EOF
    assert!(!blocks.is_empty());
}

#[test]
fn nested_tags_stripped_in_title() {
    let input = r#"<title level="1">Hello <b>World</b></title>"#;
    let blocks = parse_content(input, BASE);
    match &blocks[0] {
        ContentBlock::Title { text, .. } => {
            assert_eq!(text, "Hello World");
        }
        _ => panic!("Expected Title"),
    }
}
