/**
 * Unit and component tests for the site, in jsdom. Coverage is gated at 95%
 * of lines for everything the site authors except the route files and the
 * server entry, which are framework wiring exercised by the build and the
 * deployed smoke test rather than by unit tests.
 */
import viteReact from "@vitejs/plugin-react";
import { defineConfig } from "vitest/config";

export default defineConfig({
  plugins: [viteReact()],
  test: {
    environment: "jsdom",
    globals: true,
    setupFiles: ["./src/test/setup.ts"],
    include: ["src/**/*.test.{ts,tsx}"],
    coverage: {
      provider: "v8",
      include: ["src/lib/**", "src/components/**", "src/server/release.ts"],
      exclude: ["src/**/*.test.{ts,tsx}", "src/test/**"],
      thresholds: { lines: 95, statements: 95, functions: 95, branches: 90 },
      reporter: ["text", "text-summary"],
    },
  },
});
