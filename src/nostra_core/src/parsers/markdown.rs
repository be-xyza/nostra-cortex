use crate::types::richtext::{CodeBlockContent, Inline, NostraBlock};
use pulldown_cmark::{Event, Options, Parser, Tag};

pub fn parse_markdown(text: &str) -> Vec<NostraBlock> {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);

    let parser = Parser::new_ext(text, options);
    let mut blocks = Vec::new();
    let mut current_text = String::new();

    for event in parser {
        match event {
            Event::Start(_tag) => {
                // Handle start tags (Heading, List, etc.)
                // Simplified for brevity, would need full state machine
            }
            Event::Text(t) => {
                current_text.push_str(&t);
            }
            Event::End(tag) => match tag {
                Tag::Paragraph => {
                    blocks.push(NostraBlock::Paragraph {
                        content: vec![Inline::Text(current_text.clone())],
                    });
                    current_text.clear();
                }
                Tag::Heading(level, _, _) => {
                    let level_u8 = match level {
                        pulldown_cmark::HeadingLevel::H1 => 1,
                        pulldown_cmark::HeadingLevel::H2 => 2,
                        pulldown_cmark::HeadingLevel::H3 => 3,
                        pulldown_cmark::HeadingLevel::H4 => 4,
                        pulldown_cmark::HeadingLevel::H5 => 5,
                        pulldown_cmark::HeadingLevel::H6 => 6,
                    };
                    blocks.push(NostraBlock::Heading {
                        level: level_u8,
                        content: vec![Inline::Text(current_text.clone())],
                    });
                    current_text.clear();
                }
                Tag::CodeBlock(kind) => {
                    let lang = match kind {
                        pulldown_cmark::CodeBlockKind::Fenced(l) => Some(l.to_string()),
                        pulldown_cmark::CodeBlockKind::Indented => None,
                    };
                    blocks.push(NostraBlock::Code {
                        language: lang,
                        content: CodeBlockContent::Simple(current_text.clone()),
                    });
                    current_text.clear();
                }
                _ => {}
            },
            _ => {}
        }
    }

    blocks
}
