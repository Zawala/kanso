/**
 * Tests for Chapters: every CHAPTERS entry renders through ChapterView with
 * its title, number, points and a captioned screenshot, and each layout
 * variant (including flipped "beside" and "wide") renders its shots.
 */
import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { CHAPTERS, type Chapter, ChapterView } from "./Chapters";

const shot = (name: string) => ({ name, alt: `alt ${name}`, width: 100, height: 50 });
const base = { id: "x", number: "09", name: "X", title: "A synthetic title.", body: "Body text.", points: ["one", "two"] };

describe("CHAPTERS", () => {
  it("has unique ids and numbers", () => {
    expect(new Set(CHAPTERS.map((c) => c.id)).size).toBe(CHAPTERS.length);
    expect(new Set(CHAPTERS.map((c) => c.number)).size).toBe(CHAPTERS.length);
  });

  it.each(CHAPTERS.map((c) => [c.id, c] as const))("renders chapter %s", (_id, chapter) => {
    const { container } = render(<ChapterView chapter={chapter} />);
    expect(container.querySelector(`article#${chapter.id}`)).not.toBeNull();
    expect(screen.getByText(chapter.title)).toBeInTheDocument();
    for (const point of chapter.points) expect(screen.getByText(point)).toBeInTheDocument();
    expect(screen.getByText(chapter.layout.caption)).toBeInTheDocument();
    expect(screen.getAllByRole("img").length).toBeGreaterThanOrEqual(1);
  });
});

describe("ChapterView layouts", () => {
  const layouts: Record<string, Chapter["layout"]> = {
    beside: { kind: "beside", shot: shot("a"), caption: "Cap." },
    "beside flipped": { kind: "beside", shot: shot("a"), caption: "Cap.", flip: true },
    wide: { kind: "wide", shot: shot("a"), caption: "Cap." },
    layered: { kind: "layered", shot: shot("a"), over: shot("b"), caption: "Cap." },
    floating: { kind: "floating", shot: shot("a"), caption: "Cap." },
  };

  it.each(Object.entries(layouts))("renders the %s layout", (_name, layout) => {
    render(<ChapterView chapter={{ ...base, layout }} />);
    expect(screen.getByText("A synthetic title.")).toBeInTheDocument();
    expect(screen.getByText("Cap.", { exact: false })).toBeInTheDocument();
    expect(screen.getAllByRole("img")).toHaveLength(layout.kind === "layered" ? 2 : 1);
    expect(screen.getByText("Fig. 09", { exact: false })).toBeInTheDocument();
  });

  it("puts the flipped layout's text on the right", () => {
    const { container } = render(<ChapterView chapter={{ ...base, layout: layouts["beside flipped"] as Chapter["layout"] }} />);
    expect(container.querySelector(".lg\\:order-2")).not.toBeNull();
  });
});
