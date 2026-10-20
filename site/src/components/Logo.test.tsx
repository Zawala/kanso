/**
 * Tests for Logo: the mark is decorative (hidden from assistive tech), draws
 * the ring and the dot, and merges a caller's className over its default size.
 */
import { render } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { Logo } from "./Logo";

describe("Logo", () => {
  it("draws a ring and a dot, hidden from assistive technology", () => {
    const { container } = render(<Logo />);
    const svg = container.querySelector("svg");
    expect(svg).toHaveAttribute("aria-hidden", "true");
    expect(svg).toHaveClass("size-6");
    expect(svg?.querySelectorAll("path")).toHaveLength(1);
    expect(svg?.querySelectorAll("circle")).toHaveLength(1);
  });

  it("lets className replace the default size", () => {
    const { container } = render(<Logo className="size-10 text-red-500" />);
    const svg = container.querySelector("svg");
    expect(svg).toHaveClass("size-10", "text-red-500");
    expect(svg).not.toHaveClass("size-6");
  });
});
