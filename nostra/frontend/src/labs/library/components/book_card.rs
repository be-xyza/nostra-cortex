use dioxus::prelude::*;
use std::collections::HashSet;
use crate::labs::library::types::{Book, LibraryView};

#[component]
pub fn BookCard(
    book: Book,
    favorites: Signal<HashSet<String>>,
    reading_list: Signal<HashSet<String>>,
    current_view: Signal<LibraryView>,
    restricted: bool,
) -> Element {
    rsx! {
        div {
            class: format!(
                "group flex flex-col gap-4 {}",
                if restricted { "cursor-not-allowed opacity-70" } else { "cursor-pointer" }
            ),
            onclick: {
                let bid = book.meta.id.clone();
                move |_| {
                    if restricted {
                        return;
                    }
                    current_view.set(LibraryView::Reader(bid.clone()))
                }
            },
            // Book Cover Card
            div { class: "aspect-[2/3] relative rounded-r-2xl rounded-l-md shadow-lg transition-all duration-300 group-hover:shadow-2xl group-hover:-translate-y-2 group-hover:scale-[1.02] overflow-hidden",
                // Spine / Binding effect
                div { class: "absolute left-0 top-0 bottom-0 w-3 bg-gradient-to-r from-black/40 to-transparent z-10" }

                // Cover Content
                div { class: format!("absolute inset-0 {} flex flex-col p-6 text-white text-center justify-between", book.cover_color),
                    div { class: "absolute inset-0 bg-black/10" } // Texture overlay

                    // Top: Author & Favorite
                    div { class: "relative z-10 flex justify-between items-start opacity-90",
                        span { class: "text-[10px] font-medium tracking-widest uppercase", "{book.meta.provenance.author_did}" }
                        button {
                            class: "p-1 -mt-1 -mr-1 hover:scale-110 transition-transform",
                            onclick: {
                                let bid = book.meta.id.clone();
                                move |evt: Event<MouseData>| {
                                    evt.stop_propagation();
                                    let mut favs = favorites.write();
                                    if favs.contains(&bid) {
                                        favs.remove(&bid);
                                    } else {
                                        favs.insert(bid.clone());
                                    }
                                }
                            },
                            svg {
                                class: format!("w-4 h-4 {}", if favorites.read().contains(&book.meta.id) { "text-red-500 fill-current" } else { "text-white/50 hover:text-white" }),
                                fill: "none", stroke: "currentColor", view_box: "0 0 24 24", stroke_width: "2",
                                path { d: "M4.318 6.318a4.5 4.5 0 000 6.364L12 20.364l7.682-7.682a4.5 4.5 0 00-6.364-6.364L12 7.636l-1.318-1.318a4.5 4.5 0 00-6.364 0z" }
                            }
                        }
                    }

                    // Center: Title
                    div { class: "relative z-10 font-serif font-bold text-2xl leading-tight",
                        "{book.meta.title}"
                    }

                    // Bottom: Logo/Publisher
                    div { class: "relative z-10 opacity-70",
                        svg { class: "w-6 h-6 mx-auto", fill: "currentColor", view_box: "0 0 24 24",
                            path { d: "M12 2L2 7l10 5 10-5-10-5zm0 9l2.5-1.25L12 8.5l-2.5 1.25L12 11zm0 2.5l-5-2.5-5 2.5L12 22l10-8.5-5-2.5-5 2.5z" }
                        }
                    }
                }

                if restricted {
                    div { class: "absolute inset-0 bg-black/60 flex items-center justify-center z-20",
                        div { class: "px-3 py-1.5 rounded-full bg-black/70 border border-white/20 text-white text-[10px] uppercase tracking-widest",
                            "Restricted"
                        }
                    }
                }
            }

            // Meta below cover
            div { class: "px-1",
                h3 { class: "font-semibold text-base mb-1 truncate", "{book.meta.title}" }

                // Priority 2: Badges Row (License, Time, Squad)
                div { class: "flex flex-wrap gap-2 text-[10px] items-center mb-2",
                     // License Badge
                     if let Some(lic) = &book.meta.license {
                        div { class: "flex items-center gap-1 px-1.5 py-0.5 bg-muted rounded border border-border/50 text-muted-foreground",
                            span { "⚖️" }
                            "{lic}"
                        }
                     }
                     // Time Badge (Mock relative time)
                     div { class: "flex items-center gap-1 px-1.5 py-0.5 bg-muted rounded border border-border/50 text-muted-foreground",
                        span { "🕒" }
                        "{book.meta.provenance.created_at}"
                     }
                     // Squad Badge (Parsed from DID)
                     div { class: "flex items-center gap-1 px-1.5 py-0.5 bg-muted rounded border border-border/50 text-muted-foreground",
                        span { "🛡️" }
                        {
                            let squad = book.meta.provenance.space_did.split(':').last().unwrap_or("core");
                            rsx!{ "{squad}" }
                        }
                     }
                }

                // Tags / Config / Actions
                div { class: "flex gap-2 mt-2 items-center justify-between",
                     div { class: "flex gap-2",
                        if let Some(v) = &book.meta.version {
                            span { class: "px-1.5 py-0.5 bg-muted rounded text-[10px] font-mono text-muted-foreground",
                                "v{v}"
                            }
                        }
                     }

                     // Actions (Fav/List)
                     div { class: "flex gap-1",
                        // Favorite Toggle
                        button {
                            class: "p-1 rounded hover:bg-accent/50 transition-colors",
                            onclick: {
                                let bid = book.meta.id.clone();
                                move |e: Event<MouseData>| {
                                    e.stop_propagation();
                                    let mut favs = favorites.write();
                                    if favs.contains(&bid) {
                                        favs.remove(&bid);
                                    } else {
                                        favs.insert(bid.clone());
                                    }
                                }
                            },
                            svg {
                                class: format!("w-4 h-4 {}", if favorites.read().contains(&book.meta.id) { "text-yellow-500 fill-current" } else { "text-muted-foreground/50" }),
                                view_box: "0 0 24 24", stroke: "currentColor", stroke_width: "2", fill: "none",
                                path { d: "M11.049 2.927c.3-.921 1.603-.921 1.902 0l1.519 4.674a1 1 0 00.95.69h4.915c.969 0 1.371 1.24.588 1.81l-3.976 2.888a1 1 0 00-.363 1.118l1.518 4.674c.3.922-.755 1.688-1.538 1.118l-3.976-2.888a1 1 0 00-1.176 0l-3.976 2.888c-.783.57-1.838-.197-1.538-1.118l1.518-4.674a1 1 0 00-.363-1.118l-3.976-2.888c-.784-.57-.38-1.81.588-1.81h4.914a1 1 0 00.951-.69l1.519-4.674z" }
                            }
                        }
                        // Reading List Toggle
                        button {
                            class: "p-1 rounded hover:bg-accent/50 transition-colors",
                            onclick: {
                                let bid = book.meta.id.clone();
                                move |e: Event<MouseData>| {
                                    e.stop_propagation();
                                    let mut list = reading_list.write();
                                    if list.contains(&bid) {
                                        list.remove(&bid);
                                    } else {
                                        list.insert(bid.clone());
                                    }
                                }
                            },
                            svg {
                                class: format!("w-4 h-4 {}", if reading_list.read().contains(&book.meta.id) { "text-orange-500 fill-current" } else { "text-muted-foreground/50" }),
                                view_box: "0 0 24 24", stroke: "currentColor", stroke_width: "2", fill: "none",
                                path { d: "M5 5a2 2 0 012-2h10a2 2 0 012 2v16l-7-3.5L5 21V5z" }
                            }
                        }
                     }
                }
            }
        }
    }
}
