#![allow(unused_imports)]
#![allow(non_snake_case)]
use crate::components::icons::{Icon, IconName};
use crate::labs::ingestion_lab::IngestionLab;
use crate::labs::library::components::book_card::BookCard;
use crate::labs::library::components::config_modal::ConfigModal;
use crate::labs::library::components::manifest_editor::ManifestEditor;
use crate::labs::library::components::reader::Reader;
use crate::labs::library::components::sidebar::Sidebar;
use crate::labs::library::data::{create_collections, load_books};
use crate::labs::library::types::*;
use crate::services::vfs_service::VfsService;
use dioxus::prelude::*;
use gloo_storage::{LocalStorage, Storage};
use std::collections::{HashMap, HashSet};

fn is_cross_space_restricted(config: &LabConfig, space_did: &str) -> bool {
    if !config.enforce_treaty {
        return false;
    }
    let Some(current_space) = config.current_space_did.as_ref() else {
        return true;
    };
    if current_space.trim().is_empty() {
        return true;
    }
    if current_space == space_did {
        return false;
    }
    config
        .treaty_token
        .as_deref()
        .unwrap_or_default()
        .trim()
        .is_empty()
}

#[component]
pub fn LibraryLab(on_back: EventHandler<()>) -> Element {
    let _ = &on_back;
    let mut current_view = use_signal(|| LibraryView::Bookshelf);
    let show_raw = use_signal(|| false);
    let show_cortex = use_signal(|| false);
    let mut show_config = use_signal(|| false);
    let config = use_signal(|| LabConfig::default());

    // Inject VFS
    let vfs = use_context::<VfsService>();
    let mut vfs_artifacts = use_signal(|| Vec::<String>::new());

    // Load VFS artifacts on mount
    use_effect(move || {
        let files = vfs.list_dir("lib/artifacts/inbox");
        vfs_artifacts.set(files.iter().map(|f| f.name.clone()).collect());
    });

    // Favorites, Reading List & Notes Signals
    let favorites = use_signal(|| {
        LocalStorage::get::<HashSet<String>>("nostra_library_favorites").unwrap_or_default()
    });
    let reading_list = use_signal(|| {
        LocalStorage::get::<HashSet<String>>("nostra_library_following").unwrap_or_default()
    });
    let notes = use_signal(|| Vec::<Note>::new());
    let active_note_content = use_signal(|| String::new());
    let show_note_panel = use_signal(|| false);
    let reading_progress = use_signal(|| 0.0);
    let mut sync_status = use_signal(|| "Sync".to_string());

    // Persist user collections
    use_effect(move || {
        let _ = LocalStorage::set("nostra_library_favorites", favorites.read().clone());
        let _ = LocalStorage::set("nostra_library_following", reading_list.read().clone());
    });

    // Load dpubs (Books are a view) from VFS (seeded once if empty)
    let initial_books = load_books(vfs);
    let mut books = use_signal(|| initial_books);

    // Create collections based on current books
    let collections = create_collections(&books.read());
    let selected_collection = use_signal(|| Option::<CollectionId>::None);
    let mut search_query = use_signal(|| String::new());
    let mut sort_order = use_signal(|| SortOption::Title);

    rsx! {
        div { class: "flex h-full bg-background text-foreground",
            // Sidebar Component
            Sidebar {
                collections: collections.clone(),
                selected_collection: selected_collection,
                current_view: current_view
            }

            // Main Content Area
            div { class: "flex-1 flex flex-col overflow-hidden",
                match &*current_view.read() {
                    LibraryView::Loom => rsx! {
                        div { class: "flex-1 overflow-hidden relative",
                            button {
                                class: "absolute top-4 left-4 z-10 px-3 py-1 bg-slate-800 rounded border border-slate-700 hover:bg-slate-700",
                                onclick: move |_| current_view.set(LibraryView::Bookshelf),
                                "← Back to Library"
                            }
                            IngestionLab {}
                        }
                    },
                    LibraryView::Bookshelf => {
                         let current_col_id = selected_collection.read();
                         let query = search_query.read().to_lowercase();

                         let mut display_books = books.read().iter()
                             .filter(|b| {
                                 // Filter logic (collection + search)
                                 let matches_collection = if let Some(id) = current_col_id.as_ref() {
                                     match id {
                                         CollectionId::Favorites => favorites.read().contains(&b.meta.id),
                                         CollectionId::ReadingList => reading_list.read().contains(&b.meta.id),
                                         _ => {
                                             if let Some(col) = collections.iter().find(|c| &c.id == id) {
                                                 col.book_ids.contains(&b.meta.id)
                                             } else {
                                                 true
                                             }
                                         }
                                     }
                                 } else {
                                     true
                                 };

                                 let matches_search = if query.is_empty() {
                                     true
                                 } else {
                                     b.meta.title.to_lowercase().contains(&query) ||
                                     b.meta.provenance.author_did.to_lowercase().contains(&query)
                                 };
                                 matches_collection && matches_search
                             })
                             .cloned()
                             .collect::<Vec<_>>();

                        // Sort Results
                        display_books.sort_by(|a, b| match *sort_order.read() {
                            SortOption::Title => a.meta.title.cmp(&b.meta.title),
                            SortOption::Author => a.meta.provenance.author_did.cmp(&b.meta.provenance.author_did),
                            SortOption::Date => b.meta.provenance.created_at.cmp(&a.meta.provenance.created_at),
                        });

                        // Render Bookshelf Header (inline for now)
                        let header_content = if let Some(id) = current_col_id.as_ref() {
                            if let Some(col) = collections.iter().find(|c| &c.id == id) {
                                rsx! {
                                    span { class: "text-3xl", "{col.icon}" }
                                    div {
                                        h1 { class: "text-xl font-bold tracking-tight", "{col.name}" }
                                        p { class: "text-sm text-muted-foreground", "{col.description}" }
                                    }
                                }
                            } else {
                                rsx! {}
                            }
                        } else {
                             rsx! {
                                span { class: "text-3xl", "🏠" }
                                div {
                                    h1 { class: "text-xl font-bold tracking-tight", "All Books" }
                                    p { class: "text-sm text-muted-foreground", "Browse all available knowledge sources." }
                                }
                            }
                        };

                         rsx! {
                            div { class: "h-16 border-b border-border bg-background/50 backdrop-blur flex items-center px-8 justify-between shrink-0",
                                div { class: "flex items-center gap-3",
                                    {header_content}
                                    button {
                                        class: "ml-4 px-3 py-1.5 bg-accent/20 hover:bg-accent/40 border border-accent rounded-lg text-[10px] font-bold uppercase tracking-widest text-accent-foreground transition-all flex items-center gap-2 group",
                                        onclick: move |_| {
                                            println!("Telemetry: Invoking OneKE Legacy Extraction Worker...");
                                        },
                                        svg { class: "w-3 h-3 group-hover:rotate-12 transition-transform", fill: "none", stroke: "currentColor", view_box: "0 0 24 24", stroke_width: "2.5",
                                            path { d: "M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-8l-4-4m0 0L8 8m4-4v12" }
                                        }
                                        "Import Legacy Node"
                                    }
                                    // Sync Button
                                    button {
                                        class: "ml-4 px-3 py-1.5 bg-blue-500/20 hover:bg-blue-500/40 border border-blue-500 rounded-lg text-[10px] font-bold uppercase tracking-widest text-blue-300 transition-all flex items-center gap-2",
                                        onclick: move |_| {
                                            async move {
                                                if *sync_status.read() == "Syncing..." { return; }
                                                let cfg = config.read().clone();
                                                if cfg.enforce_treaty {
                                                    let current_space = cfg
                                                        .current_space_did
                                                        .as_deref()
                                                        .map(|s| s.trim())
                                                        .unwrap_or("");
                                                    if current_space.is_empty() {
                                                        sync_status.set("Set Space".to_string());
                                                        return;
                                                    }
                                                    let treaty = cfg
                                                        .treaty_token
                                                        .as_deref()
                                                        .map(|t| t.trim())
                                                        .unwrap_or("");
                                                    let agent = crate::api::create_agent().await;
                                                    if treaty.is_empty() {
                                                        sync_status.set("Syncing dpubs...".to_string());
                                                        match vfs
                                                            .sync_dpub_from_backend_guarded(
                                                                &agent,
                                                                "/lib/dpubs",
                                                                cfg.current_space_did.clone(),
                                                                cfg.treaty_token.clone(),
                                                            )
                                                            .await
                                                        {
                                                            Ok(count) => {
                                                                sync_status.set(format!("Synced dpubs ({})", count));
                                                                books.set(load_books(vfs));
                                                            }
                                                            Err(e) => {
                                                                println!("Sync Error: {}", e);
                                                                sync_status.set("Error".to_string());
                                                            }
                                                        }
                                                        return;
                                                    }
                                                    sync_status.set("Syncing guarded...".to_string());
                                                    match vfs
                                                        .sync_from_backend_guarded(
                                                            &agent,
                                                            "/lib/",
                                                            cfg.current_space_did.clone(),
                                                            cfg.treaty_token.clone(),
                                                        )
                                                        .await
                                                    {
                                                        Ok(count) => {
                                                            sync_status.set(format!("Synced guarded ({})", count));
                                                            let files = vfs.list_dir("lib/artifacts/inbox");
                                                            vfs_artifacts.set(files.iter().map(|f| f.name.clone()).collect());
                                                            books.set(load_books(vfs));
                                                        }
                                                        Err(e) => {
                                                            println!("Sync Error: {}", e);
                                                            sync_status.set("Error".to_string());
                                                        }
                                                    }
                                                    return;
                                                }
                                                sync_status.set("Syncing...".to_string());
                                                let agent = crate::api::create_agent().await;
                                                match vfs.sync_from_backend(&agent, "/lib/").await {
                                                    Ok(count) => {
                                                        sync_status.set(format!("Synced ({})", count));
                                                        let files = vfs.list_dir("lib/artifacts/inbox");
                                                        vfs_artifacts.set(files.iter().map(|f| f.name.clone()).collect());
                                                        // Reload dpus/books from VFS after sync
                                                        books.set(load_books(vfs));
                                                    },
                                                    Err(e) => {
                                                        println!("Sync Error: {}", e);
                                                        sync_status.set("Error".to_string());
                                                    }
                                                }
                                            }
                                        },
                                        span { "{sync_status}" }
                                    }
                                    if config.read().enforce_treaty {
                                        {
                                            let hint = if config
                                                .read()
                                                .treaty_token
                                                .as_deref()
                                                .map(|t| t.trim())
                                                .unwrap_or("")
                                                .is_empty()
                                            {
                                                "Treaty on: dpubs-only"
                                            } else {
                                                "Treaty on: guarded full sync"
                                            };
                                            rsx! {
                                                span { class: "text-[10px] uppercase tracking-widest text-muted-foreground", "{hint}" }
                                            }
                                        }
                                    }

                                }
                                // Search & Config
                                div { class: "flex items-center gap-2",
                                    // Sort Select
                                    div { class: "relative",
                                        select {
                                            class: "appearance-none bg-muted/50 border border-input rounded-md py-1.5 pl-3 pr-8 text-sm focus:outline-none focus:ring-2 focus:ring-ring cursor-pointer hover:bg-muted transition-colors",
                                            onchange: move |evt| {
                                                match evt.value().as_str() {
                                                    "Title" => sort_order.set(SortOption::Title),
                                                    "Author" => sort_order.set(SortOption::Author),
                                                    "Date" => sort_order.set(SortOption::Date),
                                                     _ => {}
                                                }
                                            },
                                            option { value: "Title", "Sort by Title" }
                                            option { value: "Author", "Sort by Author" }
                                            option { value: "Date", "Sort by Date" }
                                        }
                                        // Icon
                                        div { class: "absolute right-2 top-2 text-muted-foreground pointer-events-none",
                                            svg { class: "w-4 h-4", fill: "none", stroke: "currentColor", view_box: "0 0 24 24", stroke_width: "2",
                                                path { d: "M19 9l-7 7-7-7" }
                                            }
                                        }
                                    }
                                    // Config Button
                                    button {
                                        class: "w-8 h-8 rounded-full bg-muted/50 hover:bg-muted flex items-center justify-center transition-colors text-muted-foreground hover:text-foreground",
                                        onclick: move |_| show_config.set(true),
                                        title: "Library Configuration",
                                        svg { class: "w-4 h-4", fill: "none", stroke: "currentColor", view_box: "0 0 24 24", stroke_width: "2",
                                            path { d: "M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" }
                                            path { d: "M15 12a3 3 0 11-6 0 3 3 0 016 0z" }
                                        }
                                    }
                                    input {
                                         class: "w-full bg-muted/50 border border-input rounded-full pl-10 pr-4 py-1.5 text-sm focus:outline-none focus:ring-2 focus:ring-ring focus:bg-background transition-all",
                                         placeholder: "Search library...",
                                         value: "{search_query}",
                                         oninput: move |evt| search_query.set(evt.value())
                                    }
                                }
                            }

                            // Books Grid Loop
                            div { class: "flex-1 overflow-y-auto p-8",
                                div { class: "grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-8",
                                    for book in display_books {
                                        {
                                            let restricted = {
                                                let cfg = config.read();
                                                is_cross_space_restricted(&cfg, &book.meta.provenance.space_did)
                                            };
                                            rsx! {
                                                BookCard {
                                                    book: book,
                                                    favorites: favorites,
                                                    reading_list: reading_list,
                                                    current_view: current_view,
                                                    restricted: restricted
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    },
                    LibraryView::ManifestEditor(id) => {
                        rsx! {
                            ManifestEditor {
                                dpub_id: id.clone(),
                                books: books,
                                current_view: current_view,
                            }
                        }
                    }
                    LibraryView::Reader(id) => {
                         if let Some(book) = books.read().iter().find(|b| &b.meta.id == id).cloned() {
                             let cfg = config.read().clone();
                             let access_blocked = is_cross_space_restricted(&cfg, &book.meta.provenance.space_did);
                             rsx! {
                                 Reader {
                                     book: book,
                                     current_view: current_view,
                                     show_cortex: show_cortex,
                                     show_raw: show_raw,
                                     show_note_panel: show_note_panel,
                                     notes: notes,
                                     active_note_content: active_note_content,
                                     reading_progress: reading_progress,
                                     access_blocked: access_blocked,
                                     viewer_space_did: cfg.current_space_did.clone(),
                                     treaty_token: cfg.treaty_token.clone(),
                                 }
                             }
                         } else {
                             rsx! { div { "Book not found" } }
                         }
                    }
                }
            }

            ConfigModal {
                show_config: show_config,
                config: config
            }
        }
    }
}
