use dioxus::prelude::*;

#[derive(Clone, Debug, PartialEq)]
struct FileNode {
    id: String,
    name: String,
    path: String,
    is_dir: bool,
    children: Vec<FileNode>,
    expanded: bool,
}

#[derive(Clone, Debug, PartialEq)]
enum BlockType {
    Paragraph,
    Heading1,
    Heading2,
    CodeBlock,
    Image,
}

#[derive(Clone, Debug, PartialEq)]
struct EditorBlock {
    id: String,
    block_type: BlockType,
    content: String,
    metadata: Option<String>, // e.g., URL for Image, Language for Code
}

#[component]
pub fn ArtifactsEditor() -> Element {
    // VFS Tree (Mock)
    let file_tree = use_signal(|| {
        vec![FileNode {
            id: "root".to_string(),
            name: "root".to_string(),
            path: "/".to_string(),
            is_dir: true,
            expanded: true,
            children: vec![FileNode {
                id: "test_doc".to_string(),
                name: "test.md".to_string(),
                path: "/test.md".to_string(),
                is_dir: false,
                expanded: false,
                children: vec![],
            }],
        }]
    });

    // Active File State
    let mut active_file_path = use_signal(|| Option::<String>::None);

    // Block Editor State
    let mut blocks = use_signal(|| {
        vec![
        EditorBlock {
            id: "b1".to_string(),
            block_type: BlockType::Heading1,
            content: "Welcome to the Block Editor".to_string(),
            metadata: None,
        },
        EditorBlock {
            id: "b2".to_string(),
            block_type: BlockType::Paragraph,
            content: "This is a block-based editing experience. Try clicking below to add more blocks.".to_string(),
            metadata: None,
        }
    ]
    });

    let mut handle_file_click = move |node: &FileNode| {
        if !node.is_dir {
            active_file_path.set(Some(node.path.clone()));
        }
    };

    let mut add_block = move |idx: usize| {
        let mut current = blocks.read().clone();
        current.insert(
            idx + 1,
            EditorBlock {
                id: format!("b_{}", current.len()),
                block_type: BlockType::Paragraph,
                content: "".to_string(),
                metadata: None,
            },
        );
        blocks.set(current);
    };

    rsx! {
        div { class: "flex h-full bg-[#0F172A] text-[#E2E8F0]",
            // Sidebar
            div { class: "w-64 border-r border-[#334155] flex flex-col shrink-0",
                div { class: "p-4 border-b border-[#334155]",
                    span { class: "font-semibold tracking-tight text-sm", "FILESHEM" }
                }
                div { class: "flex-1 p-2 font-mono text-sm",
                     for node in file_tree.read().iter() {
                        {
                             let node = node.clone();
                             rsx! {
                                 div { class: "pl-2",
                                    div {
                                        class: "flex items-center gap-2 p-1 hover:bg-[#1E293B] rounded cursor-pointer text-[#94A3B8]",
                                        onclick: move |_| handle_file_click(&node),
                                        span { "📄" }
                                        span { "{node.name}" }
                                    }
                                 }
                             }
                        }
                    }
                }
            }

            // Main Editor
            div { class: "flex-1 flex flex-col overflow-y-auto",
                // Header
                div { class: "h-12 border-b border-[#334155] bg-[#1E293B] flex items-center justify-between px-6",
                    if let Some(path) = active_file_path.read().as_ref() {
                        div { class: "flex items-center gap-4",
                            span { class: "font-mono text-sm text-[#94A3B8]", "{path}" }
                            // Ingestion Controls (Phase 3)
                            div { class: "flex items-center gap-2",
                                select {
                                    class: "bg-[#0F172A] text-xs text-gray-400 border border-[#334155] rounded px-2 py-1 outline-none",
                                    option { "Discovery Mode (Graphiti)" }
                                    option { "Compliance Mode (OneKE)" }
                                }
                                button {
                                    class: "bg-indigo-600 hover:bg-indigo-500 text-white text-xs px-3 py-1 rounded transition-colors flex items-center gap-1",
                                    onclick: move |_| println!("Ingesting..."),
                                    "⚡ Ingest"
                                }
                            }
                        }
                    } else {
                        span { class: "text-sm text-[#64748B]", "Select a file" }
                    }
                }

                // Block Canvas
                div { class: "flex-1 max-w-3xl mx-auto w-full p-8 pb-32 space-y-2",
                    for (idx, block) in blocks.read().iter().enumerate() {
                        div { class: "group relative pl-6",
                            // Hover Controls
                            div { class: "absolute left-0 top-1.5 opacity-0 group-hover:opacity-100 transition-opacity flex flex-col gap-1",
                                button {
                                    class: "w-4 h-4 flex items-center justify-center rounded hover:bg-[#334155] text-[#94A3B8]",
                                    onclick: move |_| add_block(idx),
                                    "+"
                                }
                                button {
                                    class: "w-4 h-4 flex items-center justify-center rounded hover:bg-[#334155] text-[#94A3B8]",
                                    "::"
                                }
                            }

                            // Block Render
                            match block.block_type {
                                BlockType::Heading1 => rsx! {
                                    input {
                                        class: "w-full bg-transparent text-3xl font-bold text-white outline-none placeholder-[#334155]",
                                        value: "{block.content}",
                                        oninput: move |evt| {
                                            let mut current = blocks.read().clone();
                                            current[idx].content = evt.value();
                                            blocks.set(current);
                                        },
                                        placeholder: "Heading 1"
                                    }
                                },
                                BlockType::Heading2 => rsx! {
                                    input {
                                        class: "w-full bg-transparent text-2xl font-bold text-[#E2E8F0] outline-none placeholder-[#334155] mt-4",
                                        value: "{block.content}",
                                        oninput: move |evt| {
                                            let mut current = blocks.read().clone();
                                            current[idx].content = evt.value();
                                            blocks.set(current);
                                        },
                                        placeholder: "Heading 2"
                                    }
                                },
                                BlockType::Paragraph => rsx! {
                                    textarea {
                                        class: "w-full bg-transparent text-[#CBD5E1] outline-none resize-none overflow-hidden min-h-[1.5em] leading-relaxed",
                                        value: "{block.content}",
                                        oninput: move |evt| {
                                            let mut current = blocks.read().clone();
                                            current[idx].content = evt.value();
                                            blocks.set(current);
                                            // Auto-resize logic would go here
                                        },
                                        placeholder: "Type '/' for commands..."
                                    }
                                },
                                _ => rsx! { div { "Unsupported Block" } }
                            }
                        }
                    }

                    // Trailing Add
                    div {
                        class: "h-8 opacity-0 hover:opacity-100 transition-opacity cursor-pointer flex items-center gap-2 text-[#64748B] hover:text-[#94A3B8]",
                        onclick: move |_| add_block(blocks.read().len().saturating_sub(1)),
                        span { class: "text-lg", "+" }
                        span { class: "text-sm", "Click to add block" }
                    }
                }
            }
        }
    }
}
