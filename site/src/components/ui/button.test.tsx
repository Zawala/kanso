/**
 * Tests for Button: default and explicit variants and sizes map to their
 * classes, className overrides merge, props pass through, and asChild styles
 * the child element (a link) instead of rendering a button.
 */
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import { Button } from "./button";

describe("Button", () => {
  it("defaults to the primary medium button", () => {
    render(<Button>Go</Button>);
    expect(screen.getByRole("button", { name: "Go" })).toHaveClass("bg-ink", "h-11");
  });

  it.each([
    ["primary", "bg-ink"],
    ["outline", "border-rule"],
    ["ghost", "text-ink-soft"],
    ["onSumi", "bg-on-sumi"],
  ] as const)("applies the %s variant", (variant, cls) => {
    render(<Button variant={variant}>Go</Button>);
    expect(screen.getByRole("button")).toHaveClass(cls);
  });

  it.each([
    ["sm", "h-9"],
    ["md", "h-11"],
    ["lg", "h-14"],
  ] as const)("applies the %s size", (size, cls) => {
    render(<Button size={size}>Go</Button>);
    expect(screen.getByRole("button")).toHaveClass(cls);
  });

  it("lets className override defaults", () => {
    render(<Button className="h-20 ml-1">Go</Button>);
    const b = screen.getByRole("button");
    expect(b).toHaveClass("h-20", "ml-1");
    expect(b).not.toHaveClass("h-11");
  });

  it("passes props and handles clicks and disabled", async () => {
    const onClick = vi.fn();
    const { rerender } = render(<Button onClick={onClick} type="submit">Go</Button>);
    await userEvent.click(screen.getByRole("button"));
    expect(onClick).toHaveBeenCalledTimes(1);
    expect(screen.getByRole("button")).toHaveAttribute("type", "submit");
    rerender(<Button disabled onClick={onClick}>Go</Button>);
    expect(screen.getByRole("button")).toBeDisabled();
  });

  it("styles the child instead of rendering a button with asChild", () => {
    render(
      <Button asChild variant="outline" size="lg">
        <a href="/download">Download</a>
      </Button>,
    );
    expect(screen.queryByRole("button")).toBeNull();
    const link = screen.getByRole("link", { name: "Download" });
    expect(link).toHaveAttribute("href", "/download");
    expect(link).toHaveClass("border-rule", "h-14");
  });
});
