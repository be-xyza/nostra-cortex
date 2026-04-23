# Study 16.2: Claude-Brain Technology Analysis

**Date**: 2026-01-18
**Source**: [memvid/claude-brain](https://github.com/memvid/claude-brain)
**Purpose**: Analyze patterns and architecture from claude-brain to inform the Skills Sync Service design

---

## Executive Summary

Claude-brain is a Claude Code plugin that provides persistent memory across sessions using a single portable `.mv2` file. This study extracts patterns, content structures, and instructions that can directly guide the **Skills Sync Service** architecture.

> [!IMPORTANT]
> **Key Insight**: Claude-brain demonstrates a working model of "skill synchronization" at the observation/memory level. The Skills Sync Service can apply similar patterns at the skills/capability level.

---

## 1. Architecture Patterns

### 1.1 Single-File Portable State

Claude-brain stores all memory in a single `.claude/mind.mv2` file:
- **Portable**: Can be `git commit`, `scp`, or shared
- **Self-contained**: No database, no cloud, no API keys
- **Searchable**: Instant semantic search

**Implication for Skills Sync Service**:
```
.nostra/
├── skills/
│   ├── base.md          # Canonical skills (from Nostra Space)
│   ├── user.md          # User overrides/preferences
│   ├── merged.md        # Merged result (comparable to mind.mv2)
│   └── provenance.json  # Merge history & lineage
```

### 1.2 Hook-Based Lifecycle

Claude-brain uses lifecycle hooks to capture and inject context:

| Hook Event | Purpose | Skills Sync Equivalent |
|:-----------|:--------|:-----------------------|
| `SessionStart` | Inject relevant memories | Sync/merge skills from Nostra Space |
| `PostToolUse` | Capture observations | Capture skill execution results |
| `Stop` | Session summary | Telemetry/feedback submission |

**hooks.json structure**:
```json
{
  "hooks": {
    "SessionStart": [{ "hooks": [{ "type": "command", "command": "..." }] }],
    "PostToolUse": [{ "matcher": "*", "hooks": [...] }],
    "Stop": [{ "hooks": [...] }]
  }
}
```

### 1.3 Observation Type Classification

Claude-brain categorizes observations into semantic types:

| Type | Icon | Description |
|:-----|:-----|:------------|
| `discovery` | 💡 | New information discovered |
| `decision` | 🎯 | Important decisions made |
| `problem` | ⚠️ | Problems or errors encountered |
| `solution` | ✅ | Solutions implemented |
| `pattern` | 🔄 | Patterns recognized |
| `warning` | 🚨 | Warnings or concerns |
| `success` | 🎉 | Successful outcomes |
| `refactor` | 🔧 | Code refactoring done |
| `bugfix` | 🐛 | Bugs fixed |
| `feature` | ✨ | Features added |

**Implication for Skills Sync Service**:
Skills can be typed similarly:
- `capability` - What the agent can do
- `constraint` - Limitations or guardrails
- `preference` - User/agent stylistic preferences
- `knowledge` - Domain-specific knowledge
- `procedure` - Step-by-step processes
- `integration` - External tool/API usage

---

## 2. Data Model

### 2.1 Observation Schema

```typescript
interface Observation {
  id: string;
  timestamp: number;
  type: ObservationType;
  tool?: string;
  summary: string;
  content: string;
  metadata?: {
    files?: string[];
    functions?: string[];
    error?: string;
    confidence?: number;
    tags?: string[];
    sessionId?: string;
  };
}
```

**Skill Schema Proposal** (based on this pattern):
```typescript
interface Skill {
  id: string;
  version: string;
  type: SkillType;
  domain: string;
  summary: string;
  content: string; // The actual SKILL.md content
  metadata: {
    compatibility: string[];      // ["claude", "openai", "gemini"]
    confidence: number;           // Execution success rate
    dependencies?: string[];      // Other skills required
    tags?: string[];
    source?: "canonical" | "community" | "user";
  };
}
```

### 2.2 Context Injection

Claude-brain injects context at session start with smart prioritization:

1. **Recent Activity** - File edits first (most important)
2. **Project-Relevant Memories** - Semantic search by project name
3. **Type-Based Stats** - Quick reference by observation type

**Skills Sync Equivalent**:
1. **Active Skills** - Currently enabled capabilities
2. **Project-Specific Skills** - Skills matched to project domain
3. **User Preferences** - Personal style overrides

### 2.3 Compression Strategy

PostToolUse hook uses "ENDLESS MODE" compression:
- Compresses large outputs to ~500 tokens
- Prioritizes file edits (always captured)
- Skips noisy command outputs
- Max output: 2500 characters

---

## 3. Command Interface

### 3.1 Command Definition Pattern

Commands use markdown with YAML frontmatter:

```markdown
---
description: Ask questions about memories and get context-aware answers
argument-hint: <question>
allowed-tools: Bash
---

# Memory Question

**Usage**: `/mind:ask <question>`

Execute the ask script with user's question:

```bash
node "${CLAUDE_PLUGIN_ROOT}/dist/scripts/ask.js" "$ARGUMENTS"
```
```

### 3.2 Available Commands

| Command | Purpose |
|:--------|:--------|
| `/mind:stats` | Memory statistics and storage info |
| `/mind:search <query>` | Search memories for content |
| `/mind:ask <question>` | Ask questions about memories |
| `/mind:recent [count]` | Show recent activity timeline |

**Skills Sync Commands** (proposed):
| Command | Purpose |
|:--------|:--------|
| `/skills:sync` | Pull and merge skills from Nostra |
| `/skills:search <query>` | Search available skills |
| `/skills:status` | Show sync status and conflicts |
| `/skills:feedback <message>` | Submit skill feedback (telemetry) |

---

## 4. SKILL.md Pattern

### 4.1 File Structure

```markdown
---
name: mind
description: Claude Mind - Search and manage Claude's persistent memory
---

# Claude Mind

You have access to a persistent memory system...

## How to Execute Memory Commands
Use the bundled SDK scripts via Node.js...

## Memory Types
Memories are automatically classified into these types:
- **discovery** - New information discovered
...

## File Location
Your memory is stored at: `.claude/mind.mv2`

## Usage Tips
1. **Start of session**: Recent memories are automatically injected
2. **During coding**: Observations are captured automatically
...
```

### 4.2 Key Components

| Section | Purpose |
|:--------|:--------|
| YAML Frontmatter | Name, description for discovery |
| Context Declaration | What the skill provides |
| Command/Script References | How to use the skill |
| Type Definitions | Categorization of data |
| Usage Tips | Behavioral guidance |

---

## 5. Implications for Skills Sync Service

### 5.1 Architecture Alignment

| Claude-Brain | Skills Sync Service |
|:-------------|:--------------------|
| `.claude/mind.mv2` | Nostra Space (Canonical Registry) |
| Local file persistence | CRUD sync to `~/.agent/skills/` |
| Hook-based capture | Workflow-triggered sync |
| Semantic search | Space-based skill discovery |
| Observation types | Skill types/domains |

### 5.2 Key Patterns to Adopt

1. **Single-File Merged State**
   - The `merged.md` pattern provides a "compiled" skill bundle
   - Provenance tracking (who contributed what)

2. **Hook-Based Automation**
   - `SessionStart` → Skill sync check
   - `PostToolUse` → Skill execution telemetry
   - `Stop` → Aggregate feedback submission

3. **Type Classification**
   - Skills should have semantic types
   - Enables smart merging and prioritization

4. **Frontmatter Metadata**
   - All SKILL.md files should use YAML frontmatter
   - Enables machine-readable discovery

5. **Compression for Telemetry**
   - Telemetry reports should be compressed
   - Focus on high-value observations

### 5.3 Gaps Identified

| Gap | Description | Suggested Resolution |
|:----|:------------|:---------------------|
| **Cloud Sync** | Claude-brain is purely local | Skills Sync Service bridges to Nostra Space |
| **Governance** | No multi-user approval | Nostra Governance for canonical skill updates |
| **Monetization** | No payment model | Subscription + Bounty integration |
| **Semantic Merge** | Simple file storage | LLM-powered merge logic via AgentTask |

---

## 6. Recommended Next Steps

1. **[ ] Define Skill Schema** - Formalize `Skill` interface based on `Observation` pattern
2. **[ ] Design Hook Mapping** - Map claude-brain hooks to Nostra Workflows
3. **[ ] Create Skill Types Taxonomy** - Define skill classification system
4. **[ ] Prototype Merge Logic** - Test semantic skill merging
5. **[ ] Design Telemetry Format** - Standardize feedback reports

---

## 7. Source References

- **Repository**: [memvid/claude-brain](https://github.com/memvid/claude-brain)
- **Key Files**:
  - `skills/mind/SKILL.md` - Main skill definition
  - `hooks/hooks.json` - Hook configuration
  - `src/types.ts` - Type definitions
  - `src/hooks/session-start.ts` - Context injection
  - `src/hooks/post-tool-use.ts` - Observation capture
  - `commands/*.md` - Command definitions
