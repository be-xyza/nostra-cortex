import type { Comment } from "../../store/uiStore.ts";

export type HeapCommentPersistenceAuthority = "local_ui_state";

export interface HeapCommentAuthorityContract {
  persistence: HeapCommentPersistenceAuthority;
  durableEvidence: false;
  governedHeapRecord: false;
  exportableAsEvidence: false;
  recommendedPersistenceTarget: "undecided";
}

export const HEAP_COMMENT_AUTHORITY_CONTRACT: HeapCommentAuthorityContract = {
  persistence: "local_ui_state",
  durableEvidence: false,
  governedHeapRecord: false,
  exportableAsEvidence: false,
  recommendedPersistenceTarget: "undecided",
};

export interface BuildLocalHeapCommentOptions {
  id: string;
  author: string;
  text: string;
  createdAt: string;
}

export function buildLocalHeapComment(options: BuildLocalHeapCommentOptions): Comment {
  return {
    id: options.id,
    author: options.author,
    text: options.text.trim(),
    createdAt: options.createdAt,
    authority: HEAP_COMMENT_AUTHORITY_CONTRACT,
  };
}

export function isDurableHeapComment(comment: {
  authority?: {
    durableEvidence?: boolean;
    governedHeapRecord?: boolean;
  };
}): boolean {
  return comment.authority?.durableEvidence === true || comment.authority?.governedHeapRecord === true;
}
