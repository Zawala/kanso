/**
 * Tests for the decorative marks: Enso draws with animation classes unless
 * animate is false, stays hidden from assistive tech, and gets unique filter
 * ids; Seal renders its characters hidden from assistive tech.
 */
import { render } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { Enso, Seal } from "./Enso";

describe("Enso", () => {
  it("renders the stroke with its animation class and a dot", () => {
    const { container, getByTestId } = render(<Enso className="size-9" />);
    expect(getByTestId("enso-stroke")).toHaveClass("enso-stroke");
    expect(container.querySelectorAll(".enso-stroke")).toHaveLength(2);
    expect(container.querySelector("circle")).toHaveClass("enso-dot", "fill-shu");
    expect(container.querySelector("svg")).toHaveAttribute("aria-hidden", "true");
    expect(container.querySelector("svg")).toHaveClass("size-9", "overflow-visible");
  });

  it("drops the animation classes when animate is false", () => {
    const { container, getByTestId } = render(<Enso animate={false} />);
    expect(getByTestId("enso-stroke")).not.toHaveClass("enso-stroke");
    expect(container.querySelector(".enso-stroke")).toBeNull();
    expect(container.querySelector("circle")).not.toHaveClass("enso-dot");
  });

  it("points its group at its own filter, unique per instance", () => {
    const { container } = render(
      <>
        <Enso />
        <Enso />
      </>,
    );
    const ids = [...container.querySelectorAll("filter")].map((f) => f.id);
    expect(new Set(ids).size).toBe(2);
    for (const [i, g] of [...container.querySelectorAll("g")].entries()) expect(g).toHaveAttribute("filter", `url(#${ids[i]})`);
  });
});

describe("Seal", () => {
  it("shows the seal characters, hidden from assistive tech", () => {
    const { container } = render(<Seal className="extra" />);
    const seal = container.querySelector("span");
    expect(seal).toHaveTextContent("簡素");
    expect(seal).toHaveAttribute("aria-hidden", "true");
    expect(seal).toHaveClass("seal", "extra");
  });
});
