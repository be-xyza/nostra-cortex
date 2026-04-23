---
id: '006'
name: sentiment-capture-patterns
title: 'Requirements: Sentiment Capture Patterns'
type: general
project: nostra
status: draft
authors:
- User
tags: []
created: '2026-01-28'
updated: '2026-01-28'
---

# Requirements: Sentiment Capture Patterns

## Functional Requirements
| ID | Requirement | Description | Priority |
|----|-------------|-------------|----------|
| REQ-001 | **Emoji Reaction** | Users can react to entities with emojis. | High |
| REQ-002 | **Emoji Limit** | The interface should feature a limited set (max 3) of primary emojis suitable for the context (e.g., ❤️, 👍, 🔥). | High |
| REQ-003 | **Auto-Detection** | (Option A) System detects emojis in user comments/posts and promotes them to reactions. | Medium |
| REQ-004 | **Manual Selection** | (Option B) Users manually select from the 3 curated emojis. | High |
| REQ-005 | **Upvoting** | Support for simple binary voting (up/down or just up). | Medium |
| REQ-006 | **Polls** | Ability to attach polls to entities. | Low |


## Technical Requirements
| ID | Requirement | Description | Priority |
|----|-------------|-------------|----------|
| TR-001 | **Graph Storage** | Store reactions as directed edges: `(User)-[:REACTED_WITH]->(Entity)`. | Must |
| TR-002 | **Materialized View** | Maintain a cached `score` property on Entities for O(1) sorting/retrieval. | Should |
| TR-003 | **Regex Matcher** | Implement a Motoko-based parsing function to detect: ❤️, 👍, 🔥, 👎, 💩, etc. | Must |
| TR-004 | **Aggregation** | System must handle concurrent reaction updates without locking the Entity. | Must |

## Technical Constraints
- **Backend**: Motoko (KG Data Canister).
- **Storage**: Must fit within the graph schema (likely as Relationships or weighted Edges).
- **Performance**: Aggregation of votes/reactions must be efficient.
