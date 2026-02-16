use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::{BTreeSet, HashMap};

const POSITION_BASE: u32 = 65_535;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CrdtPosition {
    pub digits: Vec<u32>,
    pub actor_id: String,
    pub seq: u64,
}

impl Ord for CrdtPosition {
    fn cmp(&self, other: &Self) -> Ordering {
        self.digits
            .cmp(&other.digits)
            .then_with(|| self.actor_id.cmp(&other.actor_id))
            .then_with(|| self.seq.cmp(&other.seq))
    }
}

impl PartialOrd for CrdtPosition {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ArtifactCrdtChar {
    pub char_id: String,
    pub position: CrdtPosition,
    pub value: String,
    pub deleted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ArtifactCrdtMutation {
    Insert {
        char_id: String,
        position: CrdtPosition,
        value: String,
    },
    Delete {
        char_id: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ArtifactCrdtUpdateEnvelope {
    pub op_id: String,
    pub artifact_id: String,
    pub session_id: String,
    pub actor_id: String,
    pub sequence: u64,
    pub lamport: u64,
    pub created_at: String,
    #[serde(default)]
    pub stream_channel: Option<String>,
    #[serde(default)]
    pub mutations: Vec<ArtifactCrdtMutation>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ArtifactCrdtConflict {
    pub code: String,
    pub message: String,
    pub blocking: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ArtifactCrdtState {
    pub schema_version: String,
    pub artifact_id: String,
    pub updated_at: String,
    #[serde(default)]
    pub chars: Vec<ArtifactCrdtChar>,
    #[serde(default)]
    pub applied_op_ids: Vec<String>,
    #[serde(default)]
    pub pending_delete_ids: Vec<String>,
    pub last_lamport: u64,
    pub last_sequence: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ArtifactCrdtApplyResult {
    pub applied: bool,
    pub idempotent: bool,
    pub materialized_markdown: String,
    pub op_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ArtifactCollabCheckpoint {
    pub checkpoint_id: String,
    pub artifact_id: String,
    pub created_at: String,
    pub op_count: u64,
    pub state_hash: String,
    pub snapshot_key: String,
}

pub fn init_state(artifact_id: &str, markdown: &str) -> ArtifactCrdtState {
    let mut chars = Vec::new();
    for (idx, ch) in markdown.chars().enumerate() {
        let seq = (idx as u64) + 1;
        chars.push(ArtifactCrdtChar {
            char_id: format!("bootstrap:{seq:08}"),
            position: CrdtPosition {
                digits: vec![((idx as u32) + 1) * 2],
                actor_id: "bootstrap".to_string(),
                seq,
            },
            value: ch.to_string(),
            deleted: false,
        });
    }

    ArtifactCrdtState {
        schema_version: "1.0.0".to_string(),
        artifact_id: artifact_id.to_string(),
        updated_at: Utc::now().to_rfc3339(),
        chars,
        applied_op_ids: Vec::new(),
        pending_delete_ids: Vec::new(),
        last_lamport: 0,
        last_sequence: 0,
    }
}

pub fn materialize_markdown(state: &ArtifactCrdtState) -> String {
    let mut visible = state
        .chars
        .iter()
        .filter(|item| !item.deleted)
        .collect::<Vec<_>>();
    visible.sort_by(|a, b| a.position.cmp(&b.position));
    visible
        .into_iter()
        .map(|item| item.value.as_str())
        .collect()
}

pub fn build_replace_markdown_update(
    state: &ArtifactCrdtState,
    artifact_id: &str,
    session_id: &str,
    actor_id: &str,
    op_id: &str,
    sequence: u64,
    lamport: u64,
    markdown_source: &str,
    stream_channel: Option<String>,
) -> ArtifactCrdtUpdateEnvelope {
    let mut visible = state
        .chars
        .iter()
        .filter(|item| !item.deleted)
        .collect::<Vec<_>>();
    visible.sort_by(|a, b| a.position.cmp(&b.position));

    let old_chars = visible
        .iter()
        .map(|item| item.value.clone())
        .collect::<Vec<_>>();
    let new_chars = markdown_source
        .chars()
        .map(|item| item.to_string())
        .collect::<Vec<_>>();

    let mut prefix = 0usize;
    while prefix < old_chars.len()
        && prefix < new_chars.len()
        && old_chars[prefix] == new_chars[prefix]
    {
        prefix += 1;
    }

    let mut suffix = 0usize;
    while suffix < (old_chars.len() - prefix)
        && suffix < (new_chars.len() - prefix)
        && old_chars[old_chars.len() - 1 - suffix] == new_chars[new_chars.len() - 1 - suffix]
    {
        suffix += 1;
    }

    let old_delete_start = prefix;
    let old_delete_end = old_chars.len().saturating_sub(suffix);
    let new_insert_start = prefix;
    let new_insert_end = new_chars.len().saturating_sub(suffix);

    let mut mutations = Vec::new();
    for node in visible
        .iter()
        .skip(old_delete_start)
        .take(old_delete_end.saturating_sub(old_delete_start))
    {
        mutations.push(ArtifactCrdtMutation::Delete {
            char_id: node.char_id.clone(),
        });
    }

    let mut left = if old_delete_start == 0 {
        None
    } else {
        visible
            .get(old_delete_start - 1)
            .map(|item| item.position.clone())
    };
    let right = if old_delete_end >= visible.len() {
        None
    } else {
        visible
            .get(old_delete_end)
            .map(|item| item.position.clone())
    };

    for (offset, value) in new_chars
        .iter()
        .skip(new_insert_start)
        .take(new_insert_end.saturating_sub(new_insert_start))
        .enumerate()
    {
        let local_seq = sequence.saturating_mul(10_000) + (offset as u64) + 1;
        let position = alloc_between(left.as_ref(), right.as_ref(), actor_id, local_seq);
        let char_id = format!("{actor_id}:{lamport}:{offset:08}");
        mutations.push(ArtifactCrdtMutation::Insert {
            char_id,
            position: position.clone(),
            value: value.clone(),
        });
        left = Some(position);
    }

    ArtifactCrdtUpdateEnvelope {
        op_id: op_id.to_string(),
        artifact_id: artifact_id.to_string(),
        session_id: session_id.to_string(),
        actor_id: actor_id.to_string(),
        sequence,
        lamport,
        created_at: Utc::now().to_rfc3339(),
        stream_channel,
        mutations,
    }
}

pub fn apply_update(
    state: &mut ArtifactCrdtState,
    envelope: &ArtifactCrdtUpdateEnvelope,
) -> Result<ArtifactCrdtApplyResult, String> {
    if envelope.artifact_id != state.artifact_id {
        return Err("artifact ID mismatch for CRDT update envelope".to_string());
    }

    if state
        .applied_op_ids
        .iter()
        .any(|existing| existing == &envelope.op_id)
    {
        return Ok(ArtifactCrdtApplyResult {
            applied: false,
            idempotent: true,
            materialized_markdown: materialize_markdown(state),
            op_count: state.applied_op_ids.len() as u64,
        });
    }

    let mut index = HashMap::new();
    for (idx, node) in state.chars.iter().enumerate() {
        index.insert(node.char_id.clone(), idx);
    }
    let mut pending: BTreeSet<String> = state.pending_delete_ids.iter().cloned().collect();

    for mutation in &envelope.mutations {
        match mutation {
            ArtifactCrdtMutation::Insert {
                char_id,
                position,
                value,
            } => {
                if index.contains_key(char_id) {
                    continue;
                }
                let deleted = pending.remove(char_id);
                let node = ArtifactCrdtChar {
                    char_id: char_id.clone(),
                    position: position.clone(),
                    value: value.clone(),
                    deleted,
                };
                state.chars.push(node);
                index.insert(char_id.clone(), state.chars.len() - 1);
            }
            ArtifactCrdtMutation::Delete { char_id } => {
                if let Some(idx) = index.get(char_id).copied() {
                    if let Some(node) = state.chars.get_mut(idx) {
                        node.deleted = true;
                    }
                } else {
                    pending.insert(char_id.clone());
                }
            }
        }
    }

    state.pending_delete_ids = pending.into_iter().collect();
    state.applied_op_ids.push(envelope.op_id.clone());
    state.applied_op_ids.sort();
    state.applied_op_ids.dedup();
    state.last_lamport = state.last_lamport.max(envelope.lamport);
    state.last_sequence = state.last_sequence.max(envelope.sequence);
    state.updated_at = Utc::now().to_rfc3339();

    Ok(ArtifactCrdtApplyResult {
        applied: true,
        idempotent: false,
        materialized_markdown: materialize_markdown(state),
        op_count: state.applied_op_ids.len() as u64,
    })
}

pub fn state_hash(state: &ArtifactCrdtState) -> String {
    let mut visible = state
        .chars
        .iter()
        .filter(|item| !item.deleted)
        .map(|item| format!("{}:{}", item.char_id, item.value))
        .collect::<Vec<_>>();
    visible.sort();
    format!(
        "sha-lite:{}:{}",
        state.applied_op_ids.len(),
        visible.join("|")
    )
}

fn alloc_between(
    left: Option<&CrdtPosition>,
    right: Option<&CrdtPosition>,
    actor_id: &str,
    seq: u64,
) -> CrdtPosition {
    let mut digits = Vec::new();
    let mut index = 0usize;
    loop {
        let left_digit = left
            .and_then(|item| item.digits.get(index).copied())
            .unwrap_or(0);
        let right_digit = right
            .and_then(|item| item.digits.get(index).copied())
            .unwrap_or(POSITION_BASE);
        if right_digit > left_digit + 1 {
            digits.push((left_digit + right_digit) / 2);
            break;
        }
        digits.push(left_digit);
        index += 1;
    }

    CrdtPosition {
        digits,
        actor_id: actor_id.to_string(),
        seq,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crdt_converges_three_client_sequence() {
        let mut state_a = init_state("artifact-1", "abc");
        let mut state_b = state_a.clone();
        let mut state_c = state_a.clone();

        let op_1 = build_replace_markdown_update(
            &state_a,
            "artifact-1",
            "sess-1",
            "actor-a",
            "op-1",
            1,
            10,
            "aXbc",
            Some("stream:cortex:artifact-1".to_string()),
        );
        let op_2 = build_replace_markdown_update(
            &state_b,
            "artifact-1",
            "sess-1",
            "actor-b",
            "op-2",
            1,
            11,
            "abYc",
            Some("stream:cortex:artifact-1".to_string()),
        );
        let op_3 = build_replace_markdown_update(
            &state_c,
            "artifact-1",
            "sess-1",
            "actor-c",
            "op-3",
            1,
            12,
            "abcZ",
            Some("stream:cortex:artifact-1".to_string()),
        );

        apply_update(&mut state_a, &op_1).expect("apply op1 to a");
        apply_update(&mut state_a, &op_2).expect("apply op2 to a");
        apply_update(&mut state_a, &op_3).expect("apply op3 to a");

        apply_update(&mut state_b, &op_3).expect("apply op3 to b");
        apply_update(&mut state_b, &op_1).expect("apply op1 to b");
        apply_update(&mut state_b, &op_2).expect("apply op2 to b");

        apply_update(&mut state_c, &op_2).expect("apply op2 to c");
        apply_update(&mut state_c, &op_3).expect("apply op3 to c");
        apply_update(&mut state_c, &op_1).expect("apply op1 to c");

        let final_a = materialize_markdown(&state_a);
        let final_b = materialize_markdown(&state_b);
        let final_c = materialize_markdown(&state_c);
        assert_eq!(final_a, final_b);
        assert_eq!(final_b, final_c);
    }

    #[test]
    fn duplicate_op_is_idempotent() {
        let mut state = init_state("artifact-1", "hello");
        let op = build_replace_markdown_update(
            &state,
            "artifact-1",
            "sess-1",
            "actor-a",
            "op-idempotent",
            1,
            10,
            "hello world",
            None,
        );

        let first = apply_update(&mut state, &op).expect("first apply");
        assert!(first.applied);
        assert!(!first.idempotent);

        let second = apply_update(&mut state, &op).expect("second apply");
        assert!(!second.applied);
        assert!(second.idempotent);
    }
}
