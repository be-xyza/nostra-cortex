import React from "react";
import { NdlBadge } from "./NdlBadge";

interface NdlMetadataBlockProps {
    typeIndicator?: string;
    versionChain: string;
    phase: string;
    confidence: number;
    authorityScope: string;
    compact?: boolean;
}

export function NdlMetadataBlock({
    typeIndicator,
    versionChain,
    phase,
    confidence,
    authorityScope,
    compact = false
}: NdlMetadataBlockProps) {
    const confidenceVariant = confidence > 80 ? "ok" : confidence > 50 ? "warn" : "bad";
    const phaseVariant = phase.toLowerCase() === "production" ? "ok" : phase.toLowerCase() === "alpha" ? "bad" : "warn";

    return (
        <div
            className={`flex flex-wrap items-center gap-1.5 text-xs ${compact ? "pb-0 mb-1 mt-[-6px]" : "pb-1 mb-2 mt-[-4px]"}`}
            role="group"
            aria-label="Contribution Metadata"
        >
            {typeIndicator && (
                <span
                    className={`${compact ? "text-[8px] px-1.5" : "text-[10px] px-2.5"} uppercase font-bold tracking-widest py-0.5 rounded-full bg-blue-500/10 text-blue-400`}
                    title="Type Indicator"
                >
                    {typeIndicator}
                </span>
            )}
            <span className={`font-mono ${compact ? "text-[8px]" : "text-[10px]"} text-slate-500 ml-0.5 mr-0.5`} title="Version Chain">
                {versionChain}
            </span>
            <NdlBadge label={phase} variant={phaseVariant} compact={compact} />
            <NdlBadge label={`${confidence}% Conf`} variant={confidenceVariant} compact={compact} />
            {!compact && <NdlBadge label={authorityScope} variant="neutral" />}
        </div>
    );
}
