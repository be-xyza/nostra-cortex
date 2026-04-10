/**
 * chatHintRegistry.ts
 *
 * Observable, upgradable hint registry for the Chat panel.
 * Hints are data-driven — never hardcoded in JSX.
 *
 * Future: source from a worker endpoint (GET /api/cortex/studio/chat/hints)
 * or from space-level configuration. The ChatPanel reads from
 * `resolveChatHints()` which can be swapped to an async fetch
 * without touching the component.
 */

export interface ChatHint {
    id: string;
    label: string;
    /** Optional prompt text to pre-fill the chat input */
    prompt?: string;
    /** Which view modes this hint is relevant to (empty = all) */
    viewModes?: string[];
    /** Minimum number of selected blocks for this hint to appear */
    minSelection?: number;
    /** Maximum number of selected blocks for this hint to appear */
    maxSelection?: number;
    /** Priority for ordering (lower = higher priority) */
    priority: number;
}

/**
 * Default hint set — serves as the seed registry.
 * These can be overridden via space config or worker response.
 */
const DEFAULT_HINTS: ChatHint[] = [
    // General (no view / no selection)
    {
        id: "general-status",
        label: "What's new in this space?",
        prompt: "Summarize recent activity in this space",
        viewModes: [],
        maxSelection: 0,
        priority: 10,
    },
    {
        id: "general-summarize",
        label: "Summarize today's activity",
        prompt: "What happened in this space today?",
        viewModes: [],
        maxSelection: 0,
        priority: 20,
    },
    // Tasks view
    {
        id: "tasks-blocking",
        label: "What's blocking progress?",
        prompt: "What tasks are blocked or overdue?",
        viewModes: ["Tasks"],
        maxSelection: 0,
        priority: 10,
    },
    {
        id: "tasks-prioritize",
        label: "Help me prioritize",
        prompt: "Prioritize my open tasks by urgency and impact",
        viewModes: ["Tasks"],
        maxSelection: 0,
        priority: 20,
    },
    // Proposals view
    {
        id: "proposals-walkthrough",
        label: "Walk me through this plan",
        prompt: "Explain the steps in this proposal and any risks",
        viewModes: ["Proposals"],
        minSelection: 1,
        maxSelection: 1,
        priority: 10,
    },
    {
        id: "proposals-compare",
        label: "Compare these proposals",
        prompt: "Compare the selected proposals and recommend which to approve",
        viewModes: ["Proposals"],
        minSelection: 2,
        priority: 15,
    },
    // Selection-based (any view)
    {
        id: "selection-analyze",
        label: "Analyze these together",
        prompt: "Analyze the selected blocks and surface connections",
        viewModes: [],
        minSelection: 2,
        priority: 10,
    },
    {
        id: "selection-summarize",
        label: "Summarize selection",
        prompt: "Summarize the selected blocks into a concise overview",
        viewModes: [],
        minSelection: 1,
        priority: 15,
    },
    // Inbox view
    {
        id: "inbox-triage",
        label: "Help me triage",
        prompt: "Review my inbox and suggest which items need immediate attention",
        viewModes: ["Inbox"],
        maxSelection: 0,
        priority: 10,
    },
    // Drafts view
    {
        id: "drafts-review",
        label: "Review my drafts",
        prompt: "Review my draft blocks and suggest improvements or promotions",
        viewModes: ["Drafts"],
        maxSelection: 0,
        priority: 10,
    },
];

let _hintOverrides: ChatHint[] | null = null;

/**
 * Override the default hints with a custom set (e.g., from a worker response).
 * Pass null to reset to defaults.
 */
export function setChatHints(hints: ChatHint[] | null): void {
    _hintOverrides = hints;
}

/**
 * Returns the current hint set. Reads from overrides if set, otherwise defaults.
 * Future: make this async and fetch from gateway.
 */
export function getChatHints(): ChatHint[] {
    return _hintOverrides ?? DEFAULT_HINTS;
}

/**
 * Resolve applicable hints given current context.
 */
export function resolveChatHints(
    viewMode: string,
    selectionCount: number,
): ChatHint[] {
    return getChatHints()
        .filter((hint) => {
            // View mode filter
            if (hint.viewModes && hint.viewModes.length > 0 && !hint.viewModes.includes(viewMode)) {
                return false;
            }
            // Selection count filters
            if (hint.minSelection !== undefined && selectionCount < hint.minSelection) {
                return false;
            }
            if (hint.maxSelection !== undefined && selectionCount > hint.maxSelection) {
                return false;
            }
            return true;
        })
        .sort((a, b) => a.priority - b.priority)
        .slice(0, 4); // Show at most 4 hints
}
