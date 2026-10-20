/**
 * Test helper: replaces TanStack Router's <Link> with a plain anchor so
 * components that link between pages render without a router context.
 * Import it for its side effect (vi.mock is hoisted within this module).
 */
import type { ReactNode } from "react";
import { vi } from "vitest";

vi.mock("@tanstack/react-router", () => ({
  Link: ({ to, children, ...rest }: { to: string; children?: ReactNode; className?: string }) => (
    <a href={to} {...rest}>
      {children}
    </a>
  ),
}));
