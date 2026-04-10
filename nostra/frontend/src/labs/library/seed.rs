use super::types::*;
use crate::services::vfs_service::VfsService;
use serde::Deserialize;

const SEED_VERSION: &str = "dpub-v1-seed-2026-02-05";

fn slugify_id(id: &str) -> String {
    let mut out = String::with_capacity(id.len());
    for ch in id.chars() {
        if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
            out.push(ch.to_ascii_lowercase());
        } else {
            out.push('_');
        }
    }
    while out.contains("__") {
        out = out.replace("__", "_");
    }
    out.trim_matches('_').to_string()
}

fn cover_color_for_seed(i: usize) -> &'static str {
    const COLORS: &[&str] = &[
        "bg-purple-600",
        "bg-indigo-600",
        "bg-emerald-600",
        "bg-amber-600",
        "bg-cyan-600",
        "bg-teal-600",
        "bg-blue-600",
        "bg-violet-600",
        "bg-rose-600",
        "bg-orange-600",
        "bg-gray-700",
        "bg-slate-700",
    ];
    COLORS[i % COLORS.len()]
}

fn convert_legacy(json: &str, cover_color: &str) -> Option<Book> {
    #[derive(Deserialize)]
    struct LegacyBook {
        id: String,
        title: String,
        author: String,
        chapters: Vec<LegacyChapter>,
        version: Option<String>,
        published: Option<String>,
    }

    #[derive(Deserialize)]
    struct LegacyChapter {
        title: String,
        content: String,
    }

    let legacy = serde_json::from_str::<LegacyBook>(json).ok()?;

    Some(Book {
        context: serde_json::Value::Null,
        meta: BookMeta {
            id: legacy.id,
            title: legacy.title.clone(),
            version: legacy.version,
            phase: Some("Archival".to_string()),
            provenance: Provenance {
                author_did: legacy.author,
                space_did: "did:nostra:space:legacy".to_string(),
                created_at: legacy.published.unwrap_or_default(),
            },
            license: Some("Nostra Open 1.0".to_string()),
        },
        manifest: None,
        cover_color: cover_color.to_string(),
        content: legacy
            .chapters
            .into_iter()
            .enumerate()
            .map(|(i, ch)| BookContent {
                id: format!("ch{}", i),
                title: Some(ch.title.clone()),
                content_type: "Contribution::Chapter".to_string(),
                blocks: vec![
                    Block::Heading {
                        level: 2,
                        content: ch.title,
                    },
                    Block::LegacyHtml { content: ch.content },
                ],
            })
            .collect(),
        editions: vec![],
        latest_edition: None,
        knowledge_graph: None,
        hypothesis: Some(
            "If we codify this knowledge, then ecosystem alignment increases.".to_string(),
        ),
    })
}

fn build_seed_books() -> Vec<Book> {
    // Legacy JSON books (seed source only)
    const LABS_CONSTITUTION_JSON: &str =
        include_str!("../../../../../research/034-nostra-labs/books/labs_constitution.json");
    const UI_UX_MANIFESTO_JSON: &str =
        include_str!("../../../../../research/034-nostra-labs/books/ui_ux_manifesto.json");
    const CONTRIBUTION_LIFECYCLE_JSON: &str = include_str!(
        "../../../../../research/034-nostra-labs/books/contribution_lifecycle_constitution.json"
    );
    const STEWARDSHIP_ROLES_JSON: &str = include_str!(
        "../../../../../research/034-nostra-labs/books/stewardship_roles_constitution.json"
    );
    const AGENT_CHARTER_JSON: &str =
        include_str!("../../../../../research/034-nostra-labs/books/agent_charter.json");
    const SPACES_CONSTITUTION_JSON: &str =
        include_str!("../../../../../research/034-nostra-labs/books/spaces_constitution.json");
    const GOVERNANCE_ESCALATION_JSON: &str = include_str!(
        "../../../../../research/034-nostra-labs/books/governance_escalation_framework.json"
    );
    const KNOWLEDGE_INTEGRITY_JSON: &str = include_str!(
        "../../../../../research/034-nostra-labs/books/knowledge_integrity_doctrine.json"
    );
    const SECURITY_PRIVACY_JSON: &str = include_str!(
        "../../../../../research/034-nostra-labs/books/security_privacy_doctrine.json"
    );

    const NOSTRA_CORE_JSON: &str =
        include_str!("../../../../../research/034-nostra-labs/books/book_nostra_core.json");
    const ICP_CORE_JSON: &str =
        include_str!("../../../../../research/034-nostra-labs/books/book_icp_core.json");

    // V2 book specimen (already in V2 shape)
    const FULL_BOOK_SPEC_JSON: &str =
        include_str!("../../../../../research/057-development-brain/SPECS/FULL_BOOK_SCHEMA.json");

    // dPub bootstrap specimen: Bible (KJV) sample
    const BIBLE_KJV_SAMPLE_JSON: &str = include_str!(
        "../../../../../research/100-bible-native-dpub-corpus/sample_dpubs/bible_kjv_sample.dpub.json"
    );

    let legacy_sources = [
        LABS_CONSTITUTION_JSON,
        UI_UX_MANIFESTO_JSON,
        CONTRIBUTION_LIFECYCLE_JSON,
        STEWARDSHIP_ROLES_JSON,
        AGENT_CHARTER_JSON,
        SPACES_CONSTITUTION_JSON,
        GOVERNANCE_ESCALATION_JSON,
        KNOWLEDGE_INTEGRITY_JSON,
        SECURITY_PRIVACY_JSON,
    ];

    let mut books: Vec<Book> = Vec::new();
    for (i, src) in legacy_sources.iter().enumerate() {
        if let Some(b) = convert_legacy(src, cover_color_for_seed(i)) {
            books.push(b);
        }
    }

    if let Ok(mut bible_sample) = serde_json::from_str::<Book>(BIBLE_KJV_SAMPLE_JSON) {
        bible_sample.cover_color = "bg-emerald-600".to_string();
        bible_sample.manifest = None;
        books.push(bible_sample);
    }

    if let Ok(mut v2_book) = serde_json::from_str::<Book>(FULL_BOOK_SPEC_JSON) {
        v2_book.cover_color = "bg-orange-600".to_string();
        v2_book.manifest = None;
        books.push(v2_book);
    }

    // Legacy mockup import as a preserved artifact
    const LEGACY_MOCKUP_MD: &str = include_str!(
        "../../../../../research/034-nostra-labs/_archive/MOCKUP_LABS_PLAYGROUND_2026-01-19_Initial.md"
    );
    let legacy_mockup_json = serde_json::to_string(&serde_json::json!({
        "id": "legacy_mockup",
        "title": "Legacy Labs Playground Mockup",
        "author": "Nostra Design Team",
        "version": "2026-01-19",
        "published": "2026-01-19",
        "description": "Original mockup for the Labs Playground transformation. Preserved for historical lineage.",
        "chapters": [
            { "title": "Mockup Content", "content": format!("<pre>{}</pre>", LEGACY_MOCKUP_MD) }
        ]
    }))
    .unwrap();
    if let Some(b) = convert_legacy(&legacy_mockup_json, "bg-gray-700") {
        books.push(b);
    }

    // GDPR mock (seed source only; intentionally legacy html)
    books.push(Book {
        context: serde_json::Value::Null,
        meta: BookMeta {
            id: "book_gdpr".to_string(),
            title: "The Nostra GDPR Standard".to_string(),
            version: Some("0.1".to_string()),
            phase: Some("Deliberative".to_string()),
            provenance: Provenance {
                author_did: "Nostra Governance".to_string(),
                space_did: "did:nostra:space:compliance".to_string(),
                created_at: "2026-01-22".to_string(),
            },
            license: Some("CC-BY-SA 4.0".to_string()),
        },
        manifest: None,
        cover_color: "bg-blue-600".to_string(),
        content: vec![
            BookContent {
                id: "ch1".to_string(),
                title: Some("Chapter 1: General Provisions".to_string()),
                content_type: "Contribution::Chapter".to_string(),
                blocks: vec![Block::LegacyHtml {
                    content: r#"
                    <h2>Article 1: Subject-matter and objectives</h2>
                    <p>1. This Regulation lays down rules relating to the protection of natural persons with regard to the processing of personal data and rules relating to the free movement of personal data.</p>
                    <p>2. This Regulation protects fundamental rights and freedoms of natural persons and in particular their right to the protection of personal data.</p>
                    <h2>Article 4: Definitions</h2>
                    <p>For the purposes of this Regulation:</p>
                    <ul>
                        <li>(1) ‘personal data’ means any information relating to an identified or identifiable natural person (‘data subject’);</li>
                        <li>(2) ‘processing’ means any operation or set of operations which is performed on personal data...</li>
                    </ul>
                "#
                    .to_string(),
                }],
            },
            BookContent {
                id: "ch3".to_string(),
                title: Some("Chapter 3: Rights of the Data Subject".to_string()),
                content_type: "Contribution::Chapter".to_string(),
                blocks: vec![Block::LegacyHtml {
                    content: r#"
                    <h2>Article 17: Right to Erasure ('Right to be Forgotten')</h2>
                    <p>1. The data subject shall have the right to obtain from the controller the erasure of personal data concerning him or her without undue delay...</p>
                    <PolicyBlock id='art_17' intent='erasure' />
                    <hr />
                    <h2>Article 20: Right to Data Portability</h2>
                    <p>1. The data subject shall have the right to receive the personal data concerning him or her, which he or she has provided to a controller...</p>
                    <PolicyBlock id='art_20' intent='portability' />
                "#
                    .to_string(),
                }],
            },
        ],
        editions: vec![],
        latest_edition: None,
        knowledge_graph: None,
        hypothesis: Some(
            "If we implement immutable constraints for erasure, then we can satisfy GDPR Art 17 on-chain."
                .to_string(),
        ),
    });

    if let Ok(mut nostra_core) = serde_json::from_str::<Book>(NOSTRA_CORE_JSON) {
        nostra_core.cover_color = "bg-black".to_string();
        nostra_core.manifest = None;
        books.push(nostra_core);
    }

    if let Ok(mut icp_core) = serde_json::from_str::<Book>(ICP_CORE_JSON) {
        icp_core.cover_color = "bg-indigo-600".to_string();
        icp_core.manifest = None;
        books.push(icp_core);
    }

    books
}

pub fn ensure_seeded(vfs: VfsService) -> Result<(), String> {
    let seed_path = "/lib/dpubs/.seed_version";
    if let Some(bytes) = vfs.read_file_bytes(seed_path) {
        if let Ok(s) = String::from_utf8(bytes) {
            if s.trim() == SEED_VERSION {
                return Ok(());
            }
        }
    }

    let mut vfs = vfs;

    for (i, mut book) in build_seed_books().into_iter().enumerate() {
        // Populate a minimal manifest (flat list), to enable edition traversal deterministically.
        if book.manifest.is_none() {
            book.manifest = Some(DPubManifest {
                chapters: book
                    .content
                    .iter()
                    .map(|c| ManifestNode {
                        id: c.id.clone(),
                        title_cache: c.title.clone().unwrap_or_else(|| c.id.clone()),
                        reference: None,
                        children: vec![],
                    })
                    .collect(),
            });
        }

        // cover_color is not serialized; ensure it remains stable across loads by embedding
        // it into meta.versioned seed order as a hint, and then rehydrate in UI.
        // (UI hydration still assigns colors deterministically.)
        book.cover_color = cover_color_for_seed(i).to_string();

        let slug = slugify_id(&book.meta.id);
        let path = format!("/lib/dpubs/{}/dpub.json", slug);
        if vfs.read_file_bytes(&path).is_some() {
            continue;
        }
        let json = serde_json::to_vec(&book).map_err(|e| e.to_string())?;
        vfs.write_file(&path, json, "application/json")?;
    }

    vfs.write_file(seed_path, SEED_VERSION.as_bytes().to_vec(), "text/plain")?;

    Ok(())
}

pub fn dpub_dir_for_id(dpub_id: &str) -> String {
    format!("/lib/dpubs/{}", slugify_id(dpub_id))
}
