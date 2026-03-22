# Nostra Platform Specification (v2)

## Overview
Nostra is the **platform authority layer** of Nostra Cortex on the Internet Computer Protocol (ICP). It defines canonical entities, governance, contribution history, and platform constraints while Cortex provides execution runtime behavior.

Uniquely, Nostra treats **Time** and **Simulation** as first-class primitives alongside Knowledge. It integrates a durable workflow engine (Temporal), a gaming/simulation bridge (Godot), and a vector-native memory layer to support not just static collaboration, but dynamic, long-running, and interactive processes.

Nostra is designed to support:
- Open or private collaboration
- Iterative refinement and forking
- Knowledge lineage and traceability
- Structured execution and accountability
- Long-term learning and discovery
- **Durable, multi-year workflows**
- **interactive simulations and game states**

Built on ICP, Nostra leverages decentralized identity, deterministic backends, and a composable canister architecture to support scalable, transparent, and auditable collaboration.

## Design Principles
1. **Everything Is a Contribution**
   - Ideas, questions, issues, comments, deliverables, and milestones are all contributions with authorship, timestamps, and context.
2. **Time Is a Primitive**
   - We do not just "store" time; we *execute* over it. Workflows can sleep for months, wake up, and continue reliably.
3. **Execution Is First-Class**
   - Projects, initiatives, deliverables, and milestones are not "attachments" — they are core entities linked to the knowledge graph.
4. **Simulation Is Valid Input**
   - Game states, simulations, and interactive labs feed data back into the knowledge graph just like human comments do.
5. **Polyglot Interface**
   - We use the right tool for the job: React/Vite/cortex-web for the secure host, A2UI for the protocol, Lit for standard components, and D3 for visualization.
6. **History Is Sacred**
   - Versioning, lineage, forks, merges, and decisions are preserved and queryable.
7. **Spaces Are Sovereign**
   - Each space defines its own culture, contribution types, visibility, and governance constraints.
8. **Modular Persistence**
   - Architecture scales horizontally via standalone canisters (e.g., `kg-registry`, `kg-schema`), each owning its specific domain and memory bounds (DEC-002).

---

## Architecture Overview

Nostra Cortex operates on a **polyglot stack** with two conceptual layers:

> **Nostra** = Platform authority layer (data, contributions, governance; defines what exists)
> **Cortex** = Execution runtime layer (workers, agents, workflows; defines how work runs)

1.  **Backend (The "Brain")**:
    *   **Canisters (Motoko/Rust)**: Core logic, storage, and indexers.
    *   **Execution Adapters (Rust)**: Handles durable execution through a Hybrid Workflow Authority (Initiative 134), executing `nostra-workflow-core` state machines natively off-chain for WASM compatibility rather than relying strictly on the official Temporal SDK. Serverless Workflow DSL is a deterministic projection, not the canonical loop.
    *   **Grader Matrix & Evaluation**: Subagent execution incorporates formal Eval-Driven Orchestration (Initiative 133) utilizing predefined Grader workflows to evaluate capabilities and route A2UI Feedback Projections natively.
    *   **Vector Database**: Specialized canisters for embedding storage and semantic search.

3.  **Cortex (The "Execution Layer")**:
    *   **Delegated**: See `cortex/spec.md` for runtime details.

---




## Unified Contribution Model

All primary objects in Nostra inherit from a base `Contribution` abstraction:

### Contribution (Abstract Type)
```
Contribution
- id: Text
- spaceId: Text
- type: ContributionType
- title: Text
- description: Text
- status: Status
- tags: [Text]
- contributors: [Principal]
- version: Nat
- previousVersionId: ?Text          // Explicit chain: ID of prior version (null for v1)
- previousVersionChecksum: ?Text    // Explicit chain: Checksum of prior version (null for v1)
- confidence: Float (0.0 to 1.0)
- phase: ContributionPhase
- createdAt: Time
- updatedAt: Time
- visibility: Visibility (inherited from space)
```

> [!NOTE]
> **Version Chaining (DEC-085-006)**: The `previousVersionId` and `previousVersionChecksum` fields make version ordering structurally provable rather than merely inferred from the `version` number. Each version cryptographically attests to its predecessor, enabling tamper detection and auditable history chains.

### Contribution Types

| Type | Phase | Purpose |
|------|-------|---------|
| **Idea** | Exploratory | Conceptual seed or hypothesis |
| **Question** | Exploratory | Clarification, inquiry, or challenge |
| **Post** | Exploratory | Contextual signal or update |
| **Comment** | Exploratory | Contextual discussion |
| **Issue** | Deliberative | Problem, risk, or defect |
| **Poll** | Deliberative | Collective choice mechanism |
| **Proposal** | Deliberative | Formal request for change or action |
| **Review** | Deliberative | Formal evaluation or structured critique |
| **Decision** | Decisive | Authoritative resolution |
| **Project** | Executable | Structured execution container |
| **Initiative** | Executable | Multi-project or strategic effort |
| **Deliverable** | Executable | Tangible output |
| **Milestone** | Executable | Progress checkpoint |
| **Bounty** | Executable | Payment offer for completion |
| **Pledge** | Executable | Payment commitment toward a goal |
| **Service** | Executable | Monetized workflow offering |
| **Event** | Executable | Time-bound occurrence |
| **Artifact** | Archival | Knowledge asset (File, Doc, Media) |
| **Report** | Archival | Structured findings and analysis |
| **Reflection** | Archival | Gating thought or introspection |
| **Essay** | Archival | Structured argument or narrative |
| **Institution** | Structural | Named organizational identity with lifecycle |


This abstraction simplifies backend APIs, activity streams, permissions, and graph logic.

### Entity Types (Relationships)
Distinct from contributions, these are reference nodes in the graph:
- **Person**: Non-user identity (e.g., historical figure, deceased relative)
- **Organization**: External entity reference
- **Library**: External data source (e.g., Git Repository)

---

## Contribution Lifecycle

Nostra follows a cyclical model where execution generates new knowledge:

```
Idea
 ├─ receives Questions / Reviews
 ├─ spawns Issues / Proposals
 ├─ forks / merges
 ├─ produces Artifacts / Reports
 └─ evolves into →
      Project
        ├─ Deliverables
        ├─ Milestones
        ├─ Bounties / Pledges
        └─ Comments
              ↓
        Outcomes & Learnings (fed back as Ideas / Essays)
```

**Key Insight**: Nostra is cyclical, not linear. Artifacts and reflections feed back into new ideas. Execution generates new knowledge.

---

## Space Configuration Layer

Each space can define its own collaboration rules:

### Configurable Options
- **Enabled contribution types**: e.g., enable Pledges for fundraising, disable Bounties
- **Allowed transitions**: e.g., Ideas → Projects only by owners
- **Visibility rules**: public, private, member-only
- **Governance model**: (future) voting, quorum, delegation
- **Required metadata fields**: per contribution type

### Space Archetypes
This configuration supports various collaboration contexts:
- **DAO spaces**: Proposals, Votes, Bounties, Services
- **Research spaces**: Ideas, Essays, Reviews, Reports
- **Product teams**: Projects, Issues, Deliverables, Milestones
- **Families/Communities**: Events, Pledges, Artifacts (Photos)

---

## Activity & Graph Layer

### Dual View Architecture
Nostra provides two complementary views of contributions:
1. **Temporal View** (Activity Stream): Chronological stream of all contributions
2. **Relational View** (Contribution Graph): Structural relationships between contributions

### Graph Edges
The contribution graph stores typed relationships:
- `idea → project` (evolves)
- `project → deliverable` (contains)
- `issue → project` (affects)
- `question → idea` (clarifies)
- `review → deliverable` (evaluates)
- `person → artifact` (depicted in)
- `fork → parent` (derives)
- `merge → target` (combines)
- `comment → contribution` (discusses)

### Capabilities Enabled
- **Explore Graph Visualization** (`react-force-graph-2d`): Semantic zooming (macro to micro detail) with Intent-Driven Projections (Story Mode, Density, Lineage, Geographic) dictating spatial representation.
- AI reasoning over knowledge structure
- Recommendation systems
- Governance analytics

---

## Canister Architecture (Modular)

The backend is organized as logical modules, designed for future physical separation:

| Module | Responsibilities |
|--------|-----------------|
| **Space** | Membership, roles, visibility, configuration |
| **Contribution** | CRUD for all contribution types, versioning |
| **Activity** | Unified activity stream / timeline |
| **Interaction** | Votes, follows, lists, reactions |
| **Discussion** | Comments, questions, reflections |
| **Similarity & Tag** | Similarity scoring, tag engagement, discovery |
| **Profile** | User data, visit timestamps, preferences |
| **Governance** | Proposals, voting sessions, tallying, strategy management |

### Benefits
- Matches ICP best practices
- Enables independent scaling
- Supports future inter-canister AI agents
- Clear boundaries for testing and maintenance

---

## AI Integration Readiness

Nostra's structured contribution graph enables future AI capabilities:

### Supported AI Features
- AI-generated summaries per space, idea, or project
- Similarity and merge suggestions
- Automatic roadmap generation
- Detection of duplicate efforts
- Knowledge gap identification
- Governance insights

### Principle
AI agents consume Nostra data — they do not replace human judgment. Initial integration is read-only; controlled write access may be introduced with explicit user approval mechanisms.

---

## Authentication
- Users authenticate using Internet Identity
- All access control is based on principal IDs

## Core Data Models

### Spaces
- Multi-tenant organizations that contain ideas, projects, and initiatives
- Properties: name, description, tags, visibility (public/private)
- Role-based access: owner, member, viewer
- Members are invited by principal ID
- Backend stores all space data and manages permissions
- Backend provides complete CRUD operations: createSpace, getSpaces, getSpaceById, updateSpace, toggleSpaceVisibility
- Backend createSpace function accepts parameters: name, description, tags, visibility and assigns the creator as owner
- Backend enforces proper access control using AccessControl module - only space owners can toggle visibility and manage members
- Backend ensures spaces are properly stored and retrievable through space-scoped queries
- Backend returns structured responses consistent with other contribution types

### Ideas
- Created within a specific space
- Properties: title, abstract, problem statement, proposed solution, tags, status, version, parentId, forkReason, contributors
- Status options: seed, active, archived
- Full version history - each edit creates a new version
- Backend stores all idea data including complete version history
- Backend provides complete CRUD operations: createIdea, getIdeasBySpace, getIdea, updateIdea, getIdeaDetail
- Backend enforces proper access control - only space members can create and update ideas
- Backend ensures ideas are properly linked to spaces and accessible through space-scoped queries

### Projects
- Created within a specific space
- Properties: id, spaceId, title, description, tags, status, contributors, version tracking
- Status options: seed, active, archived
- Full version history - each edit creates a new version
- Backend stores all project data including complete version history
- Backend provides complete CRUD operations: createProject, getProjectsBySpace, getProject, updateProject, getProjectDetail
- Backend enforces proper access control - only space members can create and update projects
- Backend ensures projects are properly linked to spaces and accessible through space-scoped queries
- Backend createProject function accepts parameters: spaceId, title, description, tags, status
- Backend includes contributor details, timestamps, and project milestones/deliverables linkage
- Backend implements permission checks ensuring only space members can create or edit projects




### Initiatives
- Created within a specific space
- Properties: id, spaceId, title, description, tags, status, contributors, version tracking
- Status options: seed, active, archived
- Full version history - each edit creates a new version
- Backend stores all initiative data including complete version history
- Backend provides complete CRUD operations: createInitiative, getInitiativesBySpace, getInitiative, updateInitiative, getInitiativeDetail
- Backend enforces proper access control - only space members can create and update initiatives
- Backend ensures initiatives are properly linked to spaces and accessible through space-scoped queries
- Backend createInitiative function accepts parameters: spaceId, title, description, tags, status
- Backend enforces access control and integrates contributor and version tracking


### Issues
- Problem reports or feature requests within a specific space
- Properties: title, description, priority, status, tags, assignee, spaceId
- Status options: open, in-progress, resolved, closed
- Priority levels: low, medium, high, critical
- Can be assigned to space members
- Backend stores all issue data with proper space linkage
- Backend provides complete CRUD operations: createIssue, getIssuesBySpace, getIssue, updateIssue, getIssueDetail
- Backend ensures issues are properly linked to their respective spaces through spaceId field
- Backend enforces proper access control - only space members can create and update issues
- Backend createIssue function accepts parameters: spaceId, title, description, priority, status, tags, assignee
- Backend ensures each issue tracks creator, timestamps, and related comments and milestones
- Backend ensures each issue tracks creator, timestamps, and related comments and milestones

### Outputs
- Attachments to ideas with structured metadata
- Types: Document, Company, Software, Proposal, Spec
- Properties: type, description, structured fields (JSON format)
- Backend stores all output data linked to ideas

### Project Deliverables
- Attachments to projects with structured metadata
- Types: Document, Software, Report, Presentation, Prototype
- Properties: type, title, description, status, due date, assignee, projectId
- Status options: planned, in-progress, completed, cancelled
- Backend stores all deliverable data linked to projects
- Backend provides complete CRUD operations: createDeliverable, getDeliverablesByProject, getDeliverable, updateDeliverable
- Backend enforces proper access control - only space members can create and update deliverables
- Backend ensures deliverables are properly linked to projects and accessible through project-scoped queries

### Project Milestones
- Key checkpoints and goals within projects
- Properties: title, description, due date, status, completion percentage, projectId
- Status options: upcoming, in-progress, completed, overdue
- Backend stores all milestone data linked to projects
- Backend provides complete CRUD operations: createMilestone, getMilestonesByProject, getMilestone, updateMilestone
- Backend enforces proper access control - only space members can create and update milestones
- Backend ensures milestones are properly linked to projects and accessible through project-scoped queries

### Thoughts
- User reflections on ideas that gate access to comments
- Properties: id, ideaId, user, content, timestamp
- Required before viewing or posting comments on an idea (unless skipped)
- Backend stores thoughts both per-idea and in global user profile for introspection
- Backend tracks when users skip the thought prompt for comment access

### Comments
- Community discussions on ideas, visible only after submitting a thought or skipping the prompt
- Properties: id, ideaId, user, content, timestamp, upvotes, downvotes
- Users can upvote or downvote comments with one vote per user per comment
- Backend stores all comments linked to ideas and manages vote tracking
- Support for replies to comments with one level of nesting depth
- Reply properties: id, parentCommentId, ideaId, user, content, timestamp, upvotes, downvotes
- Backend stores replies linked to parent comments

### Project Comments
- Community discussions on projects
- Properties: id, projectId, user, content, timestamp, upvotes, downvotes
- Users can upvote or downvote comments with one vote per user per comment
- Backend stores all comments linked to projects and manages vote tracking
- Support for replies to comments with one level of nesting depth
- Backend stores replies linked to parent comments

### Issue Comments
- Community discussions on issues
- Properties: id, issueId, user, content, timestamp, upvotes, downvotes
- Users can upvote or downvote comments with one vote per user per comment
- Backend stores all comments linked to issues and manages vote tracking
- Support for replies to comments with one level of nesting depth
- Backend stores replies linked to parent comments

### Questions
- User-submitted questions related to specific ideas or general space questions
- Properties: id, ideaId (optional for space-level questions), spaceId, user, content, timestamp, upvotes, downvotes
- Users can upvote or downvote questions with one vote per user per question
- Backend stores all questions linked to ideas or spaces and manages vote tracking
- Backend supports space-level questions with createSpaceQuestion function accepting spaceId and content parameters
- Backend provides getQuestionsBySpace function to retrieve all questions associated with a given space
- Backend reuses voting, commenting, and threading logic from idea questions for space-level questions

### Question Comments
- Comments specifically on questions, separate from idea-level comments
- Properties: id, questionId, user, content, timestamp, upvotes, downvotes
- Users can upvote or downvote question comments with one vote per user per comment
- Support for replies to question comments with one level of nesting depth
- Backend stores all question comments linked to questions and manages vote tracking

### User Profile Data
- Extended to include personal interaction data:
  - followedIdeas: list of idea IDs the user follows
  - followedProjects: list of project IDs the user follows
  - ideaLists: user-created named lists containing idea IDs
  - projectLists: user-created named lists containing project IDs
  - lastVisitTimestamps: tracks user's last visit time per space for "new/unread" detection
- Backend stores and manages user profile extensions

## Key Features

### Space Management
- Space owners can access settings panel for user management
- User management panel allows adding/removing members and changing roles
- Role assignment: owner, member, viewer with appropriate permissions
- Backend enforces role-based access control for all space operations
- Backend implements complete space management API with createSpace, getSpaces, getSpaceById, updateSpace, toggleSpaceVisibility functions
- Backend space management operations use AccessControl module to restrict visibility toggling and member management to space owners only
- Backend createSpace assigns creator as owner and handles name, description, tags, and visibility parameters
- Backend space operations return structured responses consistent with other contribution types
- Backend ensures space operations sync with frontend activity stream

### Space Summary
- Space detail view displays summary counts for Ideas, Projects, Initiatives, Questions, Issues, Members, Deliverables, and Milestones
- Counts are prominently displayed in header or summary section
- Backend provides aggregated count data for each space including all contribution types

### Space Activity Stream
- Space detail page main content area displays unified stream of all contributions within the space
- Stream includes ideas, questions, issues, projects, initiatives, deliverables, and milestones in chronological order
- Each contribution displays with clear type indicators (icons or labels) and summary content previews
- Stream dynamically updates when new contributions are added within the space
- Backend provides unified activity stream data sorted by timestamp for all contribution types
- Backend ensures all contribution types are properly included in space activity stream through correct spaceId filtering
- Frontend activity stream component properly merges all contribution type data
- Frontend refreshes activity stream queries after new contribution creation to show immediate updates
- Backend ensures all new contribution types (Projects, Initiatives, Issues, Space Questions) integrate with existing space activity streams

### Ideas Management
- Users with member or owner roles can create ideas within spaces
- Ideas support full CRUD operations with proper version tracking
- Backend createIdea function ensures proper spaceId assignment and storage
- Backend getIdeasBySpace function correctly filters and returns ideas for specified space
- Backend getIdeasBySpace function correctly filters and returns ideas for specified space
- Backend linkage verification ensures ideas created via createIdea are retrievable through getIdeasBySpace for the same space

### Projects Management
- Users with member or owner roles can create projects within spaces
- Projects support full CRUD operations with proper version tracking
- Backend createProject function ensures proper spaceId assignment and storage
- Backend getProjectsBySpace function correctly filters and returns projects for specified space
- Backend getProjectsBySpace function correctly filters and returns projects for specified space
- Backend linkage verification ensures projects created via createProject are retrievable through getProjectsBySpace for the same space

### Initiatives Management
- Users with member or owner roles can create initiatives within spaces
- Initiatives support full CRUD operations with proper version tracking
- Backend createInitiative function ensures proper spaceId assignment and storage
- Backend getInitiativesBySpace function correctly filters and returns initiatives for specified space
- Backend getInitiativesBySpace function correctly filters and returns initiatives for specified space
- Backend linkage verification ensures initiatives created via createInitiative are retrievable through getInitiativesBySpace for the same space

### Deliverables Management
- Users with member or owner roles can create deliverables within projects
- Deliverables support full CRUD operations with status and assignment tracking
- Backend createDeliverable function ensures proper projectId assignment and storage
- Backend getDeliverablesByProject function correctly filters and returns deliverables for specified project
- Backend getDeliverablesByProject function correctly filters and returns deliverables for specified project

### Milestones Management
- Users with member or owner roles can create milestones within projects
- Milestones support full CRUD operations with progress and completion tracking
- Backend createMilestone function ensures proper projectId assignment and storage
- Backend getMilestonesByProject function correctly filters and returns milestones for specified project
- Backend getMilestonesByProject function correctly filters and returns milestones for specified project

### Universal Forking (Research 066)
- **Concept**: Forking is a constitutional right. Any Entity (Idea, Project, Space) can be forked to diverge reality.
- **Mechanism**:
    - `fork(entity_id, target_space_id, reason)` -> returns new `entity_id`
    - The new entity retains `fork_from` pointer to the ancestor.
    - History Lineage is preserved.
- **Scope**: Not just Ideas. Spaces themselves can be forked (creating a new community with the same history but different future rules).
- Backend tracks all fork relationships and lineage graph.

### Merge Proposals
- Users can propose merging two ideas
- Proposals include source idea, target idea, and rationale
- Space owners approve or reject merge proposals
- Approved merges combine tags and preserve lineage history
- Backend manages merge proposal workflow and execution

### Issues Management
- Users with member or owner roles can create issues within spaces
- Issues can be assigned to space members
- Issue status and priority can be updated by members and owners
- Backend stores and manages all issue data with proper space linkage
- Backend createIssue function ensures proper spaceId assignment and storage
- Backend getIssuesBySpace function correctly filters and returns issues for specified space
- Backend getIssuesBySpace function correctly filters and returns issues for specified space
- Backend linkage verification ensures issues created via createIssue are retrievable through getIssuesBySpace for the same space

### Project Detail View
- Comprehensive project detail page displaying all project metadata
- Shows title, description, tags, contributors, status, version, creation and update timestamps
- Displays project deliverables with status, due dates, and assignees
- Shows project milestones with completion status and progress tracking
- Lists linked initiatives and related projects within the same space
- Includes discussion section with project-specific comments and replies
- Backend provides complete project data including deliverables, milestones, and relationships

### Issue Detail View
- Comprehensive issue detail page displaying all issue metadata
- Shows title, description, tags, creator, status, priority, assignee, creation and update timestamps
- Includes discussion section with issue-specific comments and replies
- Shows related contributions including linked ideas, deliverables, and milestones
- Displays milestone tracker for progress updates and related deliverables
- Includes context-aware Contribute actions for adding comments, questions, or related ideas
- Backend provides complete issue data including comments, relationships, and progress tracking

### Similarity Engine
- Automatically computes similarity scores between ideas in the same space
- Similarity based on tag overlap and keyword matching (title + abstract)
- Displays "Related Ideas" and "Potential Merge Candidates"
- Backend performs similarity calculations and caching

### Gated Commenting System
- Users must submit a "Thought" or skip the prompt before accessing comments on an idea
- Thoughts prompt modal appears when first opening comments tab with "Submit Thought" and "Skip" options
- Skipping is logged but does not create a thought entry
- After submitting thought or skipping, full comments section becomes available
- Backend enforces thought requirement or skip confirmation before allowing comment access

### Questions System
- Users can post questions related to specific ideas or general space questions
- Questions display visible metrics: upvote count, downvote count, number of comments, and share button
- Questions can be upvoted or downvoted by other users
- Each user can only vote once per question
- Users can comment on questions with full discussion threads
- Question comments support upvoting, downvoting, replying, and sharing
- Backend tracks vote counts and prevents duplicate voting

### Comment and Reply System
- Comments support upvoting, downvoting, replying, and sharing
- Replies are nested one level deep under parent comments
- All comment interactions include consistent iconography and styling
- Backend manages nested comment structure and interaction tracking

### Discussion View
- Unified discussion hub for each idea showing chronological stream of all contributions
- Displays comments, questions, merge proposals, outputs, and thoughts in reverse chronological order
- Provides comprehensive view of all idea-related activity and discussions
- Backend provides unified activity stream data sorted by timestamp

### Idea Interactions
- Users can upvote or downvote ideas with vote tracking to prevent multiple votes
- Users can follow ideas to track updates and activity
- Users can create personal named lists and add ideas to these lists
- Interaction buttons (upvote, downvote, follow, add to list) positioned directly below idea title and version header
- Backend stores interaction data and enforces voting restrictions

### Project Interactions
- Users can upvote or downvote projects with vote tracking to prevent multiple votes
- Users can follow projects to track updates and activity
- Users can create personal named lists and add projects to these lists
- Interaction buttons (upvote, downvote, follow, add to list) positioned directly below project title and version header
- Backend stores project interaction data and enforces voting restrictions

### Idea Contribution Actions
- "Contribute" call-to-action button positioned prominently below the existing interaction buttons
- Expands to show options: Add Comment, Ask Question, Add Proposal, Add Project, Add Initiative, and Report Issue
- Each option opens respective input modal or section using existing forms
- Modal or side-panel components integrate with existing functionality

### Project Contribution Actions
- "Contribute" call-to-action button positioned prominently below the existing interaction buttons
- Expands to show options: Add Comment, Add Deliverable, Add Milestone, Link Initiative, and Report Issue
- Each option opens respective input modal or section using existing forms
- Modal or side-panel components integrate with existing functionality

### Issue Contribution Actions
- "Contribute" call-to-action button positioned prominently below the existing interaction buttons
- Expands to show options: Add Comment, Ask Question, and Add Related Idea
- Each option opens respective input modal or section using existing forms
- Modal or side-panel components integrate with existing functionality

### Space Contribution Actions
- "Contribute" button available in Space View for direct contributions
- Expands to show options: Propose Idea, Add Project, Add Initiative, Ask Question, and Report Issue
- Allows contributions directly from space without entering individual ideas
- Backend handles space-level contributions with proper permission checks
- All contribution types from space properly connect to respective backend APIs with correct spaceId or projectId
- Frontend forms validate and send all required parameters to backend APIs

### Idea Sharing
- "Share Idea" button positioned alongside the Contribute button
- Allows users to copy a public link to the idea if the idea's space is public
- For ideas in private spaces:
  - Button is hidden for non-members, or
  - Displays message: "Only members of this space can view this idea."
- Backend provides space visibility information for share link generation
- Frontend enforces private/public space access logic for shared URLs and idea access routes

### Project Sharing
- "Share Project" button positioned alongside the Contribute button
- Allows users to copy a public link to the project if the project's space is public
- For projects in private spaces:
  - Button is hidden for non-members, or
  - Displays message: "Only members of this space can view this project."
- Backend provides space visibility information for share link generation
- Frontend enforces private/public space access logic for shared URLs and project access routes

### New/Unread Detection
- Ideas, projects, initiatives, deliverables, and milestones created or updated after user's last visit to a space are highlighted as "new" or "unread"
- Visual emphasis using subtle color accents or badges
- Backend tracks user visit timestamps per space for comparison

### Tag Cloud Navigation
- Tag cloud UI displays all tags from ideas, projects, initiatives, deliverables, and milestones within a space as animated bubble elements
- Bubble sizes are dynamic based on each tag's engagement score derived from votes, comments, and follows of associated content
- Smooth scaling animations represent engagement differences between tags
- Clicking tags filters content by selected tag
- Backend provides tag engagement data for dynamic bubble sizing

### Space Visibility Management
- Public/private toggle for spaces visible only to space owners
- Toggle available in space list view and space management interface
- Backend handles space visibility updates and access control

### Contribution Tracking
- Tracks all activities: idea creation, edits, forks, merge proposals, thoughts, comments, questions, votes, skipped prompts, question comments, replies, issues, projects, initiatives, project comments, issue comments, deliverables, milestones
- Maintains per-idea, per-project, per-issue, per-deliverable, and per-milestone contribution logs with contributor lists
- Backend stores complete activity history including all interaction types

## User Interface
- Clean navigation flow: Space list → Ideas/Projects/Initiatives/Issues list → Content details
- Space list includes public/private toggle for owners and visibility indicators
- Space settings panel for user management accessible to owners and admins
- Space detail view includes summary section with counts for Ideas, Projects, Initiatives, Questions, Issues, Members, Deliverables, and Milestones
- Space detail view main content area displays unified activity stream of all contributions with type indicators and content previews
- Space detail view includes animated tag cloud with dynamic bubble sizing for filtering and navigation
- Space view includes Contribute button for proposing ideas, adding projects, adding initiatives, asking questions, and reporting issues directly
- Activity stream dynamically updates when new contributions are added within the space
- Activity stream properly displays all contribution types with immediate refresh after creation
- Ideas, projects, initiatives, deliverables, and milestones in space view highlight new/unread items with visual emphasis
- Idea detail view shows: content with interaction buttons below title, followed by Contribute and Share Idea buttons, version history, outputs, forks, merges, related ideas, comments (gated by thoughts), questions
- Project detail view shows: content with interaction buttons below title, followed by Contribute and Share Project buttons, version history, deliverables, milestones, linked initiatives, related projects, comments
- Issue detail view shows: content with metadata (title, description, tags, creator, status, priority, assignee, timestamps), followed by Contribute button, comments section with reply threads, related contributions (linked ideas, deliverables, milestones), milestone tracker for progress updates
- Discussion View page serves as primary hub showing unified chronological stream of all contributions
- Questions UI displays visible metrics (upvotes, downvotes, comment count) and share button matching design mockup
- Question comments section with threading support and interaction buttons
- Comment and reply system with upvote, downvote, reply, and share options using consistent iconography
- Contribute button expands to show Add Comment, Ask Question, Add Proposal, Add Project, Add Initiative, and Report Issue options
- Project Contribute button expands to show Add Comment, Add Deliverable, Add Milestone, Link Initiative, and Report Issue options
- Issue Contribute button expands to show Add Comment, Ask Question, and Add Related Idea options
- Space Contribute button expands to show Propose Idea, Add Project, Add Initiative, Ask Question, and Report Issue options
- Issues management interface with status, priority, and assignment controls
- Projects and initiatives management interface with status and contributor controls
- Deliverables management interface with status, due date, and assignment controls
- Milestones management interface with progress tracking and completion status controls
- Share Idea button copies public link for public spaces or shows privacy message for private spaces
- Share Project button copies public link for public spaces or shows privacy message for private spaces
- Questions section with upvote/downvote buttons and form to submit new questions
- Modal or dropdown component for managing user lists and adding ideas to lists
- Modal or dropdown component for managing user lists and adding projects to lists
- Thoughts prompt modal with "Submit Thought" and "Skip" options for comment access
- Comments section with list of comments and input for new comments
- Project comments section with list of comments and input for new comments
- Issue comments section with list of comments and input for new comments
- "My Thoughts Log" page in user profile showing all previously written thoughts with links to ideas
- Space management interface for owners to manage members, settings, and visibility
- User management panel with role assignment and member management controls
- Merge proposal interface for creating and reviewing proposals
- Project deliverables management interface with status tracking and assignment
- Project milestones management interface with progress tracking and due dates
- Issue detail page routing at `/issues/:id` with links from space activity stream
- Issue detail page routing at `/issues/:id` with links from space activity stream
- All UI maintains theme compatibility and accessibility standards (see `research/034-nostra-labs/NOSTRA_ACCESSIBILITY_PRINCIPLES.md` and A2UI a11y catalogs)

## Backend Operations
- Store and retrieve spaces, ideas, projects, initiatives, outputs, issues, deliverables, milestones with proper access control
- Implement complete space management API with createSpace, getSpaces, getSpaceById, updateSpace, toggleSpaceVisibility functions
- Backend createSpace function accepts name, description, tags, visibility parameters and assigns creator as owner
- Backend space management operations use AccessControl module to restrict visibility toggling and member management to space owners only
- Backend space operations return structured responses consistent with other contribution types
- Backend ensures space operations sync with frontend activity stream
- Implement space membership management API with addMember, addViewer, removeMember, removeViewer functions
- Manage user roles and permissions within spaces
- Handle space visibility updates and public/private access control
- Process user management operations: adding/removing members, role assignments
- Store and manage ideas with complete CRUD operations: createIdea, getIdeasBySpace, getIdea, updateIdea, getIdeaDetail
- Store and manage projects with complete CRUD operations: createProject, getProjectsBySpace, getProject, updateProject, getProjectDetail
- Store and manage initiatives with complete CRUD operations: createInitiative, getInitiativesBySpace, getInitiative, updateInitiative, getInitiativeDetail
- Store and manage issues with complete CRUD operations: createIssue, getIssuesBySpace, getIssue, updateIssue, getIssueDetail
- Store and manage deliverables with complete CRUD operations: createDeliverable, getDeliverablesByProject, getDeliverable, updateDeliverable
- Store and manage milestones with complete CRUD operations: createMilestone, getMilestonesByProject, getMilestone, updateMilestone
- Backend create functions must properly assign and store spaceId for space-scoped contributions and projectId for project-scoped contributions
- Backend get-by-space and get-by-project functions must correctly filter by respective ID parameters and return accurate results
- Backend linkage verification ensures all contributions created are immediately retrievable through respective query functions
- Backend enforces access control for all operations - only space members can create and update contributions
- Backend storage maps persist all new entities consistently with proper indexing and retrieval logic
- Backend returns public-safe data models without restricted fields for each type when queried via public methods
- Provide space summary data: counts for ideas, projects, initiatives, questions, issues, members, deliverables, and milestones
- Provide unified space activity stream data combining all contribution types sorted by timestamp
- Backend ensures all contribution types are properly included in unified activity stream through correct filtering
- Process versioning and history tracking for ideas, projects, and initiatives
- Handle fork creation and relationship tracking
- Manage merge proposal workflow
- Calculate and cache idea similarity scores
- Calculate tag engagement scores based on votes, comments, and follows of associated content across all contribution types
- Provide tag engagement data for dynamic bubble sizing in tag cloud
- Store and manage thoughts with gated comment access enforcement
- Track and log when users skip thought prompts
- Store and retrieve comments for all contribution types with upvoting, downvoting, and reply functionality
- Store and manage questions with upvote/downvote functionality and comment counts
- Handle space-level questions separate from idea-specific questions
- Store and manage question comments with full threading and interaction support
- Handle comment and reply interactions: upvoting, downvoting, and sharing
- Provide unified discussion stream data combining all contribution types in chronological order
- Handle interactions for all contribution types: upvoting, downvoting, following, and list management
- Maintain user profile data including followed items, personal lists, and visit timestamps
- Track user visit timestamps per space for new/unread detection
- Maintain global thoughts log per user for profile introspection
- Track all user contributions and activities across all contribution types
- Enforce space-scoped permissions for all operations
- Prevent duplicate voting by tracking user vote history per contribution and comment
- Provide space visibility information for sharing functionality
- Validate access permissions for shared URLs based on space privacy settings
- Ensure private content remains accessible only to users with proper space permissions
- Handle space-level contributions with appropriate permission validation
- Backend APIs accept all required parameters for each contribution type

### Governance & Institutions
- **Institutions**: Explicit organizational entities with distinct lifecycles (Emergent -> Operational -> Archived) and stewardship.
    - Linked to spaces via `governs` and other institutions via `derives_from` (lineage).
    - Supports referencing Chaters/Constitutions.
- **Proposals**:
    - The atomic unit of governance. Can trigger code changes, schema updates, or economic actions.
    - Managed by `GovernanceHost`.
- **Voting**:
    - Pluggable voting strategies (`SimpleMajority`, `TokenWeighted`).
    - Configurable logic (Quorums, Thresholds).
- **Storage**:
    - Governance state is persisted in the main backend stable memory, snapshotting active proposals and current strategies.
- Backend returns appropriate success and error responses for all operations
- Backend ensures created contributions are immediately available in activity streams and queries through proper linkage

### Commons (119)

A **Commons** is a portable constitutional bundle — an Institution whose `scope` contains `"commons"`. It carries rules, metadata requirements, governance hooks, and automation workflows that any Space can adopt, fork, or pin to.

- **Convention**: An Institution is a Commons when `scope` contains the `"commons"` marker. No separate entity type; reuses Institution lifecycle, forking, governance, and lineage.
- **Adoption Modes**:
    - `adopted` — Space auto-upgrades with minor Commons version bumps.
    - `pinned` — Space locks to a specific version; manual upgrade required.
    - Expressed via composite edge type: `governs:commons:adopted` or `governs:commons:pinned`.
- **Detachment**: Removing the adoption edge detaches the Space from the Commons.
- **Forking**: A Space can fork a Commons to create a customized variant, tracked via `derives_from` lineage.
- **API Surface**:
    - `adoptCommons(spaceId, commonsId, mode)` — creates adoption edge, requires `manage_space`.
    - `detachCommons(spaceId, commonsId)` — removes adoption edge.
    - `getCommonsForSpace(spaceId)` — returns governing Commons institutions.
    - `listCommons()` — lists all Commons in the system.
    - `getCommonsAdoptions(spaceId)` — returns adoption details (mode, version).
- **Seed**: "Nostra Core Commons v0" is created during bootstrap as the baseline constitutional bundle.
- **Enforcement** (future): Commons rules will be enforced via SIQS `IntegrityRule` definitions (Research 118).
