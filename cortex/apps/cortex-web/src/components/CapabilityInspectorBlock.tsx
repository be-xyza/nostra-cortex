import React from 'react';
import { HeapBlockCard } from './heap/HeapBlockCard';
import type { CapabilityNode, HeapBlockListItem } from '../contracts';

interface CapabilityInspectorBlockProps {
  node: CapabilityNode;
}

/**
 * CapabilityInspectorBlock
 * 
 * A bridge component that projects CapabilityNode metadata into the 
 * standardized HeapBlockListItem contract for high-fidelity rendering.
 */
export const CapabilityInspectorBlock: React.FC<CapabilityInspectorBlockProps> = ({ node }) => {
  // Map CapabilityNode metadata to HeapBlockListItem structure
  const blockType = node.domain === "pattern" ? "pattern" : "capability";
  const block: HeapBlockListItem = {
    projection: {
      artifactId: node.id,
      title: node.title,
      blockType, // Use native systemic contextual blocks
      updatedAt: new Date().toISOString(),
      tags: node.domain ? [node.domain] : [],
      mentionsInline: [],
      attributes: {
        intent: node.intent_type,
        role: node.required_role || "none",
        status: node.promotion_status || "development",
        health: node.health || "healthy",
        visibility: node.visibility_state || "visible",
        variance: node.variance || "stable",
        governance: node.promotion_status || "stewardship",
        surfacing: node.surfacing_heuristic || node.inspector?.surfacing_heuristic || "unspecified",
        frequency: node.operational_frequency || node.inspector?.operational_frequency || "unspecified",
        ...(node.inspector?.placement_constraint?.preferredNavBand ? { navBand: node.inspector.placement_constraint.preferredNavBand } : {}),
      },
    },
    surfaceJson: {
      text: node.description,
      // Pass invariant violations as metadata for the PayloadRenderer if needed
      invariants: node.invariant_violations,
      promotion: node.promotion_status,
      promotion_logic: node.inspector?.promotion_status
    },
    warnings: node.invariant_violations?.map(v => v.message || "Requirement not met"),
  };

  return (
    <div className="capability-inspector-block">
      <HeapBlockCard
        block={block}
        isSelected={true}
        onClick={() => {}}
        onDoubleClick={() => {}}
        // In the future, we could add specific cardActions here 
        // to migrate/promote/lock nodes directly from the inspector
      />
    </div>
  );
};
