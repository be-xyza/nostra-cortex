# Reference Intake Governance

This is the operational source of truth for classifying and adding repositories under the research reference extension.

## Canonical Paths

- Canonical root: `research/reference/`
- Canonical metadata: `research/reference/index.toml` (Repos), `research/reference/knowledge/index.toml` (Knowledge)
- Canonical analyses: `research/reference/analysis/*.md`
- Knowledge Artifacts: `research/reference/knowledge/{topic}/{year}_{author}_{slug}/`

## Local Contract Note

This checkout does not currently include local copies of the following assets that older intake workflow notes referenced:

- `research/reference/knowledge/PAPER_TEMPLATE.md`
- `research/reference/analysis/ANALYSIS_TEMPLATE.md`
- `docs/reference/knowledge_taxonomy.toml`
- `docs/reference/topics.md`
- `scripts/check_reference_metadata_v2.py`
- `scripts/check_reference_taxonomy_integrity.py`

Until those assets are restored, reference intake in this repo operates in `primary-source manually validated` mode for knowledge artifacts. Use the required field/section contracts in this document rather than claiming validator-backed compliance.

## Constitutional Operating Mode

- Default authority mode is `recommendation_only`.
- Sensitive actions (`rename`, `merge`, `archive`, `delete`, root moves, scope changes) require steward escalation and logged rationale.
- Archive-before-update is required for governance and catalog files before mutation.

## Required Workflow (Agents)
1. Analyze candidate repository intent and fit.
2. Frame intake as an experiment (assumptions, expected retained artifacts, decision gates).
3. Score relevance with the required scorecard.
4. Decide placement (`research/reference/repos`, `research/reference/topics/<topic>`, or `research/reference/inbox`).
5. Register metadata and analysis artifacts.
6. If action is sensitive, escalate and log a decision before execution.

## Required Workflow (Knowledge Artifacts)
1. Categorize artifact (`paper|book|standard|legal_doc`).
2. Determine topic placement (`research/reference/knowledge/<topic>`).
3. Create folder `research/reference/knowledge/<topic>/<year>_<author>_<slug>`.
4. Place source files (PDF, LaTeX, TXT, etc.) in that folder.
5. Create `metadata.md` with `schema_version: "2.0"` and the required fields listed below.
6. Create/update analysis memo in `research/reference/analysis/<artifact-id>.md` using AnalysisV2 sections.
7. Register path in `research/reference/knowledge/index.toml`.
8. If local validator assets are restored, run them. Otherwise record the intake as `primary-source manually validated` and note any missing validator/tooling drift.

## Evolvability Contract (Current Local Mode)
- Use the base artifact types documented here: `paper|book|standard|legal_doc`.
- Prefer existing local topics unless a steward explicitly approves adding a new topic without a restored topic-registry file.
- Restoring taxonomy/topic-registry files is a separate governance task; do not imply they are active until they exist in this checkout.

## Placement Rules
- Place under an existing topic when `topic_fit >= 4`.
- Create a new topic only when `topic_fit <= 3` and at least two related repos are expected within 60 days.
- Place in `research/reference/repos/<repo-name>` when useful but cross-topic.
- Reject intake when `(ecosystem_fit + adapter_value + component_value + pattern_value + ux_value + future_optionality) < 12` and no active research initiative references it.
- Topic names must be kebab-case and capability/domain oriented (vendor name allowed only for vendor ecosystem bundles like `temporal`).

## Required Scorecard Fields
- `ecosystem_fit` (0-5)
- `adapter_value` (0-5)
- `component_value` (0-5)
- `pattern_value` (0-5)
- `ux_value` (0-5)
- `future_optionality` (0-5)
- `topic_fit` (0-5)

## Required Metadata Fields (`research/reference/index.toml`)
- `why_here`
- `links_to_nostra_cortex` (canonical key; do not use `possible_links_to_nostra_cortex`)
- `known_risks`
- `suggested_next_experiments`
- `primary_steward`
- `authority_mode` (`recommendation_only|execute_with_approval`)
- `escalation_path`
- `lineage_record`
- `initiative_refs` (list of research initiative IDs)

## Required Metadata Fields (`knowledge/*/*/metadata.md`)
- `schema_version` (`2.0`)
- `artifact_id`
- `artifact_type` (`paper|book|standard|legal_doc`)
- `title`, `authors`, `year`, `publisher`, `upstream_url`
- `source_files[]` with `path`, `sha256` (64-char hex), `mime_type`
- `topic`, `tags`, `status`, `nostra_cortex_scope`
- `initiative_refs`, `primary_steward`, `authority_mode`, `escalation_path`, `lineage_record`, `review_date`
- `confidence_score`, `source_reliability`, `validation_proof`
- `standards_alignment` (universal six dimensions + weighted score)
- `topic_alignment` (optional, topic-specific dimensions)
- `classification_extensions` (optional, for custom/out-of-scope classification metadata)
- `adoption_decision`, `known_risks`, `suggested_next_experiments`

## Required Analysis Fields (`analysis/*.md`) — AnalysisV2
- `Placement`
- `Intent`
- `Possible Links To Nostra Platform and Cortex Runtime`
- `Initiative Links`
- `Pattern Extraction`
- `Adoption Decision`
- `Known Risks`
- `Suggested Next Experiments`

## Hybrid Standards Scoring (KnowledgeMetadataV2)
- Universal layer required for all artifacts: `modularity`, `composability`, `confidence_integrity`, `portability`, `durability`, `accessibility`.
- Each dimension must include `score (0..5)`, `applicability (core|supporting|not_applicable)`, and `rationale`.
- `overall_weighted_score` is computed from `core + supporting` dimensions only.
- Topic layer is optional but recommended when it materially impacts adoption decisions.

## Mandatory Updates Per Intake
- Add or update `research/reference/index.toml` and `research/reference/index.md` for repos.
- Add or update `research/reference/knowledge/index.toml` for knowledge artifacts.
- Add or update analysis memo in `research/reference/analysis/*.md`.
- If a new topic is required before a local topic registry exists, escalate to a steward and record the decision.
- For sensitive actions, log decision entries in `research/097-nostra-cortex-alignment/DECISIONS.md`.
- If classification policy changes, archive and update `AGENTS.md`.

## Manual Validation Contract (Current Local Mode)
Until validator assets are restored, knowledge intake must be closed out as `primary-source manually validated` and must manually verify:

Hard failures:
1. Placeholder content in metadata (`TBD`, `{1-5}`, `Unknown`, `Risk 1`, etc.).
2. Missing/invalid `source_files[].sha256`.
3. Empty `initiative_refs` when `status` is `reviewed` or `adopted`.
4. Missing confidence fields or `validation_proof`.
5. Missing universal `standards_alignment` block.
6. Analysis docs missing required AnalysisV2 sections.
7. Analysis docs containing dead internal paths.
8. Empty `validation_proof.evidence_refs` when `status` is `reviewed` or `adopted`.

Warnings:
1. Knowledge index entry without `metadata.md` sidecar.
2. Intake claims validator-backed or fully protocol-compliant status without restored validator assets and an actual validator run.

If validator and topic-registry assets are restored later, this section can be replaced with a validator-backed contract again.
