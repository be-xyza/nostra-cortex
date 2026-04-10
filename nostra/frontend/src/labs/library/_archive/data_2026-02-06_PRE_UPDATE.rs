use super::seed::{dpub_dir_for_id, ensure_seeded};
use super::urn::{VersionSpec, VersionedRef};
use super::types::*;
use crate::services::vfs_service::{NodeType, VfsService};

fn cover_color_from_id(id: &str) -> String {
    // Deterministic, stable mapping without persisting UI-only fields.
    let palette = [
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
    let mut acc: u64 = 1469598103934665603;
    for b in id.as_bytes() {
        acc ^= *b as u64;
        acc = acc.wrapping_mul(1099511628211);
    }
    palette[(acc as usize) % palette.len()].to_string()
}

fn hydrate_ui_fields(mut book: Book) -> Book {
    book.cover_color = cover_color_from_id(&book.meta.id);
    book
}

pub fn dpub_path_for_id(dpub_id: &str) -> String {
    format!("{}/dpub.json", dpub_dir_for_id(dpub_id))
}

pub fn editions_dir_for_id(dpub_id: &str) -> String {
    format!("{}/editions", dpub_dir_for_id(dpub_id))
}

pub fn edition_dir_for_id(dpub_id: &str, version: &str) -> String {
    format!("{}/{}", editions_dir_for_id(dpub_id), version)
}

pub fn edition_manifest_path_for_id(dpub_id: &str, version: &str) -> String {
    format!("{}/edition_manifest.json", edition_dir_for_id(dpub_id, version))
}

pub fn edition_snapshot_path_for_id(dpub_id: &str, version: &str) -> String {
    format!("{}/snapshot.json", edition_dir_for_id(dpub_id, version))
}

pub fn feed_path_for_id(dpub_id: &str) -> String {
    format!("{}/feed.json", dpub_dir_for_id(dpub_id))
}

pub fn load_books(vfs: VfsService) -> Vec<Book> {
    let _ = ensure_seeded(vfs);

    let entries = vfs.list_dir("/lib/dpubs");
    let mut books: Vec<Book> = Vec::new();

    for entry in entries {
        if entry.node_type != NodeType::Directory {
            continue;
        }
        let dpub_path = format!("/lib/dpubs/{}/dpub.json", entry.name);
        if let Some(bytes) = vfs.read_file_bytes(&dpub_path) {
            if let Ok(mut book) = serde_json::from_slice::<Book>(&bytes) {
                // Ensure a manifest exists (flat) so publishing can traverse deterministically.
                if book.manifest.is_none() {
                    book.manifest = Some(DPubManifest {
                        chapters: book
                            .content
                            .iter()
                            .map(|c| ManifestNode {
                                id: c.id.clone(),
                                title_cache: c.title.clone().unwrap_or_else(|| c.id.clone()),
                                reference: Some(VersionedRef {
                                    contribution_id: c.id.clone(),
                                    version: VersionSpec::Latest,
                                    path: None,
                                }),
                                children: vec![],
                            })
                            .collect(),
                    });
                }
                books.push(hydrate_ui_fields(book));
            }
        }
    }

    // Fallback ordering: title
    books.sort_by(|a, b| a.meta.title.cmp(&b.meta.title));
    books
}

pub fn create_collections(_books: &Vec<Book>) -> Vec<BookCollection> {
    let mut collections = Vec::new();

    // 1. ALL Constitutional Documents (The Complete Nine)
    let all_constitutional_ids = vec![
        "urn:nostra:book:labs-constitution".to_string(),
        "urn:nostra:book:ui-ux-manifesto".to_string(),
        "urn:nostra:book:contribution-lifecycle".to_string(),
        "urn:nostra:book:stewardship-roles".to_string(),
        "urn:nostra:book:agent-charter".to_string(),
        "urn:nostra:book:spaces-constitution".to_string(),
        "urn:nostra:book:governance-escalation".to_string(),
        "urn:nostra:book:knowledge-integrity".to_string(),
        "urn:nostra:book:security-privacy".to_string(),
    ];

    collections.push(BookCollection {
        id: CollectionId::Constitutional,
        name: "The Nine Constitutions".to_string(),
        description: "Foundational constitutions governing Nostra Cortex.".to_string(),
        icon: "📜".to_string(),
        book_ids: all_constitutional_ids,
        color_theme: "purple".to_string(),
    });

    // 2. Culture
    collections.push(BookCollection {
        id: CollectionId::Culture,
        name: "Culture".to_string(),
        description: "Labs and UI/UX philosophy for building meaning.".to_string(),
        icon: "🧠".to_string(),
        book_ids: vec![
            "urn:nostra:book:labs-constitution".to_string(),
            "urn:nostra:book:ui-ux-manifesto".to_string(),
        ],
        color_theme: "indigo".to_string(),
    });

    // 3. Structure
    collections.push(BookCollection {
        id: CollectionId::Structure,
        name: "Structure".to_string(),
        description: "Spaces, roles, and lifecycle governance.".to_string(),
        icon: "🏛️".to_string(),
        book_ids: vec![
            "urn:nostra:book:spaces-constitution".to_string(),
            "urn:nostra:book:stewardship-roles".to_string(),
            "urn:nostra:book:contribution-lifecycle".to_string(),
        ],
        color_theme: "emerald".to_string(),
    });

    // 4. Authority
    collections.push(BookCollection {
        id: CollectionId::Authority,
        name: "Authority".to_string(),
        description: "Agents, escalation, and decision frameworks.".to_string(),
        icon: "⚖️".to_string(),
        book_ids: vec![
            "urn:nostra:book:agent-charter".to_string(),
            "urn:nostra:book:governance-escalation".to_string(),
        ],
        color_theme: "blue".to_string(),
    });

    // 5. Security
    collections.push(BookCollection {
        id: CollectionId::Security,
        name: "Security & Integrity".to_string(),
        description: "Truth, confidence, and privacy doctrines.".to_string(),
        icon: "🛡️".to_string(),
        book_ids: vec![
            "urn:nostra:book:knowledge-integrity".to_string(),
            "urn:nostra:book:security-privacy".to_string(),
        ],
        color_theme: "rose".to_string(),
    });

    // 6. Compliance
    collections.push(BookCollection {
        id: CollectionId::Compliance,
        name: "Compliance".to_string(),
        description: "Regulatory standards (mock + future).".to_string(),
        icon: "✅".to_string(),
        book_ids: vec!["book_gdpr".to_string()],
        color_theme: "slate".to_string(),
    });

    // 7. System
    collections.push(BookCollection {
        id: CollectionId::System,
        name: "System".to_string(),
        description: "Internal specs and preserved lineage artifacts.".to_string(),
        icon: "🧰".to_string(),
        book_ids: vec!["legacy_mockup".to_string()],
        color_theme: "gray".to_string(),
    });

    // User collections (computed dynamically in LibraryLab filter logic)
    collections.push(BookCollection {
        id: CollectionId::Favorites,
        name: "Favorites".to_string(),
        description: "Starred dpus.".to_string(),
        icon: "★".to_string(),
        book_ids: vec![],
        color_theme: "yellow".to_string(),
    });

    collections.push(BookCollection {
        id: CollectionId::ReadingList,
        name: "Following".to_string(),
        description: "Subscribed dpus.".to_string(),
        icon: "📌".to_string(),
        book_ids: vec![],
        color_theme: "blue".to_string(),
    });

    collections
}
