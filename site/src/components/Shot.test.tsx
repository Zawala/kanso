/**
 * Tests for Shot: the light image with a dark-scheme source, omitted when
 * dark is false; dimensions, lazy/eager loading, and the optional caption.
 */
import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { Shot, type ShotSource } from "./Shot";

const shot: ShotSource = { name: "split", alt: "A split view", width: 2000, height: 1250 };

describe("Shot", () => {
  it("offers a dark source and a light image", () => {
    const { container } = render(<Shot shot={shot} />);
    const source = container.querySelector("source");
    expect(source).toHaveAttribute("media", "(prefers-color-scheme: dark)");
    expect(source).toHaveAttribute("srcset", "/shots/dark-split.webp");
    const img = screen.getByRole("img", { name: "A split view" });
    expect(img).toHaveAttribute("src", "/shots/light-split.webp");
    expect(img).toHaveAttribute("width", "2000");
    expect(img).toHaveAttribute("height", "1250");
    expect(img).toHaveAttribute("loading", "lazy");
    expect(img).not.toHaveAttribute("fetchpriority");
  });

  it("omits the dark source when dark is false", () => {
    const { container } = render(<Shot shot={{ ...shot, dark: false }} />);
    expect(container.querySelector("source")).toBeNull();
    expect(container.querySelector("img")).toBeInTheDocument();
  });

  it("keeps the dark source when dark is explicitly true", () => {
    const { container } = render(<Shot shot={{ ...shot, dark: true }} />);
    expect(container.querySelector("source")).not.toBeNull();
  });

  it("loads eagerly with high priority when eager", () => {
    render(<Shot shot={shot} eager />);
    const img = screen.getByRole("img");
    expect(img).toHaveAttribute("loading", "eager");
    expect(img).toHaveAttribute("fetchpriority", "high");
  });

  it("renders a figcaption with the figure label when given", () => {
    render(<Shot shot={shot} figure="Fig. 01" caption="The split view." />);
    const caption = screen.getByText("The split view.").closest("figcaption");
    expect(caption).toHaveTextContent("Fig. 01");
    expect(caption).toHaveTextContent("The split view.");
  });

  it("renders a caption without a figure label, and no caption without text", () => {
    const { container, rerender } = render(<Shot shot={shot} caption="Only words." />);
    expect(container.querySelector("figcaption")?.querySelectorAll("span")).toHaveLength(1);
    rerender(<Shot shot={shot} figure="Fig. 02" />);
    expect(container.querySelector("figcaption")).toBeNull();
  });
});
