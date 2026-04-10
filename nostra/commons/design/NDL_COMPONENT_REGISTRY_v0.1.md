# NDL Component Registry v0.1

**Status**: Draft
**Applies To**: All Nostra Cortex Renderers
**Authority Level**: Constitutional

---

## 1. Component Classification

Components are divided into three tiers:

| Tier | Category | Override Allowed? |
| :--- | :--- | :--- |
| Tier 1 | Constitutional | ❌ Never |
| Tier 2 | Structural | ⚠ Profile Only |
| Tier 3 | Presentational | ✅ Yes |

---

## 2. Tier 1 – Constitutional Components (Immutable)

These may not be overridden, hidden, restyled beyond token variance, or removed by:
*   AI Agents
*   Profiles
*   Themes
*   Forked Commons

### 2.1 ContributionTypeIndicator
Displays canonical type.

Examples:
*   `Idea`
*   `Proposal`
*   `Decision`
*   `Project`

**Must**:
*   Be visible at header level
*   Be machine-detectable
*   Use NDL icon set

### 2.2 PhaseBadge
Displays phase:
*   Exploratory
*   Deliberative
*   Decisive
*   Executable
*   Archival
*   Structural

**Must**:
*   Visually differentiate decisiveness
*   Be high contrast

### 2.3 GovernanceIndicator
Required for:
*   `Proposal`
*   `Decision`
*   Institutional governance artifacts

**Must display**:
*   Vote state
*   Strategy
*   Ratification status

### 2.4 RatificationMarker
Only for `Decision`.

**Must**:
*   Be visually distinct
*   Use immutable accent token
*   Appear in header region

### 2.5 VersionChainDisplay
**Must**:
*   Show current version
*   Show link to previous version
*   Not be collapsible by default

### 2.6 SpaceVisibilityIndicator
**Must**:
*   Display Public / Private
*   Reflect space-level truth

---

## 3. Tier 2 – Structural Components

May adapt per profile but must preserve semantic meaning.

### 3.1 MetadataBlock
Displays:
*   Contributors
*   Timestamps
*   Status
*   Confidence

**Cannot hide required fields.**

### 3.2 LineageBreadcrumb
Displays:
*   Fork origin
*   Merge ancestry

*Animation allowed on first render.*

### 3.3 InteractionLayer
Displays:
*   Upvote / Downvote
*   Follow
*   Add to List
*   Contribute

*Layout may adapt per profile.*

---

## 4. Tier 3 – Presentational Components

May be customized freely within token constraints.
*   Buttons
*   Cards
*   Modals
*   Inputs
*   Tabs
*   TagCloud
*   Animations (non-semantic)

---

## 5. Governance-Sensitive Components

These require validation hooks:
*   `ProposalVotePanel`
*   `DecisionSummaryBlock`
*   `CommonsAdoptionIndicator`
*   `InstitutionalLifecycleBadge`

**Renderer must verify**:
*   Data integrity
*   Immutable visual anchor
*   Governance state accuracy

---

## 6. Profile Overrides

Profiles may adjust:
*   Density
*   Spacing
*   Layout grid
*   Color tone within token range

Profiles **may not adjust**:
*   Authority markers
*   Governance visuals
*   Contribution grammar

---

## 7. AI Rendering Enforcement

AI renderers must:
*   Use only registry components
*   Respect Tier 1 immutability
*   Declare `ai_render_safe: true`
*   Pass NDL schema validation

---

## 8. Extension Policy

New components require:
1.  Proposal submission
2.  Steward review
3.  Version bump
4.  Registry update

---

## 9. Future Additions (v0.2 Targets)
*   Cryptographic governance badge
*   Signature verification UI
*   Temporal heatmap overlays
*   Cross-space federation indicator
*   Simulation state delta renderer
