use crate::services::vfs_service::VfsService;
use dioxus::prelude::*;
use dioxus::web::WebEventExt;
use wasm_bindgen::JsCast;
use web_sys::{DragEvent, FileReader};

#[component]
pub fn IngestionLab() -> Element {
    let vfs = use_context::<VfsService>();
    let mut processing_status = use_signal(|| "Ready to weave.".to_string());
    let mut dropped_files = use_signal::<Vec<String>>(|| vec![]);

    let on_drag_over = move |evt: Event<DragData>| {
        evt.prevent_default();
    };

    let on_drop = move |evt: Event<DragData>| {
        evt.prevent_default();
        evt.stop_propagation();
        processing_status.set("Processing...".to_string());

        // Get the underlying web_sys::DragEvent to access DataTransfer fully
        if let Some(web_event) = evt.as_web_event().dyn_ref::<DragEvent>() {
            if let Some(transfer) = web_event.data_transfer() {
                if let Some(file_list) = transfer.files() {
                    let count = file_list.length();
                    for i in 0..count {
                        if let Some(file) = file_list.item(i) {
                            let name = file.name();
                            let size = file.size();

                            // Scope: Phase 2 (Text/Images only)
                            if size > 5_000_000.0 {
                                processing_status
                                    .set(format!("Skipped {}: Too large (>5MB)", name));
                                continue;
                            }

                            dropped_files.write().push(name.clone());

                            // Read File
                            let reader = FileReader::new().unwrap();
                            let reader_clone = reader.clone();
                            let name_clone = name.clone();
                            let mime = file.type_();
                            let mut vfs_clone = vfs;

                            let onload = wasm_bindgen::closure::Closure::wrap(Box::new(
                                move |_evt: web_sys::Event| {
                                    if let Ok(array_buffer) = reader_clone.result() {
                                        let array = js_sys::Uint8Array::new(&array_buffer);
                                        let bytes = array.to_vec();

                                        // Write to Inbox
                                        let path = format!("lib/artifacts/inbox/{}", name_clone);
                                        match vfs_clone.write_file(&path, bytes, &mime) {
                                            Ok(_) => {}
                                            Err(e) => web_sys::console::log_1(&e.into()),
                                        }
                                    }
                                },
                            )
                                as Box<dyn FnMut(_)>);

                            reader.set_onload(Some(onload.as_ref().unchecked_ref()));
                            onload.forget(); // Leak to keep callback alive
                            reader.read_as_array_buffer(&file).unwrap();
                        }
                    }
                    processing_status.set(format!("Ingested {} threads.", count));
                }
            }
        }
    };

    rsx! {
        div { class: "flex flex-col h-full bg-slate-900 text-slate-100 p-8",
            div { class: "mb-8",
                h1 { class: "text-3xl font-light text-transparent bg-clip-text bg-gradient-to-r from-emerald-400 to-cyan-500",
                    "The Loom"
                }
                p { class: "text-slate-400", "Ingest raw matter into the Knowledge Graph." }
            }

            // Drop Zone
            div {
                class: "flex-1 border-2 border-dashed border-slate-700 rounded-xl flex items-center justify-center transition-colors hover:border-emerald-500 hover:bg-slate-800/50",
                ondragover: on_drag_over,
                ondrop: on_drop,

                div { class: "text-center",
                    div { class: "text-6xl mb-4", "🧶" }
                    h3 { class: "text-xl font-medium", "Drop Artifacts Here" }
                    p { class: "text-slate-500 mt-2", "Markdown, CSV, Images" }
                    p { class: "text-xs text-slate-600 mt-4", "Max 5MB per thread. Local Processing." }
                }
            }

            // Status Bar
            div { class: "mt-8 bg-slate-800 rounded-lg p-4 border border-slate-700",
                div { class: "flex justify-between items-center mb-4",
                    span { class: "font-mono text-sm text-emerald-400", "{processing_status}" }
                    // Export Button (Zip)
                   button { class: "bg-slate-700 hover:bg-slate-600 px-3 py-1 rounded text-xs",
                        "Coming Soon: Export Zip"
                   }
                }

                // Recent Drops
                div { class: "space-y-2",
                    for file in dropped_files.read().iter() {
                        div { class: "flex items-center gap-2 text-sm text-slate-300",
                            span { "📄" }
                            span { "{file}" }
                            span { class: "ml-auto text-xs text-slate-500", "Indexed" }
                        }
                    }
                }
            }
        }
    }
}
