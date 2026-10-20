/**
 * A real screenshot of the app, rendered by the app's own snapshot harness,
 * shown in the reader's colour scheme: the light shot by default and the dark
 * one under `prefers-color-scheme: dark`. Captions read like a book's figures.
 */
import { cn } from "../lib/cn";

/** A screenshot in public/shots, by name without the theme prefix or extension. */
export interface ShotSource {
  /** e.g. "split" for light-split.webp and dark-split.webp. */
  readonly name: string;
  /** False when only the light shot exists. */
  readonly dark?: boolean;
  readonly alt: string;
  /** Pixel size of the files, so the browser reserves the space. */
  readonly width: number;
  readonly height: number;
}

export interface ShotProps {
  shot: ShotSource;
  figure?: string;
  caption?: string;
  className?: string;
  /** Load as soon as possible (the hero), instead of when scrolled near. */
  eager?: boolean;
}

export function Shot({ shot, figure, caption, className, eager = false }: ShotProps) {
  return (
    <figure className={cn("m-0", className)}>
      <picture>
        {shot.dark !== false && <source media="(prefers-color-scheme: dark)" srcSet={`/shots/dark-${shot.name}.webp`} />}
        <img
          src={`/shots/light-${shot.name}.webp`}
          alt={shot.alt}
          width={shot.width}
          height={shot.height}
          loading={eager ? "eager" : "lazy"}
          decoding="async"
          {...(eager ? { fetchPriority: "high" as const } : {})}
          className="block h-auto w-full rounded-[0.7rem] bg-paper-deep shadow-window ring-1 ring-rule/70"
        />
      </picture>
      {caption && (
        <figcaption className="mt-4 flex gap-3 text-[0.9375rem] leading-snug text-ink-faint">
          {figure && <span className="shrink-0 font-display font-bold text-ink-soft">{figure}</span>}
          <span>{caption}</span>
        </figcaption>
      )}
    </figure>
  );
}
