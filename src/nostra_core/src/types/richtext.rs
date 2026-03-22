use candid::{CandidType, Deserialize};
use serde::Serialize;
use sha2::{Digest, Sha256};

pub type ContributionId = String;
pub type EntityId = String;
pub type Url = String;

#[derive(Clone, Debug, CandidType, Serialize, Deserialize, PartialEq)]
pub enum NostraBlock {
    Heading {
        level: u8,
        content: InlineText,
    },
    Paragraph {
        content: InlineText,
    },
    List {
        ordered: bool,
        items: Vec<Vec<NostraBlock>>,
    },
    Code {
        language: Option<String>,
        content: CodeBlockContent,
    },
    Quote {
        blocks: Vec<NostraBlock>,
    },
    Reference {
        target: ContributionId,
        label: Option<String>,
    },
    ThematicBreak,
}

#[derive(Clone, Debug, CandidType, Serialize, Deserialize, PartialEq)]
pub enum CodeBlockContent {
    Simple(String),
    Segmented(Vec<CodeLine>),
}

#[derive(Clone, Debug, CandidType, Serialize, Deserialize, PartialEq)]
pub struct CodeLine {
    pub line_id: String,
    pub content: String,
    pub annotations: Vec<AnnotationRef>,
}

#[derive(Clone, Debug, CandidType, Serialize, Deserialize, PartialEq)]
pub struct AnnotationRef {
    pub id: String,
    pub author: String,
    pub content: String,
}

#[derive(Clone, Debug, CandidType, Serialize, Deserialize, PartialEq)]
pub enum Inline {
    Text(String),
    Emphasis(Box<Inline>),
    Strong(Box<Inline>),
    Link {
        label: Box<Inline>,
        target: Url,
    },
    ContributionRef {
        id: ContributionId,
        preview: Option<String>,
    },
    EntityRef {
        id: EntityId,
    },
    BookRef {
        book_id: ContributionId,
        edition_id: Option<String>,
        path: Option<String>,
    },
    Span(Vec<Inline>),
}

pub type InlineText = Vec<Inline>;

#[derive(Clone, Debug, CandidType, Serialize, Deserialize, PartialEq)]
pub struct RichContent {
    pub blocks: Vec<NostraBlock>,
    pub original_markdown: Option<String>,
    pub hash: String, // SHA-256 hex
}

impl RichContent {
    pub fn new(blocks: Vec<NostraBlock>, original_markdown: Option<String>) -> Self {
        let hash = Self::hash_blocks(&blocks);
        Self {
            blocks,
            original_markdown,
            hash,
        }
    }

    pub fn hash_blocks(blocks: &Vec<NostraBlock>) -> String {
        let bytes = serde_json::to_vec(blocks).unwrap_or_default();
        let mut hasher = Sha256::new();
        hasher.update(bytes);
        hex::encode(hasher.finalize())
    }
}
