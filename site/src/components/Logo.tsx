/**
 * Kanso's mark, drawn as the app icon draws it: an open ring in the text
 * colour with the vermilion dot in its gap.
 */
import { cn } from "../lib/cn";

export function Logo({ className }: { className?: string }) {
  return (
    <svg viewBox="0 0 32 32" aria-hidden="true" className={cn("size-6", className)}>
      <path d="M 19.9 6.8 A 10 10 0 1 0 25.7 13.6" fill="none" stroke="currentColor" strokeWidth="3" strokeLinecap="round" />
      <circle cx="23.6" cy="9.1" r="1.9" className="fill-dot" />
    </svg>
  );
}
