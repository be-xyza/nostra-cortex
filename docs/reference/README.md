# Reference Intake Governance

This is the operational source of truth for classifying and adding repositories under the research reference extension.

## Canonical Paths

- Canonical root: `research/reference/`
- Canonical metadata: `research/reference/index.toml` (Repos), `research/reference/knowledge/index.toml` (Knowledge)
- Canonical analyses: `research/reference/analysis/*.md`
- Knowledge Artifacts: `research/reference/knowledge/{topic}/{year}_{author}_{slug}/`
- Knowledge metadata template: `research/reference/knowledge/PAPER_TEMPLATE.md`
- Analysis template: `research/reference/analysis/ANALYSIS_TEMPLATE.md`
- Knowledge taxonomy/extension registry: `docs/reference/knowledge_taxonomy.toml`

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
5. Create `metadata.md` from `PAPER_TEMPLATE.md` using `schema_version: "2.0"`.
6. Create/update analysis memo in `research/reference/analysis/<artifact-id>.md` using AnalysisV2 sections.
7. Register path in `research/reference/knowledge/index.toml`.
8. Run validator: `python3 scripts/check_reference_metadata_v2.py --strict`.

## Evolvability Contract (Types + Categories)
- Add new `artifact_type` values by updating `docs/reference/knowledge_taxonomy.toml` (`artifact_types.allowed`) or by using an approved custom prefix in `artifact_types.custom_prefixes`.
- Add provisional out-of-scope topics in `docs/reference/knowledge_taxonomy.toml` under `topic_extensions.allowed`.
- Promote provisional topics into `docs/reference/topics.md` once stable and steward-confirmed.
- No validator code changes are required for taxonomy growth if updates are made via the registry file.

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
- If a new topic is created, update `docs/reference/topics.md`.
- For sensitive actions, log decision entries in `research/097-nostra-cortex-alignment/DECISIONS.md`.
- If classification policy changes, archive and update `AGENTS.md`.

## Validator Contract (`scripts/check_reference_metadata_v2.py`)
Prerequisite integrity guard:
- `python3 scripts/check_reference_taxonomy_integrity.py --strict`

Hard failures:
1. Placeholder content in metadata (`TBD`, `{1-5}`, `Unknown`, `Risk 1`, etc.).
2. Missing/invalid `source_files[].sha256`.
3. Topic not present in `docs/reference/topics.md`.
4. Empty `initiative_refs` when `status` is `reviewed` or `adopted`.
5. Missing confidence fields or `validation_proof`.
6. Missing universal `standards_alignment` block.
7. Analysis docs missing required AnalysisV2 sections.
8. Analysis docs containing dead internal paths.
9. Empty `validation_proof.evidence_refs` when `status` is `reviewed` or `adopted`.

Warnings:
1. Knowledge index entry without `metadata.md` sidecar.
2. Knowledge topics used in `knowledge/index.toml` but absent from topic registry.
