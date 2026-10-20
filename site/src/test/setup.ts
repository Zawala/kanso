/**
 * Global test setup: jest-dom matchers, DOM cleanup between tests, and
 * fast-check configured with a pinned seed so property failures reproduce.
 * Override the seed with FAST_CHECK_SEED to explore other inputs.
 */
import "@testing-library/jest-dom/vitest";
import { cleanup } from "@testing-library/react";
import fc from "fast-check";
import { afterEach } from "vitest";

fc.configureGlobal({ seed: Number(process.env.FAST_CHECK_SEED ?? 20261017), numRuns: 200 });

afterEach(cleanup);
