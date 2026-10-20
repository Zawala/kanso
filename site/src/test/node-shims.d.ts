/**
 * Minimal ambient types for the few Node globals the tests use, since
 * @types/node is not a dependency of the site: webcrypto, Buffer and process.
 */
declare module "node:crypto" {
  export const webcrypto: unknown;
}

declare const Buffer: {
  from(bytes: Uint8Array): { toString(encoding: "base64" | "hex"): string };
};

declare const process: { env: Record<string, string | undefined> };
