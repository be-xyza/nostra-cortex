use crate::labs::library::types::{BookCollection, CollectionId, LibraryView};
use dioxus::prelude::*;

#[component]
pub fn Sidebar(
    collections: Vec<BookCollection>,
    selected_collection: Signal<Option<CollectionId>>,
    current_view: Signal<LibraryView>,
) -> Element {
    rsx! {
        div { class: "w-64 border-r border-border bg-card/30 flex flex-col shrink-0",
            div { class: "p-4 border-b border-border/50",
                h2 { class: "font-semibold tracking-tight flex items-center gap-2",
                    span { "📚" }
                    "Library"
                }
            }
            nav { class: "flex-1 p-3 space-y-1 overflow-y-auto",
                // All Books
                button {
                    class: format!("w-full text-left px-3 py-2 rounded-md text-sm font-medium transition-colors {}",
                        if selected_collection.read().is_none() && matches!(*current_view.read(), LibraryView::Bookshelf) {
                            "bg-accent text-accent-foreground"
                        } else {
                            "text-muted-foreground hover:bg-accent/50 hover:text-foreground"
                        }
                    ),
                    onclick: move |_| {
                        selected_collection.set(None);
                        current_view.set(LibraryView::Bookshelf);
                    },
                    "🏠 All Books"
                }

                div { class: "pt-4 pb-2 px-3 text-xs font-semibold text-muted-foreground uppercase tracking-wider",
                    "Collections"
                }

                for collection in collections {
                    {
                        let is_active = selected_collection.read().as_ref() == Some(&collection.id) && matches!(*current_view.read(), LibraryView::Bookshelf);
                        let col_id = collection.id.clone();
                        rsx! {
                            button {
                                class: format!("w-full text-left px-3 py-2 rounded-md text-sm font-medium transition-colors flex items-center gap-2 {}",
                                    if is_active { "bg-accent text-accent-foreground" } else { "text-muted-foreground hover:bg-accent/50 hover:text-foreground" }
                                ),
                                onclick: move |_| {
                                    selected_collection.set(Some(col_id.clone()));
                                    current_view.set(LibraryView::Bookshelf);
                                },
                                span { "{collection.icon}" }
                                span { "{collection.name}" }
                            }
                        }
                    }
                }
            }
        }
    }
}
