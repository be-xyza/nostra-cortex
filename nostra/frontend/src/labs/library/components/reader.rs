use crate::labs::library::data::{
    dpub_path_for_id, edition_manifest_path_for_id, edition_snapshot_manifest_path_for_id,
    edition_snapshot_path_for_id, editions_dir_for_id, feed_path_for_id,
};
use crate::labs::library::seed::dpub_dir_for_id;
use crate::labs::library::types::{
    Attribution, Block, Book, BookContent, ChapterManifest, ContentValue, ContributionVersionRef,
    EditionManifest, EditionMetadata, LibraryView, Note, RichTextSpan,
};
use crate::services::vfs_service::{NodeType, VfsService};
use dioxus::prelude::*;
use js_sys::{Array, Uint8Array};
use sha2::{Digest, Sha256};
use std::collections::HashSet;
use std::io::{Cursor, Write};
use uuid::Uuid;
use wasm_bindgen::JsCast;
use web_sys::{Blob, BlobPropertyBag, HtmlAnchorElement, Url};
use zip::write::FileOptions;
use crate::api;

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
struct ReaderSnapshotManifestFile {
    path: String,
    sha256: String,
    size_bytes: u64,
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
struct ReaderSnapshotManifestEntrypoints {
    dpub_path: String,
    edition_manifest_path: String,
    snapshot_path: String,
    snapshot_manifest_path: String,
    #[serde(default)]
    feed_path: Option<String>,
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
struct ReaderSnapshotManifest {
    bundle_id: String,
    commit_hash: String,
    generated_at: String,
    bundle_version: String,
    files: Vec<ReaderSnapshotManifestFile>,
    entrypoints: ReaderSnapshotManifestEntrypoints,
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hex::encode(hasher.finalize())
}

fn merkle_root_hex(leaf_hashes_hex: &[String]) -> String {
    if leaf_hashes_hex.is_empty() {
        return sha256_hex(b"");
    }
    if leaf_hashes_hex.len() == 1 {
        return leaf_hashes_hex[0].clone();
    }

    let mut level: Vec<String> = leaf_hashes_hex.to_vec();
    while level.len() > 1 {
        let mut next = Vec::with_capacity((level.len() + 1) / 2);
        for chunk in level.chunks(2) {
            let combined = if chunk.len() == 2 {
                format!("{}{}", chunk[0], chunk[1])
            } else {
                chunk[0].clone()
            };
            next.push(sha256_hex(combined.as_bytes()));
        }
        level = next;
    }
    level[0].clone()
}

fn sanitize_html(input: &str) -> String {
    ammonia::Builder::default().clean(input).to_string()
}

fn short_hash(value: &str, max_chars: usize) -> String {
    if value.chars().count() <= max_chars {
        value.to_string()
    } else {
        value.chars().take(max_chars).collect()
    }
}

fn extract_editions(feed: &serde_json::Value) -> Vec<serde_json::Value> {
    if let Some(editions) = feed.get("editions").and_then(|e| e.as_array()) {
        return editions.clone();
    }
    if let Some(items) = feed.get("items").and_then(|e| e.as_array()) {
        return items.clone();
    }
    Vec::new()
}

async fn fetch_feed_with_fallback(
    dpub_id: String,
    viewer_space: Option<String>,
    treaty: Option<String>,
    vfs: VfsService,
) -> Option<serde_json::Value> {
    let dpub_dir = dpub_dir_for_id(&dpub_id);
    let agent = api::create_agent().await;
    api::get_dpub_feed(&agent, dpub_dir, 20, viewer_space, treaty)
        .await
        .ok()
        .and_then(|json| serde_json::from_str::<serde_json::Value>(&json).ok())
        .or_else(|| {
            let feed_path = feed_path_for_id(&dpub_id);
            vfs.read_file_bytes(&feed_path)
                .and_then(|bytes| serde_json::from_slice::<serde_json::Value>(&bytes).ok())
        })
}

fn ordered_content_from_manifest(book: &Book) -> Vec<BookContent> {
    let mut ordered: Vec<BookContent> = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();

    if let Some(manifest) = book.manifest.as_ref() {
        for node in manifest.chapters.iter() {
            if let Some(ch) = book.content.iter().find(|c| c.id == node.id) {
                ordered.push(ch.clone());
                seen.insert(ch.id.clone());
            }
        }
    }

    for ch in book.content.iter() {
        if !seen.contains(&ch.id) {
            ordered.push(ch.clone());
        }
    }

    ordered
}

fn resolve_publish_context(
    commit_hash_input: &str,
    source_ref_input: &str,
    fallback_source_ref: String,
) -> (api::DpubPublishContextArg, bool) {
    let commit_hash = commit_hash_input.trim().to_string();
    let source_ref = source_ref_input.trim().to_string();
    let used_fallback_commit = commit_hash.is_empty();
    (
        api::DpubPublishContextArg {
            commit_hash: if used_fallback_commit {
                "unknown-local".to_string()
            } else {
                commit_hash
            },
            source_ref: if source_ref.is_empty() {
                Some(fallback_source_ref)
            } else {
                Some(source_ref)
            },
        },
        used_fallback_commit,
    )
}

fn collect_file_paths(vfs: &VfsService, dir: &str, out: &mut Vec<String>) {
    let dir = dir.trim_end_matches('/').to_string();
    for node in vfs.list_dir(&dir) {
        let path = format!("{}/{}", dir, node.name);
        match node.node_type {
            NodeType::Directory => collect_file_paths(vfs, &path, out),
            NodeType::File { .. } => out.push(path),
        }
    }
}

fn download_bytes(filename: &str, mime: &str, bytes: Vec<u8>) -> Result<(), String> {
    let u8_array = Uint8Array::from(bytes.as_slice());
    let parts = Array::new();
    parts.push(&u8_array.into());

    let opts = BlobPropertyBag::new();
    opts.set_type(mime);
    let blob =
        Blob::new_with_u8_array_sequence_and_options(&parts, &opts).map_err(|e| format!("{e:?}"))?;
    let url = Url::create_object_url_with_blob(&blob).map_err(|e| format!("{e:?}"))?;

    let window = web_sys::window().ok_or_else(|| "Missing window".to_string())?;
    let document = window
        .document()
        .ok_or_else(|| "Missing document".to_string())?;
    let a: HtmlAnchorElement = document
        .create_element("a")
        .map_err(|e| format!("{e:?}"))?
        .dyn_into()
        .map_err(|_| "Failed to cast to HtmlAnchorElement".to_string())?;
    a.set_href(&url);
    a.set_download(filename);
    a.click();
    let _ = Url::revoke_object_url(&url);
    Ok(())
}

fn zip_vfs_dir(vfs: &VfsService, dir: &str, zip_root: &str) -> Result<Vec<u8>, String> {
    let mut file_paths: Vec<String> = Vec::new();
    collect_file_paths(vfs, dir, &mut file_paths);

    let mut cursor = Cursor::new(Vec::<u8>::new());
    let mut zip = zip::ZipWriter::new(&mut cursor);
    let options =
        FileOptions::<()>::default().compression_method(zip::CompressionMethod::Deflated);

    let prefix = format!("{}/", dir.trim_end_matches('/'));
    for path in file_paths.into_iter() {
        let Some(bytes) = vfs.read_file_bytes(&path) else {
            continue;
        };
        let rel = path.strip_prefix(&prefix).unwrap_or(path.as_str());
        let name = format!("{}/{}", zip_root.trim_end_matches('/'), rel);
        zip.start_file(name, options)
            .map_err(|e| format!("zip start_file error: {e}"))?;
        zip.write_all(&bytes)
            .map_err(|e| format!("zip write error: {e}"))?;
    }

    zip.finish()
        .map_err(|e| format!("zip finish error: {e}"))?;
    Ok(cursor.into_inner())
}

#[component]
pub fn Reader(
    book: Book,
    current_view: Signal<LibraryView>,
    show_cortex: Signal<bool>,
    show_raw: Signal<bool>,
    show_note_panel: Signal<bool>,
    notes: Signal<Vec<Note>>,
    active_note_content: Signal<String>,
    reading_progress: Signal<f64>,
    access_blocked: bool,
    viewer_space_did: Option<String>,
    treaty_token: Option<String>,
) -> Element {
    let vfs = use_context::<VfsService>();
    let dpub_id = book.meta.id.clone();
    let dpub_title = book.meta.title.clone();
    let book_for_local_publish = book.clone();
    let mut current_view = current_view;
    let dpub_id_for_canister_publish = dpub_id.clone();
    let dpub_id_for_export = dpub_id.clone();
    let access_blocked = access_blocked;
    let viewer_space_did = viewer_space_did.filter(|v| !v.trim().is_empty());
    let viewer_space_label = viewer_space_did
        .clone()
        .unwrap_or_else(|| "unspecified".to_string());
    let treaty_token = treaty_token.filter(|t| !t.trim().is_empty());

    let mut selected_edition = use_signal(|| Option::<String>::None);
    let mut edition_snapshot = use_signal(|| Option::<Book>::None);
    let mut edition_manifest = use_signal(|| Option::<EditionManifest>::None);
    let mut edition_snapshot_manifest = use_signal(|| Option::<ReaderSnapshotManifest>::None);
    let mut edition_verified = use_signal(|| Option::<bool>::None);

    let mut show_publish = use_signal(|| false);
    let mut publish_version = use_signal(|| "1.0.0".to_string());
    let mut publish_name = use_signal(|| String::new());
    let mut publish_commit_hash = use_signal(|| String::new());
    let mut publish_source_ref = use_signal(|| String::new());
    let mut publish_override_token = use_signal(|| String::new());
    let mut publish_status = use_signal(|| Option::<String>::None);
    let mut export_status = use_signal(|| Option::<String>::None);
    let mut show_rss = use_signal(|| false);
    let feed_xml_kind = use_signal(|| "RSS".to_string());
    let rss_xml = use_signal(|| String::new());
    let mut audit_traces = use_signal(|| Vec::<serde_json::Value>::new());

    {
        let vfs = vfs;
        let dpub_id = dpub_id.clone();
        use_effect(move || {
            let mut traces: Vec<serde_json::Value> = Vec::new();
            for node in vfs.list_dir("lib/audit_traces") {
                if !matches!(node.node_type, NodeType::File { .. }) {
                    continue;
                }
                let path = format!("lib/audit_traces/{}", node.name);
                if let Some(bytes) = vfs.read_file_bytes(&path) {
                    if let Ok(value) = serde_json::from_slice::<serde_json::Value>(&bytes) {
                        if value
                            .get("dpub_id")
                            .and_then(|v| v.as_str())
                            .map(|id| id == dpub_id)
                            .unwrap_or(false)
                        {
                            traces.push(value);
                        }
                    }
                }
            }
            traces.sort_by(|a, b| {
                let ta = a
                    .get("timestamp")
                    .and_then(|v| v.as_str())
                    .and_then(|s| s.parse::<u128>().ok())
                    .unwrap_or(0);
                let tb = b
                    .get("timestamp")
                    .and_then(|v| v.as_str())
                    .and_then(|s| s.parse::<u128>().ok())
                    .unwrap_or(0);
                tb.cmp(&ta)
            });
            audit_traces.set(traces);
        });
    }

    // Refresh edition snapshot/manifest when selection changes.
    {
        let vfs = vfs;
        let dpub_id = dpub_id.clone();
        use_effect(move || {
            if let Some(ver) = selected_edition.read().as_ref() {
                let manifest_path = edition_manifest_path_for_id(&dpub_id, ver);
                let snapshot_path = edition_snapshot_path_for_id(&dpub_id, ver);
                let snapshot_manifest_path = edition_snapshot_manifest_path_for_id(&dpub_id, ver);

                let manifest = vfs
                    .read_file_bytes(&manifest_path)
                    .and_then(|b| serde_json::from_slice::<EditionManifest>(&b).ok());
                let snapshot = vfs
                    .read_file_bytes(&snapshot_path)
                    .and_then(|b| serde_json::from_slice::<Book>(&b).ok());
                let snapshot_manifest = vfs
                    .read_file_bytes(&snapshot_manifest_path)
                    .and_then(|b| serde_json::from_slice::<ReaderSnapshotManifest>(&b).ok());

                edition_manifest.set(manifest.clone());
                edition_snapshot.set(snapshot.clone());
                edition_snapshot_manifest.set(snapshot_manifest);

                if let (Some(m), Some(s)) = (manifest, snapshot) {
                    let leaf_hashes: Vec<String> = s
                        .content
                        .iter()
                        .map(|ch| sha256_hex(serde_json::to_vec(&ch.blocks).unwrap_or_default().as_slice()))
                        .collect();
                    edition_verified.set(Some(merkle_root_hex(&leaf_hashes) == m.content_root));
                } else {
                    edition_verified.set(None);
                }
            } else {
                edition_manifest.set(None);
                edition_snapshot.set(None);
                edition_snapshot_manifest.set(None);
                edition_verified.set(None);
            }
        });
    }

    let displayed_book = edition_snapshot.read().clone().unwrap_or(book.clone());

    let editions_on_disk = {
        let mut out: Vec<String> = vec![];
        for node in vfs.list_dir(&editions_dir_for_id(&dpub_id)) {
            if node.node_type == NodeType::Directory {
                out.push(node.name);
            }
        }
        out.sort();
        out
    };

    let publish_local = move |_| {
        if access_blocked {
            publish_status.set(Some("Restricted: treaty required".to_string()));
            return;
        }
        let version = publish_version.read().trim().to_string();
        if !regex::Regex::new(r"^\\d+\\.\\d+\\.\\d+$")
            .unwrap()
            .is_match(&version)
        {
            publish_status.set(Some("Invalid SemVer (expected X.Y.Z)".to_string()));
            return;
        }

        let license = book_for_local_publish.meta.license.clone().unwrap_or_default();
        if license.trim().is_empty() {
            publish_status.set(Some("Missing license".to_string()));
            return;
        }

        let override_token = publish_override_token.read().trim().to_string();
        if license.to_lowercase().contains("arranged") && override_token.is_empty() {
            publish_status.set(Some(
                "License requires an explicit override token (Arranged)".to_string(),
            ));
            return;
        }

        publish_status.set(Some("Publishing locally...".to_string()));

        let dpub_id = book_for_local_publish.meta.id.clone();
        let dpub_path = dpub_path_for_id(&dpub_id);
        let (publish_context, used_fallback_commit) = resolve_publish_context(
            publish_commit_hash.read().as_str(),
            publish_source_ref.read().as_str(),
            dpub_path.clone(),
        );
        let edition_id = Uuid::new_v4().to_string();
        let published_at = chrono::Utc::now().to_rfc3339();
        let ordered_content = ordered_content_from_manifest(&book_for_local_publish);

        let mut chapter_manifests: Vec<ChapterManifest> = Vec::new();
        let mut leaf_hashes: Vec<String> = Vec::new();
        for (i, ch) in ordered_content.iter().enumerate() {
            let content_bytes = serde_json::to_vec(&ch.blocks).unwrap_or_default();
            let content_hash = sha256_hex(&content_bytes);
            leaf_hashes.push(content_hash.clone());
            chapter_manifests.push(ChapterManifest {
                index: i as u32,
                contribution_ref: ContributionVersionRef {
                    contribution_id: ch.id.clone(),
                    version_hash: content_hash.clone(),
                },
                content_hash,
                title: ch.title.clone().unwrap_or_else(|| ch.id.clone()),
            });
        }

        let content_root = merkle_root_hex(&leaf_hashes);
        let manifest = EditionManifest {
            edition_id: edition_id.clone(),
            dpub_id: dpub_id.clone(),
            version: version.clone(),
            name: if publish_name.read().trim().is_empty() {
                None
            } else {
                Some(publish_name.read().trim().to_string())
            },
            content_root: content_root.clone(),
            chapters: chapter_manifests,
            published_at: published_at.clone(),
            publisher: "local".to_string(),
            previous_edition: book_for_local_publish.latest_edition.clone(),
            metadata: EditionMetadata {
                license: license.clone(),
                contributors: Vec::<Attribution>::new(),
            },
        };

        let mut snapshot = book_for_local_publish.clone();
        snapshot.content = ordered_content;
        snapshot.meta.version = Some(version.clone());
        snapshot.meta.phase = Some("Archival".to_string());

        let manifest_path = edition_manifest_path_for_id(&dpub_id, &version);
        let snapshot_path = edition_snapshot_path_for_id(&dpub_id, &version);
        let snapshot_manifest_path = edition_snapshot_manifest_path_for_id(&dpub_id, &version);

        let manifest_bytes = match serde_json::to_vec(&manifest) {
            Ok(bytes) => bytes,
            Err(e) => {
                publish_status.set(Some(format!("Error: {}", e)));
                return;
            }
        };
        let snapshot_bytes = match serde_json::to_vec(&snapshot) {
            Ok(bytes) => bytes,
            Err(e) => {
                publish_status.set(Some(format!("Error: {}", e)));
                return;
            }
        };

        let mut vfs_mut = vfs;
        if let Err(e) = vfs_mut.write_file(
            &manifest_path,
            manifest_bytes.clone(),
            "application/json",
        ) {
            publish_status.set(Some(format!("Error: {}", e)));
            return;
        }
        if let Err(e) = vfs_mut.write_file(
            &snapshot_path,
            snapshot_bytes.clone(),
            "application/json",
        ) {
            publish_status.set(Some(format!("Error: {}", e)));
            return;
        }

        // Update dpub file metadata (latest edition) locally.
        let mut updated = book_for_local_publish.clone();
        updated.latest_edition = Some(version.clone());
        if !updated.editions.iter().any(|e| e.version == version) {
            updated.editions.push(crate::labs::library::types::EditionSummary {
                edition_id,
                version: version.clone(),
                published_at: published_at.clone(),
                content_root: content_root.clone(),
            });
        }
        let dpub_updated_bytes = match serde_json::to_vec(&updated) {
            Ok(bytes) => bytes,
            Err(e) => {
                publish_status.set(Some(format!("Error: {}", e)));
                return;
            }
        };
        if let Err(e) = vfs_mut.write_file(&dpub_path, dpub_updated_bytes.clone(), "application/json")
        {
            publish_status.set(Some(format!("Error: {}", e)));
            return;
        }

        // Native feed (local canonical JSON feed).
        let feed_path = feed_path_for_id(&updated.meta.id);
        let feed = serde_json::json!({
            "type": "dpub.feed.v1",
            "dpub_id": updated.meta.id,
            "latest_edition": updated.latest_edition,
            "editions": updated.editions,
        });
        let feed_bytes = match serde_json::to_vec(&feed) {
            Ok(bytes) => bytes,
            Err(e) => {
                publish_status.set(Some(format!("Error: {}", e)));
                return;
            }
        };
        if let Err(e) = vfs_mut.write_file(&feed_path, feed_bytes.clone(), "application/json") {
            publish_status.set(Some(format!("Error: {}", e)));
            return;
        }

        let mut files = vec![
            ReaderSnapshotManifestFile {
                path: manifest_path.clone(),
                sha256: sha256_hex(&manifest_bytes),
                size_bytes: manifest_bytes.len() as u64,
            },
            ReaderSnapshotManifestFile {
                path: snapshot_path.clone(),
                sha256: sha256_hex(&snapshot_bytes),
                size_bytes: snapshot_bytes.len() as u64,
            },
            ReaderSnapshotManifestFile {
                path: dpub_path.clone(),
                sha256: sha256_hex(&dpub_updated_bytes),
                size_bytes: dpub_updated_bytes.len() as u64,
            },
            ReaderSnapshotManifestFile {
                path: feed_path.clone(),
                sha256: sha256_hex(&feed_bytes),
                size_bytes: feed_bytes.len() as u64,
            },
        ];
        files.sort_by(|a, b| a.path.cmp(&b.path));

        let snapshot_manifest = ReaderSnapshotManifest {
            bundle_id: format!("{}@{}", dpub_id, version),
            commit_hash: publish_context.commit_hash.clone(),
            generated_at: published_at.clone(),
            bundle_version: "1.0.0".to_string(),
            files,
            entrypoints: ReaderSnapshotManifestEntrypoints {
                dpub_path: dpub_path.clone(),
                edition_manifest_path: manifest_path.clone(),
                snapshot_path: snapshot_path.clone(),
                snapshot_manifest_path: snapshot_manifest_path.clone(),
                feed_path: Some(feed_path.clone()),
            },
        };
        let snapshot_manifest_bytes = match serde_json::to_vec(&snapshot_manifest) {
            Ok(bytes) => bytes,
            Err(e) => {
                publish_status.set(Some(format!("Error: {}", e)));
                return;
            }
        };
        if let Err(e) = vfs_mut.write_file(
            &snapshot_manifest_path,
            snapshot_manifest_bytes,
            "application/json",
        ) {
            publish_status.set(Some(format!("Error: {}", e)));
            return;
        }

        // Chronicle append (local).
        let event_path = "/lib/chronicle/edition_published.jsonl";
        let mut existing = vfs_mut.read_file_string(event_path).unwrap_or_default();
        let event = serde_json::json!({
            "type": "edition.published",
            "dpub_id": dpub_id,
            "edition_id": manifest.edition_id,
            "version": version,
            "content_root": content_root,
            "published_at": published_at,
            "license": license,
        });
        existing.push_str(&serde_json::to_string(&event).unwrap_or_else(|_| "{}".to_string()));
        existing.push('\n');
        let _ = vfs_mut.write_file(event_path, existing.as_bytes().to_vec(), "application/jsonl");

        // Audit trace (local, glass box).
        let trace_path = format!(
            "/lib/audit_traces/publish_edition_{}.json",
            manifest.edition_id
        );
        let trace = serde_json::json!({
            "type": "audit_trace.v1",
            "action": "publish_edition",
            "dpub_id": updated.meta.id,
            "edition_id": manifest.edition_id,
            "edition_version": updated.latest_edition,
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "inputs": {
                "license": updated.meta.license,
                "override_token": if override_token.is_empty() { serde_json::Value::Null } else { serde_json::Value::String(override_token.clone()) },
                "chapter_count": updated.content.len(),
                "commit_hash": publish_context.commit_hash,
                "source_ref": publish_context.source_ref,
            },
            "outputs": {
                "content_root": manifest.content_root,
                "manifest_path": manifest_path,
                "snapshot_path": snapshot_path,
                "snapshot_manifest_path": snapshot_manifest_path,
            }
        });
        let _ = vfs_mut.write_file(
            &trace_path,
            serde_json::to_vec(&trace).unwrap_or_default(),
            "application/json",
        );

        if used_fallback_commit {
            publish_status.set(Some(
                "Published locally (warning: commit hash fallback to unknown-local)".to_string(),
            ));
        } else {
            publish_status.set(Some("Published locally".to_string()));
        }
        show_publish.set(false);
    };

    let viewer_space_did_for_publish = viewer_space_did.clone();
    let treaty_token_for_publish = treaty_token.clone();
    let publish_canister = move |_| {
        if access_blocked {
            publish_status.set(Some("Restricted: treaty required".to_string()));
            return;
        }
        publish_status.set(Some("Publishing via canister...".to_string()));
        let dpub_id = dpub_id_for_canister_publish.clone();
        let dpub_path = dpub_path_for_id(&dpub_id);
        let version = publish_version.read().trim().to_string();
        let name = if publish_name.read().trim().is_empty() {
            None
        } else {
            Some(publish_name.read().trim().to_string())
        };
        let override_token = {
            let t = publish_override_token.read().trim().to_string();
            if t.is_empty() { None } else { Some(t) }
        };
        let (publish_context, used_fallback_commit) = resolve_publish_context(
            publish_commit_hash.read().as_str(),
            publish_source_ref.read().as_str(),
            dpub_path.clone(),
        );
        let vfs = vfs;
        let mut selected_edition = selected_edition;
        let mut show_publish = show_publish;
        let mut publish_status = publish_status;
        let viewer_space = viewer_space_did_for_publish.clone();
        let treaty = treaty_token_for_publish.clone();

        spawn(async move {
            let agent = api::create_agent().await;

            // Sync the dpub file so the workflow-engine can read it.
            if let Some(bytes) = vfs.read_file_bytes(&dpub_path) {
                if let Err(e) = api::write_file(&agent, dpub_path.clone(), bytes, "application/json".to_string()).await {
                    publish_status.set(Some(format!("Sync error: {}", e)));
                    return;
                }
            } else {
                publish_status.set(Some("Missing local dPub file".to_string()));
                return;
            }

            match api::publish_dpub_edition_v2(
                &agent,
                dpub_path.clone(),
                version.clone(),
                name,
                override_token,
                publish_context,
            )
            .await
            {
                Ok(_) => {
                    // Pull new edition files back to local VFS (missing-only sync is fine for new editions).
                    let prefix = editions_dir_for_id(&dpub_id);
                    let sync_result = vfs
                        .sync_dpub_from_backend_guarded(
                            &agent,
                            &prefix,
                            viewer_space.clone(),
                            treaty.clone(),
                        )
                        .await;
                    if let Err(e) = sync_result {
                        if used_fallback_commit {
                            publish_status.set(Some(format!(
                                "Published via canister; sync blocked: {} (warning: commit hash fallback to unknown-local)",
                                e
                            )));
                        } else {
                            publish_status.set(Some(format!(
                                "Published via canister; sync blocked: {}",
                                e
                            )));
                        }
                    } else {
                        if used_fallback_commit {
                            publish_status.set(Some(
                                "Published via canister (warning: commit hash fallback to unknown-local)"
                                    .to_string(),
                            ));
                        } else {
                            publish_status.set(Some("Published via canister".to_string()));
                        }
                    }
                    selected_edition.set(Some(version));
                    show_publish.set(false);
                }
                Err(e) => {
                    publish_status.set(Some(format!("Publish error: {}", e)));
                }
            }
        });
    };

    let dpub_id_for_export_zip = dpub_id_for_export.clone();
    let export_zip = move |_| {
        if access_blocked {
            export_status.set(Some("Restricted: treaty required".to_string()));
            return;
        }
        let mut export_status = export_status;
        export_status.set(Some("Exporting ZIP...".to_string()));

        let dir = dpub_dir_for_id(&dpub_id_for_export_zip);
        let zip_root = dir
            .split('/')
            .filter(|s| !s.is_empty())
            .last()
            .unwrap_or("dpub")
            .to_string();

        match zip_vfs_dir(&vfs, &dir, &zip_root) {
            Ok(bytes) => {
                let filename = format!("dpub_{}.zip", zip_root);
                if let Err(e) = download_bytes(&filename, "application/zip", bytes) {
                    export_status.set(Some(format!("Export error: {}", e)));
                } else {
                    export_status.set(Some("Exported".to_string()));
                }
            }
            Err(e) => export_status.set(Some(format!("Export error: {}", e))),
        }
    };

    let viewer_space_did_for_rss = viewer_space_did.clone();
    let treaty_token_for_rss = treaty_token.clone();
    let viewer_space_did_for_atom = viewer_space_did.clone();
    let treaty_token_for_atom = treaty_token.clone();

    rsx! {
        div { class: "relative flex flex-col h-full",
            // Reader Header
            div { class: "h-14 border-b border-border bg-card/50 flex items-center justify-between px-4 shrink-0",
                div { class: "flex items-center gap-4",
                    button {
                        class: "flex items-center gap-1 text-sm font-medium text-muted-foreground hover:text-foreground hover:bg-muted/50 px-3 py-1.5 rounded-md transition-colors",
                        onclick: move |_| current_view.set(LibraryView::Bookshelf),
                        span { "←" }
                        "Back to Library"
                    }
                    div { class: "h-4 w-px bg-border mx-2" }
                    h2 { class: "font-semibold text-sm", "{book.meta.title}" }
                }

                div { class: "flex items-center gap-3",
                    // Mode
                    div { class: "flex items-center gap-2 text-xs font-medium",
                        span { class: "text-muted-foreground", "Mode" }
                        select {
                            class: "bg-muted/50 border border-input rounded-md py-1 pl-2 pr-6 text-xs focus:outline-none focus:ring-2 focus:ring-ring cursor-pointer hover:bg-muted transition-colors",
                            onchange: move |evt| {
                                let v = evt.value();
                                if v == "draft" { selected_edition.set(None); }
                                else { selected_edition.set(Some(v)); }
                            },
                            option { value: "draft", selected: selected_edition.read().is_none(), "Draft" }
                            for ver in editions_on_disk.iter() {
                                option { value: "{ver}", selected: selected_edition.read().as_deref() == Some(ver.as_str()), "Edition {ver}" }
                            }
                        }
                        if let Some(verified) = edition_verified.read().as_ref() {
                            span { class: if *verified { "text-green-400" } else { "text-red-400" },
                                if *verified { "Verified" } else { "Mismatch" }
                            }
                        }
                        if let Some(snapshot_manifest) = edition_snapshot_manifest.read().as_ref() {
                            span { class: "text-[10px] text-muted-foreground font-mono",
                                "snapshot_manifest: "
                                "{short_hash(snapshot_manifest.commit_hash.as_str(), 12)}"
                            }
                        }
                    }

                    button {
                        class: "px-3 py-1.5 bg-muted/30 hover:bg-muted/50 border border-border rounded-lg text-[10px] font-bold uppercase tracking-widest text-muted-foreground transition-all",
                        onclick: {
                            let dpub_id = dpub_id.clone();
                            move |_| current_view.set(LibraryView::ManifestEditor(dpub_id.clone()))
                        },
                        "Edit Manifest"
                    }
                    button {
                        class: "px-3 py-1.5 bg-primary/15 hover:bg-primary/25 border border-primary/30 rounded-lg text-[10px] font-bold uppercase tracking-widest text-primary transition-all",
                        onclick: move |_| {
                            if access_blocked {
                                publish_status.set(Some("Restricted: treaty required".to_string()));
                                return;
                            }
                            show_publish.set(true)
                        },
                        "Publish Edition"
                    }
                    button {
                        class: "px-3 py-1.5 bg-muted/30 hover:bg-muted/50 border border-border rounded-lg text-[10px] font-bold uppercase tracking-widest text-muted-foreground transition-all",
                        onclick: {
                            let dpub_id = dpub_id.clone();
                            let dpub_title = dpub_title.clone();
                            let mut feed_xml_kind = feed_xml_kind;
                            move |_| {
                                if access_blocked {
                                    export_status.set(Some("Restricted: treaty required".to_string()));
                                    return;
                                }
                                feed_xml_kind.set("RSS".to_string());
                                let title = dpub_title.clone();
                                let dpub_id = dpub_id.clone();
                                let mut show_rss = show_rss;
                                let mut rss_xml = rss_xml;
                                let viewer_space = viewer_space_did_for_rss.clone();
                                let treaty = treaty_token_for_rss.clone();
                                let vfs = vfs;
                                spawn(async move {
                                    let feed_opt = fetch_feed_with_fallback(dpub_id.clone(), viewer_space, treaty, vfs).await;
                                    if let Some(feed) = feed_opt {
                                        let editions = extract_editions(&feed);
                                        let mut items = String::new();
                                        for e in editions.iter().rev().take(20) {
                                            let ver = e.get("version").and_then(|v| v.as_str()).unwrap_or("unknown");
                                            let published_at = e.get("published_at").and_then(|v| v.as_str()).unwrap_or("");
                                            let content_root = e.get("content_root").and_then(|v| v.as_str()).unwrap_or("");
                                            items.push_str(&format!(
                                                "<item><title>{}</title><guid>{}</guid><pubDate>{}</pubDate><description>{}</description></item>",
                                                ammonia::clean(&format!("Edition {}", ver)),
                                                ammonia::clean(&format!("{}@{}", dpub_id, ver)),
                                                ammonia::clean(published_at),
                                                ammonia::clean(content_root),
                                            ));
                                        }
                                        let xml = format!(
                                            "<?xml version=\"1.0\" encoding=\"UTF-8\"?><rss version=\"2.0\"><channel><title>{}</title><description>{}</description><link>{}</link>{}</channel></rss>",
                                            ammonia::clean(&title),
                                            ammonia::clean("DPub feed (V1)"),
                                            ammonia::clean(&dpub_id),
                                            items
                                        );
                                        rss_xml.set(xml);
                                        show_rss.set(true);
                                    }
                                });
                            }
                        },
                        "RSS"
                    }
                    button {
                        class: "px-3 py-1.5 bg-muted/30 hover:bg-muted/50 border border-border rounded-lg text-[10px] font-bold uppercase tracking-widest text-muted-foreground transition-all",
                        onclick: {
                            let dpub_id = dpub_id.clone();
                            let dpub_title = dpub_title.clone();
                            let mut feed_xml_kind = feed_xml_kind;
                            move |_| {
                                if access_blocked {
                                    export_status.set(Some("Restricted: treaty required".to_string()));
                                    return;
                                }
                                feed_xml_kind.set("Atom".to_string());
                                let title = dpub_title.clone();
                                let dpub_id = dpub_id.clone();
                                let mut show_rss = show_rss;
                                let mut rss_xml = rss_xml;
                                let viewer_space = viewer_space_did_for_atom.clone();
                                let treaty = treaty_token_for_atom.clone();
                                let vfs = vfs;
                                spawn(async move {
                                    let feed_opt = fetch_feed_with_fallback(dpub_id.clone(), viewer_space, treaty, vfs).await;
                                    if let Some(feed) = feed_opt {
                                        let editions = extract_editions(&feed);
                                        let mut entries = String::new();
                                        for e in editions.iter().rev().take(20) {
                                            let ver = e.get("version").and_then(|v| v.as_str()).unwrap_or("unknown");
                                            let published_at = e.get("published_at").and_then(|v| v.as_str()).unwrap_or("");
                                            let content_root = e.get("content_root").and_then(|v| v.as_str()).unwrap_or("");
                                            entries.push_str(&format!(
                                                "<entry><title>{}</title><id>{}</id><updated>{}</updated><summary>{}</summary></entry>",
                                                ammonia::clean(&format!("Edition {}", ver)),
                                                ammonia::clean(&format!("{}@{}", dpub_id, ver)),
                                                ammonia::clean(published_at),
                                                ammonia::clean(content_root),
                                            ));
                                        }
                                        let updated = editions
                                            .first()
                                            .and_then(|e| e.get("published_at"))
                                            .and_then(|v| v.as_str())
                                            .unwrap_or("");
                                        let xml = format!(
                                            "<?xml version=\"1.0\" encoding=\"UTF-8\"?><feed xmlns=\"http://www.w3.org/2005/Atom\"><title>{}</title><id>{}</id><updated>{}</updated>{}</feed>",
                                            ammonia::clean(&title),
                                            ammonia::clean(&dpub_id),
                                            ammonia::clean(updated),
                                            entries
                                        );
                                        rss_xml.set(xml);
                                        show_rss.set(true);
                                    }
                                });
                            }
                        },
                        "Atom"
                    }
                    button {
                        class: "px-3 py-1.5 bg-muted/30 hover:bg-muted/50 border border-border rounded-lg text-[10px] font-bold uppercase tracking-widest text-muted-foreground transition-all",
                        onclick: export_zip,
                        "Export ZIP"
                    }
                    if let Some(msg) = export_status.read().as_ref() {
                        span { class: "text-[10px] uppercase tracking-wider text-muted-foreground", "{msg}" }
                    }

                    // Cortex Mode Toggle
                    div { class: "flex items-center space-x-2 text-xs font-medium",
                        label { class: if *show_cortex.read() { "text-purple-400" } else { "text-muted-foreground" }, "Cortex Mode" }
                        button {
                            class: format!("w-8 h-4 rounded-full transition-colors relative {}", if *show_cortex.read() { "bg-purple-600" } else { "bg-muted" }),
                            onclick: move |_| show_cortex.toggle(),
                            div { class: format!("absolute top-0.5 w-3 h-3 rounded-full bg-white transition-all shadow-sm {}", if *show_cortex.read() { "left-4.5" } else { "left-0.5" }) }
                        }
                    }

                    // Raw Text Toggle
                    div { class: "flex items-center space-x-2 text-xs font-medium",
                        label { class: if *show_raw.read() { "text-primary" } else { "text-muted-foreground" }, "Raw Text" }
                        button {
                            class: format!("w-8 h-4 rounded-full transition-colors relative {}", if *show_raw.read() { "bg-primary" } else { "bg-muted" }),
                            onclick: move |_| show_raw.toggle(),
                            div { class: format!("absolute top-0.5 w-3 h-3 rounded-full bg-white transition-all shadow-sm {}", if *show_raw.read() { "left-4.5" } else { "left-0.5" }) }
                        }
                    }
                }
            }

            if *show_publish.read() {
                div { class: "absolute inset-0 z-50 bg-black/60 flex items-center justify-center p-4",
                    div { class: "w-full max-w-lg bg-card border border-border rounded-xl shadow-2xl p-4 space-y-4",
                        div { class: "flex items-center justify-between",
                            h3 { class: "text-sm font-bold uppercase tracking-widest text-muted-foreground", "Publish Edition" }
                            button {
                                class: "p-1 hover:bg-muted rounded text-muted-foreground",
                                onclick: move |_| show_publish.set(false),
                                "×"
                            }
                        }

                        div { class: "text-xs text-muted-foreground",
                            "This publishes a local immutable edition (Merkle root + snapshot + snapshot_manifest). Canister publishing uses v2 context."
                        }

                        div { class: "space-y-2",
                            label { class: "text-xs text-muted-foreground font-bold uppercase tracking-wider", "Version (SemVer)" }
                            input {
                                class: "w-full bg-background border border-input rounded-md px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-ring",
                                value: "{publish_version}",
                                oninput: move |evt| publish_version.set(evt.value()),
                                placeholder: "1.0.0"
                            }
                        }

                        div { class: "space-y-2",
                            label { class: "text-xs text-muted-foreground font-bold uppercase tracking-wider", "Commit Hash (Optional)" }
                            input {
                                class: "w-full bg-background border border-input rounded-md px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-ring font-mono",
                                value: "{publish_commit_hash}",
                                oninput: move |evt| publish_commit_hash.set(evt.value()),
                                placeholder: "git commit hash (fallback: unknown-local)"
                            }
                        }

                        div { class: "space-y-2",
                            label { class: "text-xs text-muted-foreground font-bold uppercase tracking-wider", "Source Ref (Optional)" }
                            input {
                                class: "w-full bg-background border border-input rounded-md px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-ring font-mono",
                                value: "{publish_source_ref}",
                                oninput: move |evt| publish_source_ref.set(evt.value()),
                                placeholder: "source path or URI"
                            }
                        }

		                        div { class: "space-y-2",
		                            label { class: "text-xs text-muted-foreground font-bold uppercase tracking-wider", "Edition Name (Optional)" }
		                            input {
	                                class: "w-full bg-background border border-input rounded-md px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-ring",
	                                value: "{publish_name}",
	                                oninput: move |evt| publish_name.set(evt.value()),
	                                placeholder: "First Edition"
	                            }
	                        }

	                        div { class: "space-y-2",
	                            label { class: "text-xs text-muted-foreground font-bold uppercase tracking-wider", "Override Token (Required for Arranged)" }
	                            input {
	                                class: "w-full bg-background border border-input rounded-md px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-ring font-mono",
	                                value: "{publish_override_token}",
	                                oninput: move |evt| publish_override_token.set(evt.value()),
	                                placeholder: "e.g. AGREED-123 or ACK"
	                            }
	                        }

                        if let Some(msg) = publish_status.read().as_ref() {
                            div { class: "text-xs text-muted-foreground", "{msg}" }
                        }

                        div { class: "flex items-center justify-end gap-2 pt-2",
                            button {
                                class: "px-3 py-2 rounded-md border border-border bg-muted/40 hover:bg-muted text-xs font-bold uppercase tracking-widest",
                                onclick: move |_| show_publish.set(false),
                                "Cancel"
                            }
                            button {
                                class: "px-3 py-2 rounded-md bg-primary text-primary-foreground text-xs font-bold uppercase tracking-widest hover:opacity-90",
                                onclick: publish_local,
                                "Publish (Local)"
                            }
                            button {
                                class: "px-3 py-2 rounded-md bg-blue-600 text-white text-xs font-bold uppercase tracking-widest hover:opacity-90",
                                onclick: publish_canister,
                                "Publish (Canister)"
                            }
                        }
                    }
                }
            }

            if *show_rss.read() {
                div { class: "absolute inset-0 z-50 bg-black/60 flex items-center justify-center p-4",
                    div { class: "w-full max-w-3xl bg-card border border-border rounded-xl shadow-2xl p-4 space-y-3",
                        div { class: "flex items-center justify-between",
                            h3 { class: "text-sm font-bold uppercase tracking-widest text-muted-foreground", "{feed_xml_kind} (Derived View)" }
                            button {
                                class: "p-1 hover:bg-muted rounded text-muted-foreground",
                                onclick: move |_| show_rss.set(false),
                                "×"
                            }
                        }
                        pre { class: "bg-muted/40 border border-border rounded-lg p-3 text-xs overflow-auto max-h-[70vh]", "{rss_xml}" }
                    }
                }
            }

            // Reader Body
            div { class: "flex-1 overflow-hidden relative",
                div { class: "h-full flex flex-col md:flex-row max-w-7xl mx-auto bg-background shadow-xl overflow-hidden border-x",
                    // TOC
                    if !access_blocked {
                        div { class: "w-64 border-r bg-muted/10 p-4 hidden lg:block overflow-y-auto shrink-0",
                            h3 { class: "font-bold text-xs uppercase tracking-widest text-muted-foreground mb-4", "Table of Contents" }
                            ul { class: "space-y-1",
                                for content in displayed_book.content.iter() {
                                    li { class: "text-sm py-2 px-3 rounded-md hover:bg-accent hover:text-accent-foreground cursor-pointer transition-colors text-muted-foreground hover:translate-x-1 duration-200",
                                        "{content.title.clone().unwrap_or_default()}"
                                    }
                                }
                            }
                        }
                    }

                    // Content
                    div {
                        class: "flex-1 overflow-y-auto bg-background relative",
                        onscroll: move |_| {
                            let current = *reading_progress.read();
                            if current < 100.0 {
                                reading_progress.set((current + 0.5f64).min(100.0f64));
                            }
                        },
                        div { class: "sticky top-0 z-20 h-1 bg-muted/30 overflow-hidden",
                            div {
                                class: "h-full bg-blue-500/50 transition-all duration-300",
                                style: "width: {reading_progress.read()}%",
                            }
                        }

                        div { class: "sticky top-1 z-10 bg-card/80 backdrop-blur-md border-b px-4 py-2 flex justify-between items-center",
                            div { class: "flex items-center gap-2",
                                span { class: "w-2 h-2 rounded-full bg-green-500 animate-pulse" }
                                span { class: "text-[10px] font-bold text-muted-foreground uppercase tracking-wider", "Telemetry Active" }
                            }
                            button {
                                class: "text-[10px] font-bold text-primary hover:underline uppercase tracking-wider",
                                onclick: move |_| show_note_panel.toggle(),
                                if *show_note_panel.read() { "Close Notes" } else { "Open Marginalia" }
                            }
                        }

                        if access_blocked {
                            div { class: "p-10 md:p-16 max-w-3xl mx-auto text-center",
                                div { class: "inline-flex items-center gap-2 px-3 py-1 rounded-full bg-red-500/10 border border-red-500/30 text-red-400 text-[10px] uppercase tracking-widest",
                                    "Restricted"
                                }
                                h2 { class: "text-2xl font-serif font-bold mt-6", "Cross-space treaty required" }
                                p { class: "text-sm text-muted-foreground mt-2",
                                    "This dPub belongs to {displayed_book.meta.provenance.space_did}. Current space: {viewer_space_label}."
                                }
                                p { class: "text-xs text-muted-foreground mt-2",
                                    "Add a treaty token in Lab Configuration to view or export this content."
                                }
                            }
                        } else {
                            div { class: "p-8 md:p-12 lg:p-16 max-w-3xl mx-auto",
                                div { class: "text-center pb-8 mb-8 border-b border-border/50",
                                    h1 { class: "text-4xl md:text-5xl font-serif font-bold mb-4 tracking-tight text-foreground", "{displayed_book.meta.title}" }
                                    p { class: "text-lg text-muted-foreground font-medium", "by {displayed_book.meta.provenance.author_did}" }
                                    if let Some(hypothesis) = &displayed_book.hypothesis {
                                        div { class: "mt-8 p-4 bg-muted border border-border rounded-lg text-left relative overflow-hidden",
                                            div { class: "absolute left-0 top-0 bottom-0 w-1 bg-primary" }
                                            div { class: "text-xs font-bold text-primary tracking-widest uppercase mb-1", "🧪 Hypothesis" }
                                            p { class: "text-sm text-foreground italic font-serif", "\"{hypothesis}\"" }
                                        }
                                    }
                                    p { class: "text-xs text-muted-foreground mt-4 uppercase tracking-widest", "Published {displayed_book.meta.provenance.created_at}" }
                                }

                                for content in displayed_book.content.iter() {
                                    div { class: "mb-16",
                                        if let Some(t) = &content.title {
                                            if let Some(Block::Heading { content: h_text, .. }) = content.blocks.first() {
                                                if !t.contains(h_text) && !h_text.contains(t) {
                                                    h2 { class: "text-2xl font-serif font-bold mb-6 text-foreground", "{t}" }
                                                }
                                            } else {
                                                h2 { class: "text-2xl font-serif font-bold mb-6 text-foreground", "{t}" }
                                            }
                                        }
                                        for block in content.blocks.iter() {
                                            div { class: "group relative",
                                                button {
                                                    class: "absolute -right-12 top-0 p-1.5 rounded-full bg-muted border border-border opacity-0 group-hover:opacity-100 transition-all hover:bg-accent text-muted-foreground hover:text-accent-foreground shadow-sm z-30",
                                                    title: "Add Note to this section",
                                                    onclick: move |_| show_note_panel.set(true),
                                                    "+"
                                                }
                                                match block {
                                                    Block::Heading { level: _, content } => rsx! {
                                                        h3 { class: "text-xl font-bold mt-4 mb-2", "{content}" }
                                                    },
                                                    Block::Paragraph { content } => rsx! {
                                                        div { class: "prose dark:prose-invert prose-lg max-w-none text-muted-foreground leading-relaxed mb-4",
                                                            match content {
                                                                ContentValue::String(s) => rsx! { "{s}" },
                                                                ContentValue::Rich(spans) => rsx! {
                                                                    for span in spans {
                                                                        match span {
                                                                            RichTextSpan::Text { value } => rsx! { "{value}" },
                                                                            RichTextSpan::Bold { value } => rsx! { strong { "{value}" } },
                                                                            RichTextSpan::Italic { value } => rsx! { em { "{value}" } },
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    },
                                                    Block::LegacyHtml { content } => {
                                                        if *show_raw.read() {
                                                            rsx! {
                                                                pre { class: "bg-muted p-4 rounded-lg overflow-x-auto text-xs font-mono border", "{content}" }
                                                            }
                                                        } else {
                                                            let sanitized = sanitize_html(content);
                                                            rsx! {
                                                                div { class: "prose dark:prose-invert prose-lg max-w-none text-muted-foreground leading-relaxed",
                                                                    dangerous_inner_html: "{sanitized}"
                                                                }
                                                            }
                                                        }
                                                    },
                                                    _ => rsx! { div { "Block type not rendered" } }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // Cortex Console (audit traces)
                    if *show_cortex.read() {
                        div { class: "w-80 border-l border-border bg-black text-green-400 p-4 overflow-y-auto font-mono text-xs shadow-2xl shrink-0",
                            div { class: "flex items-center gap-2 mb-4 pb-2 border-b border-green-900/50",
                                div { class: "w-2 h-2 bg-green-500 rounded-full animate-pulse" }
                                span { class: "font-bold tracking-widest", "CORTEX_AUDIT_TRACE" }
                            }
                            div { class: "space-y-4 font-screen",
                                if audit_traces.read().is_empty() {
                                    div { class: "opacity-60", "No audit traces for this dPub yet." }
                                }
                                for trace in audit_traces.read().iter() {
                                    {
                                        let action = trace.get("action").and_then(|v| v.as_str()).unwrap_or("unknown");
                                        let timestamp = trace.get("timestamp").and_then(|v| v.as_str()).unwrap_or("-");
                                        let edition_id = trace.get("edition_id").and_then(|v| v.as_str()).unwrap_or("");
                                        let content_root = trace
                                            .get("outputs")
                                            .and_then(|v| v.get("content_root"))
                                            .and_then(|v| v.as_str())
                                            .unwrap_or("");
                                        rsx! {
                                            div { class: "p-3 bg-green-900/10 rounded border border-green-900/30",
                                                span { class: "text-white font-bold block mb-1", "{action}" }
                                                div { class: "opacity-70", "ts: {timestamp}" }
                                                if !edition_id.is_empty() {
                                                    div { class: "opacity-70", "edition: {edition_id}" }
                                                }
                                                if !content_root.is_empty() {
                                                    div { class: "opacity-70 break-all", "root: {content_root}" }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // Marginalia Panel (kept from prior version)
                    if *show_note_panel.read() {
                        div { class: "w-80 border-l bg-card flex flex-col shrink-0 animate-in slide-in-from-right duration-300 shadow-2xl",
                            div { class: "p-4 border-b flex justify-between items-center bg-muted/50",
                                h3 { class: "font-bold text-xs uppercase tracking-widest", "Marginalia" }
                                button {
                                    class: "p-1 hover:bg-muted rounded transition-all",
                                    onclick: move |_| show_note_panel.set(false),
                                    "×"
                                }
                            }

                            div { class: "flex-1 overflow-y-auto p-4 space-y-4",
                                if notes.read().is_empty() {
                                    div { class: "text-center py-12 opacity-50",
                                        div { class: "text-2xl mb-2", "✍️" }
                                        p { class: "text-xs", "No notes yet. Capture thoughts as you read." }
                                    }
                                }
                                for (i, note) in notes.read().iter().enumerate() {
                                    div { class: "p-3 bg-accent/20 rounded-lg border border-accent/30 text-xs text-foreground",
                                        "{note.content}"
                                        div { class: "mt-2 opacity-50 flex justify-between items-center text-[10px]",
                                            span { "{note.timestamp}" }
                                            button {
                                                class: "hover:text-red-500",
                                                onclick: move |_| {
                                                    notes.write().remove(i);
                                                },
                                                "Delete"
                                            }
                                        }
                                    }
                                }
                            }

                            div { class: "p-4 border-t bg-muted/20",
                                textarea {
                                    class: "w-full p-3 bg-background border rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-primary min-h-[100px] resize-none",
                                    placeholder: "Capture a note to your Personal OS...",
                                    value: "{active_note_content}",
                                    oninput: move |evt| active_note_content.set(evt.value())
                                }
                                button {
                                    class: "w-full mt-2 bg-primary text-primary-foreground py-2 rounded-lg text-sm font-bold hover:opacity-90 transition-opacity shadow-sm",
                                    onclick: move |_| {
                                        if !active_note_content.read().is_empty() {
                                            notes.write().push(Note {
                                                id: Uuid::new_v4().to_string(),
                                                book_id: displayed_book.meta.id.clone(),
                                                chapter_id: "reader".to_string(),
                                                content: active_note_content.read().clone(),
                                                timestamp: "Just now".to_string(),
                                            });
                                            active_note_content.set(String::new());
                                        }
                                    },
                                    "Capture Thought"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
