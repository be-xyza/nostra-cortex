import React, { useId } from "react";
import { BrandLogoProps, resolveBrandVisualState } from "./brandLogoPolicy";

export type {
    BrandMode,
    AuthorityState,
    TemporalState,
    BrandLogoProps,
    ResolvedBrandVisualState,
} from "./brandLogoPolicy";

export { resolveBrandVisualState } from "./brandLogoPolicy";

export function BrandLogo(props: BrandLogoProps) {
    const { size = 100, animating = false } = props;
    const state = resolveBrandVisualState(props);
    const technicalMotion = state.motion.technical;
    const philosophicalMotion = state.motion.philosophical;

    const center = 50;
    const radius = state.ringRadius;
    const circumference = 2 * Math.PI * radius;

    const gapLength = (state.gapAngle / 360) * circumference;
    const solidLength = circumference - gapLength;
    const rotationOffset = -90 + (state.gapAngle / 2);

    const instanceId = useId().replace(/:/g, "");
    const nostraGradientId = `${instanceId}-nostra-grad`;
    const cortexGradientId = `${instanceId}-cortex-grad`;
    const ringAnimationName = `${instanceId}-nostra-breathe`;
    const dotAnimationName = `${instanceId}-cortex-pulse`;

    const technicalMode = state.effectiveMode === "technical";

    return (
        <svg
            xmlns="http://www.w3.org/2000/svg"
            viewBox="0 0 100 100"
            width={size}
            height={size}
            style={{
                transition: technicalMode
                    ? `all ${technicalMotion.containerTransitionMs}ms steps(${technicalMotion.stepCount}, end)`
                    : `all ${philosophicalMotion.containerTransitionSec}s ease-in-out`,
                transform: animating ? "scale(1.05)" : "scale(1)",
            }}
        >
            <defs>
                <linearGradient id={nostraGradientId} x1="0%" y1="0%" x2="100%" y2="100%">
                    <stop offset="0%" stopColor={state.outerGradientTo} />
                    <stop offset="100%" stopColor={state.outerBase} />
                </linearGradient>
                <linearGradient id={cortexGradientId} x1="0%" y1="0%" x2="100%" y2="100%">
                    <stop offset="0%" stopColor={state.innerGradientTo} />
                    <stop offset="100%" stopColor={state.innerBase} />
                </linearGradient>
            </defs>

            <circle
                cx={center}
                cy={center}
                r={radius}
                fill="none"
                stroke={state.isGradient ? `url(#${nostraGradientId})` : state.outerBase}
                strokeWidth={state.strokeWidth}
                strokeLinecap={state.strokeCap}
                strokeDasharray={`${solidLength} ${gapLength}`}
                strokeDashoffset={0}
                transform={`rotate(${rotationOffset} ${center} ${center})`}
                style={{
                    transition: technicalMode
                        ? `stroke-dasharray ${technicalMotion.ringTransitionMs}ms steps(${technicalMotion.ringStepCount}, end), stroke-width ${technicalMotion.strokeTransitionMs}ms steps(${technicalMotion.stepCount}, end)`
                        : `stroke-dasharray ${philosophicalMotion.ringTransitionSec}s ease-in-out, stroke-width ${philosophicalMotion.strokeTransitionSec}s ease`,
                    animation:
                        animating && state.effectiveMode === "philosophical"
                            ? `${ringAnimationName} ${philosophicalMotion.ringAnimationDurationSec}s infinite ease-in-out`
                            : "none",
                }}
            />

            <circle
                cx={center}
                cy={center}
                r={state.dotRadius}
                fill={state.isGradient ? `url(#${cortexGradientId})` : state.innerBase}
                style={{
                    transition: technicalMode
                        ? `fill ${technicalMotion.dotTransitionMs}ms steps(${technicalMotion.stepCount}, end)`
                        : `fill ${philosophicalMotion.dotTransitionSec}s ease`,
                    animation: animating
                        ? `${dotAnimationName} ${philosophicalMotion.dotAnimationDurationSec}s infinite alternate`
                        : "none",
                }}
            />

            <style>
                {`
          @keyframes ${ringAnimationName} {
            0%, 100% { transform: rotate(${rotationOffset}deg); stroke-width: ${state.strokeWidth}px; }
            50% { transform: rotate(${rotationOffset + philosophicalMotion.ringRotationDeltaDeg}deg); stroke-width: ${state.strokeWidth + philosophicalMotion.ringStrokeDeltaPx}px; }
          }
          @keyframes ${dotAnimationName} {
            from { r: ${state.dotRadius}px; opacity: 1; }
            to { r: ${state.dotRadius + philosophicalMotion.dotPulseRadiusDeltaPx}px; opacity: ${philosophicalMotion.dotPulseOpacityMin}; }
          }
        `}
            </style>
        </svg>
    );
}
