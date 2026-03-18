use cortex_domain::graph::{Edge, EdgeKind, Graph, Node};
use cortex_domain::memory_fs::{Oid, TreeEntry};
use std::collections::BTreeMap;

use crate::RuntimeError;

use super::sandbox::SandboxFs;

#[async_trait::async_trait]
pub trait MaterializerPort: Send + Sync {
    /// Materialize a specific semantic `Graph` from a sandbox's ingested tree hierarchy.
    async fn materialize(
        &self,
        sandbox: &SandboxFs,
        root_tree_oid: &Oid,
    ) -> Result<Graph, RuntimeError>;
}

pub struct FileGraphMaterializer;

#[async_trait::async_trait]
impl MaterializerPort for FileGraphMaterializer {
    /// Produces the `file_graph` layer of a `RepoProjection`:
    /// - Each directory becomes a `Node` with `node_type = "directory"`
    /// - Each file becomes a `Node` with `node_type = "file"` and attributes
    ///   for `extension` and `size_bytes`
    /// - Parent → child relationships become `EdgeKind::Custom("contains")` edges
    /// - Markdown/wikilinks extracted from `.md` files become `EdgeKind::References` edges
    async fn materialize(
        &self,
        sandbox: &SandboxFs,
        root_tree_oid: &Oid,
    ) -> Result<Graph, RuntimeError> {
        let mut graph = Graph::default();
        walk_tree(sandbox, root_tree_oid, "", &mut graph).await?;
        Ok(graph)
    }
}

/// Recursively walk the tree, adding nodes and edges to the graph.
async fn walk_tree(
    sandbox: &SandboxFs,
    tree_oid: &Oid,
    path_prefix: &str,
    graph: &mut Graph,
) -> Result<(), RuntimeError> {
    let tree = sandbox.read_tree(tree_oid).await?;

    for (name, entry) in &tree.entries {
        let full_path = if path_prefix.is_empty() {
            name.clone()
        } else {
            format!("{}/{}", path_prefix, name)
        };

        match entry {
            TreeEntry::Blob(blob_oid) => {
                let blob = sandbox.read_blob(blob_oid).await?;
                let extension = name
                    .rsplit('.')
                    .next()
                    .filter(|_ext| name.contains('.'))
                    .unwrap_or("")
                    .to_string();

                let mut attributes = BTreeMap::new();
                attributes.insert("extension".to_string(), extension.clone());
                attributes.insert("size_bytes".to_string(), blob.content.len().to_string());
                attributes.insert("name".to_string(), name.clone());

                graph.add_node(Node {
                    id: full_path.clone(),
                    node_type: "file".to_string(),
                    attributes,
                });

                // Add containment edge from parent directory
                if !path_prefix.is_empty() {
                    graph.add_edge(Edge {
                        from: path_prefix.to_string(),
                        to: full_path.clone(),
                        kind: EdgeKind::Custom("contains".to_string()),
                    });
                }

                // Extract references from markdown files
                if extension == "md" {
                    if let Ok(content) = std::str::from_utf8(&blob.content) {
                        let references = extract_markdown_references(content);
                        for reference in references {
                            graph.add_edge(Edge {
                                from: full_path.clone(),
                                to: reference,
                                kind: EdgeKind::References,
                            });
                        }
                    }
                }
            }
            TreeEntry::Tree(subtree_oid) => {
                let mut attributes = BTreeMap::new();
                attributes.insert("name".to_string(), name.clone());

                graph.add_node(Node {
                    id: full_path.clone(),
                    node_type: "directory".to_string(),
                    attributes,
                });

                // Add containment edge from parent directory
                if !path_prefix.is_empty() {
                    graph.add_edge(Edge {
                        from: path_prefix.to_string(),
                        to: full_path.clone(),
                        kind: EdgeKind::Custom("contains".to_string()),
                    });
                }

                // Recurse into subtree
                Box::pin(walk_tree(sandbox, subtree_oid, &full_path, graph)).await?;
            }
        }
    }

    Ok(())
}

/// Extract references from markdown content.
/// Finds both standard markdown links `[text](path)` and wikilinks `[[path]]`.
fn extract_markdown_references(content: &str) -> Vec<String> {
    let mut refs = Vec::new();

    // Standard markdown links: [text](path)
    let mut chars = content.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '[' {
            // Skip the link text portion
            let mut depth = 1;
            while let Some(inner) = chars.next() {
                if inner == '[' {
                    depth += 1;
                } else if inner == ']' {
                    depth -= 1;
                    if depth == 0 {
                        break;
                    }
                }
            }
            // Check for `](`
            if chars.peek() == Some(&'(') {
                chars.next(); // consume '('
                let mut link = String::new();
                for link_ch in chars.by_ref() {
                    if link_ch == ')' {
                        break;
                    }
                    link.push(link_ch);
                }
                let trimmed = link.trim();
                // Only include relative paths, not URLs
                if !trimmed.is_empty()
                    && !trimmed.starts_with("http://")
                    && !trimmed.starts_with("https://")
                    && !trimmed.starts_with('#')
                {
                    // Strip file:/// prefix if present
                    let reference = if let Some(stripped) = trimmed.strip_prefix("file:///") {
                        stripped.to_string()
                    } else {
                        trimmed.to_string()
                    };
                    refs.push(reference);
                }
            } else if chars.peek() == Some(&'[') {
                // This might be double-bracket wikilink start handled below
            }
        }
    }

    // Wikilinks: [[target]]
    for part in content.split("[[").skip(1) {
        if let Some(end) = part.find("]]") {
            let target = part[..end].trim();
            if !target.is_empty() {
                // Wikilinks reference by name rather than path
                refs.push(target.to_string());
            }
        }
    }

    refs
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory_fs::sandbox::FsEntry;
    use crate::memory_fs::test_support::make_sandbox_with_entries;

    #[test]
    fn materializer_creates_file_and_directory_nodes() {
        let entries = vec![
            FsEntry::File {
                name: "README.md".to_string(),
                content: b"# Hello".to_vec(),
            },
            FsEntry::Directory {
                name: "src".to_string(),
                children: vec![FsEntry::File {
                    name: "main.rs".to_string(),
                    content: b"fn main() {}".to_vec(),
                }],
            },
        ];

        let (_storage, _pool, sandbox, root_oid) = make_sandbox_with_entries(entries);

        futures::executor::block_on(async {
            let graph = FileGraphMaterializer
                .materialize(&sandbox, &root_oid)
                .await
                .unwrap();

            // Should have: README.md (file), src (dir), src/main.rs (file)
            assert_eq!(graph.nodes.len(), 3);

            let readme = graph.nodes.get("README.md").unwrap();
            assert_eq!(readme.node_type, "file");
            assert_eq!(readme.attributes.get("extension").unwrap(), "md");

            let src = graph.nodes.get("src").unwrap();
            assert_eq!(src.node_type, "directory");

            let main = graph.nodes.get("src/main.rs").unwrap();
            assert_eq!(main.node_type, "file");
            assert_eq!(main.attributes.get("extension").unwrap(), "rs");

            // Check containment edge
            let contains_edges: Vec<_> = graph
                .edges
                .iter()
                .filter(|e| e.kind == EdgeKind::Custom("contains".to_string()))
                .collect();
            assert_eq!(contains_edges.len(), 1); // src -> src/main.rs
            assert_eq!(contains_edges[0].from, "src");
            assert_eq!(contains_edges[0].to, "src/main.rs");
        });
    }

    #[test]
    fn materializer_extracts_markdown_links() {
        let md_content = b"# Links\nSee [plan](research/PLAN.md) and [[RESEARCH]].\nAlso [external](https://example.com) should be ignored.";

        let entries = vec![FsEntry::File {
            name: "INDEX.md".to_string(),
            content: md_content.to_vec(),
        }];

        let (_storage, _pool, sandbox, root_oid) = make_sandbox_with_entries(entries);

        futures::executor::block_on(async {
            let graph = FileGraphMaterializer
                .materialize(&sandbox, &root_oid)
                .await
                .unwrap();

            let ref_edges: Vec<_> = graph
                .edges
                .iter()
                .filter(|e| e.kind == EdgeKind::References)
                .collect();

            // Should find "research/PLAN.md" and "RESEARCH" but NOT "https://example.com"
            assert_eq!(ref_edges.len(), 2);
            let targets: Vec<&str> = ref_edges.iter().map(|e| e.to.as_str()).collect();
            assert!(targets.contains(&"research/PLAN.md"));
            assert!(targets.contains(&"RESEARCH"));
        });
    }

    #[test]
    fn extract_references_handles_edge_cases() {
        let refs = extract_markdown_references("");
        assert!(refs.is_empty());

        let refs = extract_markdown_references("No links here");
        assert!(refs.is_empty());

        let refs = extract_markdown_references("[text](#anchor)");
        assert!(refs.is_empty()); // anchors are excluded

        let refs = extract_markdown_references("[a](foo.md) [b](bar.md)");
        assert_eq!(refs.len(), 2);
    }
}
