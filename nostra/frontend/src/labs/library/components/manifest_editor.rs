use crate::labs::library::data::dpub_path_for_id;
use crate::labs::library::types::{Book, DPubManifest, LibraryView, ManifestNode};
use crate::labs::library::urn::{VersionSpec, VersionedRef};
use dioxus::prelude::*;
use std::collections::HashMap;

fn rebuild_flat_manifest(book: &mut Book) {
    let mut existing: HashMap<String, VersionedRef> = HashMap::new();
    if let Some(manifest) = book.manifest.as_ref() {
        for node in manifest.chapters.iter() {
            if let Some(r) = node.reference.as_ref() {
                existing.insert(node.id.clone(), r.clone());
            }
        }
    }

    book.manifest = Some(DPubManifest {
        chapters: book
            .content
            .iter()
            .map(|c| ManifestNode {
                id: c.id.clone(),
                title_cache: c.title.clone().unwrap_or_else(|| c.id.clone()),
                reference: Some(
                    existing.get(&c.id).cloned().unwrap_or(VersionedRef {
                        contribution_id: c.id.clone(),
                        version: VersionSpec::Latest,
                        path: None,
                    }),
                ),
                children: vec![],
            })
            .collect(),
    });
}

#[derive(Props, Clone, PartialEq)]
pub struct ManifestEditorProps {
    pub dpub_id: String,
    pub books: Signal<Vec<Book>>,
    pub current_view: Signal<LibraryView>,
}

#[component]
pub fn ManifestEditor(props: ManifestEditorProps) -> Element {
    let vfs = use_context::<crate::services::vfs_service::VfsService>();
    let dpub_id = props.dpub_id;
    let mut books = props.books;
    let mut current_view = props.current_view;
    let dpub_id_for_back = dpub_id.clone();

    let books_for_local = books;
    let dpub_id_for_local = dpub_id.clone();
    let local = use_signal(move || {
        books_for_local
            .read()
            .iter()
            .find(|b| b.meta.id == dpub_id_for_local)
            .cloned()
    });

    let status = use_signal(|| Option::<String>::None);

    let save = move |_| {
        let mut status = status;
        status.set(Some("Saving...".to_string()));

        let current = local.read().clone();
        if let Some(book) = current {
            let path = dpub_path_for_id(&book.meta.id);
            match serde_json::to_vec(&book) {
                Ok(bytes) => {
                    let mut vfs = vfs;
                    if let Err(e) = vfs.write_file(&path, bytes, "application/json") {
                        status.set(Some(format!("Error: {}", e)));
                        return;
                    }

                    let mut updated = books.read().clone();
                    if let Some(i) = updated.iter().position(|b| b.meta.id == book.meta.id) {
                        updated[i] = book;
                        books.set(updated);
                    }
                    status.set(Some("Saved".to_string()));
                }
                Err(e) => status.set(Some(format!("Error: {}", e))),
            }
        }
    };

    rsx! {
        div { class: "flex flex-col h-full bg-background text-foreground",
            div { class: "h-14 border-b border-border bg-card/50 flex items-center justify-between px-4 shrink-0",
                div { class: "flex items-center gap-3",
                    button {
                        class: "text-sm font-medium text-muted-foreground hover:text-foreground hover:bg-muted/50 px-3 py-1.5 rounded-md transition-colors",
                        onclick: move |_| current_view.set(LibraryView::Reader(dpub_id_for_back.clone())),
                        "← Back"
                    }
                    h2 { class: "font-semibold text-sm", "Manifest Editor" }
                    span { class: "text-xs text-muted-foreground font-mono", "{dpub_id}" }
                }
                div { class: "flex items-center gap-2",
                    if let Some(msg) = status.read().as_ref() {
                        span { class: "text-[10px] uppercase tracking-wider text-muted-foreground", "{msg}" }
                    }
                    button {
                        class: "px-3 py-1.5 bg-primary/20 hover:bg-primary/30 border border-primary/40 rounded-lg text-[10px] font-bold uppercase tracking-widest text-primary transition-all",
                        onclick: save,
                        "Save"
                    }
                }
            }

            div { class: "flex-1 overflow-y-auto p-6 max-w-3xl mx-auto w-full",
                if let Some(book) = local.read().as_ref() {
                    div { class: "mb-4",
                        h3 { class: "text-lg font-bold", "{book.meta.title}" }
                        p { class: "text-sm text-muted-foreground", "Reorder chapters (V1 flat manifest). Hierarchy + version pinning are next." }
                    }

                    div { class: "space-y-2",
                        for (idx, ch) in book.content.iter().enumerate() {
                            {
                                let chapter_id = ch.id.clone();
                                let current_ref = book
                                    .manifest
                                    .as_ref()
                                    .and_then(|m| m.chapters.get(idx))
                                    .and_then(|n| n.reference.as_ref())
                                    .cloned()
                                    .unwrap_or(VersionedRef {
                                        contribution_id: chapter_id.clone(),
                                        version: VersionSpec::Latest,
                                        path: None,
                                    });

                                rsx! {
                                    div { class: "flex items-center justify-between gap-3 p-3 rounded-lg border border-border bg-card/30",
                                        div { class: "min-w-0",
                                            div { class: "text-sm font-semibold truncate", "{ch.title.clone().unwrap_or_else(|| ch.id.clone())}" }
                                            div { class: "text-[10px] text-muted-foreground font-mono truncate", "{ch.id}" }
                                            div { class: "mt-1 flex flex-wrap items-center gap-2",
                                                span { class: "text-[10px] uppercase tracking-widest text-muted-foreground", "Ref" }
                                                select {
                                                    class: "bg-muted/40 border border-border rounded px-2 py-1 text-[10px] font-mono",
                                                    value: match &current_ref.version {
                                                        VersionSpec::Latest => "latest",
                                                        VersionSpec::ExactSemver(_) => "exact",
                                                        VersionSpec::Hash(_) => "hash",
                                                        VersionSpec::Range(_) => "range",
                                                        VersionSpec::EditionId(_) => "edition",
                                                    },
                                                    onchange: {
                                                        let chapter_id = chapter_id.clone();
                                                        move |evt| {
                                                            let v = evt.value();
                                                            let mut local = local;
                                                            let current = local.read().clone();
                                                            let Some(mut book) = current else { return; };
                                                            rebuild_flat_manifest(&mut book);
                                                            if let Some(manifest) = book.manifest.as_mut() {
                                                                if let Some(node) = manifest.chapters.get_mut(idx) {
                                                                    let current = node.reference.clone().unwrap_or(VersionedRef {
                                                                        contribution_id: chapter_id.clone(),
                                                                        version: VersionSpec::Latest,
                                                                        path: None,
                                                                    });

                                                                    let next_version = match v.as_str() {
                                                                        "exact" => match current.version {
                                                                            VersionSpec::ExactSemver(s) => VersionSpec::ExactSemver(s),
                                                                            _ => VersionSpec::ExactSemver("1.0.0".to_string()),
                                                                        },
                                                                        "latest" => VersionSpec::Latest,
                                                                        "hash" => match current.version {
                                                                            VersionSpec::Hash(h) => VersionSpec::Hash(h),
                                                                            _ => VersionSpec::Hash(String::new()),
                                                                        },
                                                                        "range" => match current.version {
                                                                            VersionSpec::Range(r) => VersionSpec::Range(r),
                                                                            _ => VersionSpec::Range("^1.0".to_string()),
                                                                        },
                                                                        "edition" => match current.version {
                                                                            VersionSpec::EditionId(id) => VersionSpec::EditionId(id),
                                                                            _ => VersionSpec::EditionId(String::new()),
                                                                        },
                                                                        _ => VersionSpec::Latest,
                                                                    };

                                                                    node.reference = Some(VersionedRef {
                                                                        contribution_id: current.contribution_id,
                                                                        version: next_version,
                                                                        path: current.path,
                                                                    });
                                                                }
                                                            }
                                                            local.set(Some(book));
                                                        }
                                                    },
                                                    option { value: "latest", "latest" }
                                                    option { value: "exact", "vX.Y.Z" }
                                                    option { value: "hash", "hash" }
                                                    option { value: "range", "^X.Y" }
                                                    option { value: "edition", "edition:id" }
                                                }

                                            if let VersionSpec::ExactSemver(v) = &current_ref.version {
                                                input {
                                                    class: "bg-muted/40 border border-border rounded px-2 py-1 text-[10px] font-mono w-28",
                                                    value: "{v}",
                                                    placeholder: "1.0.0",
                                                    oninput: {
                                                        let chapter_id = chapter_id.clone();
                                                        move |evt| {
                                                            let raw = evt.value();
                                                            let cleaned = raw.trim().trim_start_matches('v').to_string();
                                                            let mut local = local;
                                                            let current = local.read().clone();
                                                            let Some(mut book) = current else { return; };
                                                            rebuild_flat_manifest(&mut book);
                                                            if let Some(manifest) = book.manifest.as_mut() {
                                                                if let Some(node) = manifest.chapters.get_mut(idx) {
                                                                    let base = node.reference.clone().unwrap_or(VersionedRef {
                                                                        contribution_id: chapter_id.clone(),
                                                                        version: VersionSpec::Latest,
                                                                        path: None,
                                                                    });
                                                                    node.reference = Some(VersionedRef {
                                                                        contribution_id: base.contribution_id,
                                                                        version: VersionSpec::ExactSemver(cleaned),
                                                                        path: base.path,
                                                                    });
                                                                }
                                                            }
                                                            local.set(Some(book));
                                                        }
                                                    }
                                                }
                                            }

                                            if let VersionSpec::Hash(v) = &current_ref.version {
                                                input {
                                                    class: "bg-muted/40 border border-border rounded px-2 py-1 text-[10px] font-mono w-40",
                                                    value: "{v}",
                                                    placeholder: "content hash",
                                                    oninput: {
                                                        let chapter_id = chapter_id.clone();
                                                        move |evt| {
                                                            let cleaned = evt.value().trim().to_string();
                                                            let mut local = local;
                                                            let current = local.read().clone();
                                                            let Some(mut book) = current else { return; };
                                                            rebuild_flat_manifest(&mut book);
                                                            if let Some(manifest) = book.manifest.as_mut() {
                                                                if let Some(node) = manifest.chapters.get_mut(idx) {
                                                                    let base = node.reference.clone().unwrap_or(VersionedRef {
                                                                        contribution_id: chapter_id.clone(),
                                                                        version: VersionSpec::Latest,
                                                                        path: None,
                                                                    });
                                                                    node.reference = Some(VersionedRef {
                                                                        contribution_id: base.contribution_id,
                                                                        version: VersionSpec::Hash(cleaned),
                                                                        path: base.path,
                                                                    });
                                                                }
                                                            }
                                                            local.set(Some(book));
                                                        }
                                                    }
                                                }
                                            }

                                            if let VersionSpec::Range(v) = &current_ref.version {
                                                input {
                                                    class: "bg-muted/40 border border-border rounded px-2 py-1 text-[10px] font-mono w-28",
                                                    value: "{v}",
                                                    placeholder: "^1.0",
                                                    oninput: {
                                                        let chapter_id = chapter_id.clone();
                                                        move |evt| {
                                                            let raw = evt.value().trim().to_string();
                                                            let cleaned = if raw.starts_with('^') { raw } else { format!("^{}", raw) };
                                                            let mut local = local;
                                                            let current = local.read().clone();
                                                            let Some(mut book) = current else { return; };
                                                            rebuild_flat_manifest(&mut book);
                                                            if let Some(manifest) = book.manifest.as_mut() {
                                                                if let Some(node) = manifest.chapters.get_mut(idx) {
                                                                    let base = node.reference.clone().unwrap_or(VersionedRef {
                                                                        contribution_id: chapter_id.clone(),
                                                                        version: VersionSpec::Latest,
                                                                        path: None,
                                                                    });
                                                                    node.reference = Some(VersionedRef {
                                                                        contribution_id: base.contribution_id,
                                                                        version: VersionSpec::Range(cleaned),
                                                                        path: base.path,
                                                                    });
                                                                }
                                                            }
                                                            local.set(Some(book));
                                                        }
                                                    }
                                                }
                                            }

                                            if let VersionSpec::EditionId(v) = &current_ref.version {
                                                input {
                                                    class: "bg-muted/40 border border-border rounded px-2 py-1 text-[10px] font-mono w-36",
                                                    value: "{v}",
                                                    placeholder: "edition id",
                                                    oninput: {
                                                        let chapter_id = chapter_id.clone();
                                                        move |evt| {
                                                            let cleaned = evt.value().trim().to_string();
                                                            let mut local = local;
                                                            let current = local.read().clone();
                                                            let Some(mut book) = current else { return; };
                                                            rebuild_flat_manifest(&mut book);
                                                            if let Some(manifest) = book.manifest.as_mut() {
                                                                if let Some(node) = manifest.chapters.get_mut(idx) {
                                                                    let base = node.reference.clone().unwrap_or(VersionedRef {
                                                                        contribution_id: chapter_id.clone(),
                                                                        version: VersionSpec::Latest,
                                                                        path: None,
                                                                    });
                                                                    node.reference = Some(VersionedRef {
                                                                        contribution_id: base.contribution_id,
                                                                        version: VersionSpec::EditionId(cleaned),
                                                                        path: base.path,
                                                                    });
                                                                }
                                                            }
                                                            local.set(Some(book));
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                        div { class: "flex items-center gap-2 shrink-0",
                                            button {
                                                class: "w-8 h-8 rounded bg-muted/50 hover:bg-muted border border-border text-muted-foreground hover:text-foreground transition-colors",
                                                onclick: move |_| {
                                                    let mut local = local;
                                                    let current = local.read().clone();
                                                    let Some(mut book) = current else { return; };
                                                    if idx == 0 || idx >= book.content.len() {
                                                        return;
                                                    }
                                                    book.content.swap(idx, idx - 1);
                                                    rebuild_flat_manifest(&mut book);
                                                    local.set(Some(book));
                                                },
                                                title: "Move up",
                                                "↑"
                                            }
                                            button {
                                                class: "w-8 h-8 rounded bg-muted/50 hover:bg-muted border border-border text-muted-foreground hover:text-foreground transition-colors",
                                                onclick: move |_| {
                                                    let mut local = local;
                                                    let current = local.read().clone();
                                                    let Some(mut book) = current else { return; };
                                                    if idx + 1 >= book.content.len() {
                                                        return;
                                                    }
                                                    book.content.swap(idx, idx + 1);
                                                    rebuild_flat_manifest(&mut book);
                                                    local.set(Some(book));
                                                },
                                                title: "Move down",
                                                "↓"
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                } else {
                    div { class: "text-sm text-muted-foreground", "DPub not found." }
                }
            }
        }
    }
}
