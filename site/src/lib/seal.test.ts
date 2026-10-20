/**
 * Tests for the browser-side demo sealing: round-trip, wire layout,
 * tamper and wrong-key rejection, nonce freshness, and the hex/base64
 * encoders (base64 checked against Node's Buffer as an oracle). Node's
 * webcrypto is passed explicitly so jsdom's crypto doesn't matter.
 */
import { webcrypto } from "node:crypto";
import fc from "fast-check";
import { describe, expect, it } from "vitest";
import { NONCE_BYTES, TAG_BYTES, newDemoKey, openDemo, sealDemo, toBase64, toHex } from "./seal";

const crypto = webcrypto as unknown as Crypto;

describe("newDemoKey", () => {
  it("makes a non-extractable key with an eight digit hex fingerprint", async () => {
    const k = await newDemoKey(crypto);
    expect(k.fingerprint).toMatch(/^[0-9a-f]{8}$/);
    expect(k.key.extractable).toBe(false);
  });

  it("makes a different key each time", async () => {
    const a = await newDemoKey(crypto);
    const b = await newDemoKey(crypto);
    expect(a.fingerprint).not.toBe(b.fingerprint);
  });
});

describe("sealDemo / openDemo", () => {
  it("round-trips any string", async () => {
    const key = await newDemoKey(crypto);
    await fc.assert(
      fc.asyncProperty(fc.oneof(fc.string(), fc.string({ unit: "binary" })), async (text) => {
        // Lone surrogates don't survive UTF-8, so compare against the encoder's own view.
        const expected = new TextDecoder().decode(new TextEncoder().encode(text));
        expect(await openDemo(key, await sealDemo(key, text, crypto), crypto)).toBe(expected);
      }),
      { numRuns: 100 },
    );
  });

  it("seals to nonce + utf8 length + tag bytes", async () => {
    const key = await newDemoKey(crypto);
    await fc.assert(
      fc.asyncProperty(fc.string({ unit: "grapheme" }), async (text) => {
        const sealed = await sealDemo(key, text, crypto);
        expect(sealed.length).toBe(NONCE_BYTES + new TextEncoder().encode(text).length + TAG_BYTES);
      }),
      { numRuns: 100 },
    );
  });

  it("rejects a tampered byte anywhere in the sealed bytes", async () => {
    const key = await newDemoKey(crypto);
    await fc.assert(
      fc.asyncProperty(fc.string({ minLength: 1 }), fc.nat(), async (text, at) => {
        const sealed = await sealDemo(key, text, crypto);
        const copy = new Uint8Array(sealed);
        const i = at % copy.length;
        copy[i] = (copy[i] ?? 0) ^ 0x01;
        await expect(openDemo(key, copy, crypto)).rejects.toThrow();
      }),
      { numRuns: 60 },
    );
  });

  it("rejects a different key", async () => {
    const a = await newDemoKey(crypto);
    const b = await newDemoKey(crypto);
    await expect(openDemo(b, await sealDemo(a, "secret", crypto), crypto)).rejects.toThrow();
  });

  it("uses a fresh nonce, so equal text seals differently", async () => {
    const key = await newDemoKey(crypto);
    const one = await sealDemo(key, "same", crypto);
    const two = await sealDemo(key, "same", crypto);
    expect(toHex(one)).not.toBe(toHex(two));
    expect(toHex(one.subarray(0, NONCE_BYTES))).not.toBe(toHex(two.subarray(0, NONCE_BYTES)));
  });

  it("works with the default global crypto when it is available", async () => {
    if (!globalThis.crypto?.subtle) return;
    const key = await newDemoKey();
    expect(await openDemo(key, await sealDemo(key, "hi"))).toBe("hi");
  });
});

describe("encoders", () => {
  it("toHex examples", () => {
    expect(toHex(new Uint8Array([]))).toBe("");
    expect(toHex(new Uint8Array([0, 1, 15, 16, 255]))).toBe("00010f10ff");
  });

  it("toHex is two lowercase hex digits per byte", () => {
    fc.assert(
      fc.property(fc.uint8Array(), (bytes) => {
        const hex = toHex(bytes);
        expect(hex).toMatch(/^([0-9a-f]{2})*$/);
        expect(hex.length).toBe(bytes.length * 2);
        expect(hex).toBe(Buffer.from(bytes).toString("hex"));
      }),
    );
  });

  it("toBase64 matches Buffer's", () => {
    expect(toBase64(new Uint8Array([]))).toBe("");
    expect(toBase64(new Uint8Array([104, 105]))).toBe("aGk=");
    fc.assert(
      fc.property(fc.uint8Array(), (bytes) => {
        expect(toBase64(bytes)).toBe(Buffer.from(bytes).toString("base64"));
      }),
    );
  });
});
