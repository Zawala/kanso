/**
 * Kanso's mark as a brush stroke: an ensō drawn once on load, its edge roughened
 * by an SVG turbulence filter so it reads as ink rather than vector, with the
 * vermilion dot set in its opening. Decorative: hidden from assistive tech.
 * The draw-in animation lives in app.css and stops under reduced motion.
 */
import { useId } from "react";
import { cn } from "../lib/cn";

/** The arc's length, a little over its true 1,047 units so the dash fully hides it before drawing. */
const STROKE_LENGTH = 1100;

export interface EnsoProps {
  className?: string;
  /** Draw the stroke in on mount. */
  animate?: boolean;
}

export function Enso({ className, animate = true }: EnsoProps) {
  const filter = `ink-${useId().replace(/:/g, "")}`;
  return (
    <svg viewBox="0 0 520 520" className={cn("overflow-visible", className)} aria-hidden="true" focusable="false">
      <defs>
        <filter id={filter} x="-10%" y="-10%" width="120%" height="120%">
          <feTurbulence type="fractalNoise" baseFrequency="0.9" numOctaves="2" seed="7" result="grain" />
          <feDisplacementMap in="SourceGraphic" in2="grain" scale="7" xChannelSelector="R" yChannelSelector="G" />
        </filter>
      </defs>
      <g filter={`url(#${filter})`} fill="none" stroke="currentColor" strokeLinecap="round">
        {/* Radius 200 about (260, 260), from one o'clock round the left to three o'clock. */}
        <path
          data-testid="enso-stroke"
          className={animate ? "enso-stroke" : undefined}
          style={{ ["--enso-length" as string]: String(STROKE_LENGTH) }}
          d="M 353.9 83.4 A 200 200 0 1 0 459.9 253"
          strokeWidth="34"
        />
        {/* A thinner second pass where the brush pressed hardest. */}
        <path
          className={animate ? "enso-stroke" : undefined}
          style={{ ["--enso-length" as string]: String(STROKE_LENGTH) }}
          d="M 280 72 A 190 190 0 0 0 96 330"
          strokeWidth="14"
          opacity="0.85"
        />
      </g>
      <circle className={cn("fill-shu", animate && "enso-dot")} cx="465" cy="127" r="17" />
    </svg>
  );
}

/** The vermilion seal, 簡素, pressed onto the page after the stroke. */
export function Seal({ className }: { className?: string }) {
  return (
    <span
      aria-hidden="true"
      className={cn(
        "seal inline-grid size-[4.5rem] -rotate-6 place-items-center rounded-[0.35rem] bg-shu font-display text-[1.45rem] leading-[1.05] font-extrabold text-paper shadow-[inset_0_0_0_3px_var(--shu),inset_0_0_0_5px_color-mix(in_oklch,var(--paper)_70%,transparent)] [writing-mode:vertical-rl]",
        className,
      )}
    >
      簡素
    </span>
  );
}
