/**
 * Tests for useSealed with Node's webcrypto injected: nothing is sealed
 * while off, every text is sealed as base64 (nonce + ciphertext + tag, so
 * size = bytes + 28) while on, new texts are sealed as they appear, a
 * rekey changes the fingerprint and every ciphertext, and a failing crypto
 * leaves the state empty rather than throwing.
 */
import { webcrypto as nodeCrypto } from "node:crypto";
import { renderHook, waitFor } from "@testing-library/react";
import { act } from "react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { NONCE_BYTES, TAG_BYTES } from "../../lib/seal";
import { useSealed } from "./useSealed";

beforeEach(() => vi.stubGlobal("crypto", webcrypto));
afterEach(() => vi.unstubAllGlobals());

const webcrypto = nodeCrypto as unknown as Crypto;
const byteLength = (s: string) => new TextEncoder().encode(s).length;

describe("useSealed", () => {
  it("seals nothing while off", async () => {
    const { result } = renderHook(() => useSealed(false, ["a", "b"]));
    await act(async () => {});
    expect(result.current.sealed.size).toBe(0);
    expect(result.current.sizes.size).toBe(0);
    expect(result.current.fingerprint).toBeNull();
  });

  it("seals every text when on, as base64, with a fingerprint", async () => {
    const texts = ["Start here", "# Welcome\n\nbody", "日本語"];
    const { result } = renderHook(() => useSealed(true, texts));
    await waitFor(() => expect(result.current.sealed.size).toBe(3));
    for (const t of texts) {
      const sealed = result.current.sealed.get(t) as string;
      expect(sealed).toMatch(/^[A-Za-z0-9+/]+=*$/);
      expect(sealed).not.toContain(t);
      expect(result.current.sizes.get(t)).toBe(NONCE_BYTES + byteLength(t) + TAG_BYTES);
      expect(atob(sealed)).toHaveLength(result.current.sizes.get(t) as number);
    }
    expect(result.current.fingerprint).toMatch(/^[0-9a-f]{8}$/);
  });

  it("seals the same text once however often it is given", async () => {
    const { result } = renderHook(() => useSealed(true, ["x", "x", "y"]));
    await waitFor(() => expect(result.current.sealed.size).toBe(2));
  });

  it("seals texts that arrive later under the same key", async () => {
    const { result, rerender } = renderHook(({ texts }) => useSealed(true, texts), { initialProps: { texts: ["one"] } });
    await waitFor(() => expect(result.current.sealed.has("one")).toBe(true));
    const fingerprint = result.current.fingerprint;
    rerender({ texts: ["one", "two"] });
    await waitFor(() => expect(result.current.sealed.has("two")).toBe(true));
    expect(result.current.fingerprint).toBe(fingerprint);
  });

  it("starts sealing when switched on", async () => {
    const { result, rerender } = renderHook(({ on }) => useSealed(on, ["a"]), { initialProps: { on: false } });
    await act(async () => {});
    expect(result.current.sealed.size).toBe(0);
    rerender({ on: true });
    await waitFor(() => expect(result.current.sealed.size).toBe(1));
  });

  it("makes a new key on rekey, so the fingerprint and every ciphertext change", async () => {
    const { result } = renderHook(() => useSealed(true, ["a", "b"]));
    await waitFor(() => expect(result.current.sealed.size).toBe(2));
    const before = { fingerprint: result.current.fingerprint, a: result.current.sealed.get("a") };
    act(() => result.current.rekey());
    await waitFor(() => expect(result.current.fingerprint).not.toBe(before.fingerprint));
    expect(result.current.sealed.get("a")).not.toBe(before.a);
    expect(result.current.sealed.size).toBe(2);
  });

  it("gives a text a different ciphertext each time it is sealed", async () => {
    const { result } = renderHook(() => useSealed(true, ["same"]));
    await waitFor(() => expect(result.current.sealed.size).toBe(1));
    const first = result.current.sealed.get("same");
    act(() => result.current.rekey());
    await waitFor(() => expect(result.current.sealed.get("same")).not.toBe(first));
  });

  it("does not update after it has been switched off or unmounted mid-seal", async () => {
    const { result, unmount } = renderHook(() => useSealed(true, ["a"]));
    unmount();
    await act(async () => {});
    expect(result.current.sealed.size).toBe(0);
  });

  it("does not throw when sealing fails, and stays empty", async () => {
    const failing = {
      getRandomValues: webcrypto.getRandomValues.bind(webcrypto),
      subtle: new Proxy(webcrypto.subtle, {
        get: (target, prop) =>
          prop === "encrypt" ? () => Promise.reject(new Error("no")) : (Reflect.get(target, prop) as (...a: unknown[]) => unknown).bind(target),
      }),
    };
    vi.stubGlobal("crypto", failing);
    const { result } = renderHook(() => useSealed(true, ["a"]));
    await act(async () => {
      await new Promise((r) => setTimeout(r, 20));
    });
    expect(result.current.sealed.size).toBe(0);
    expect(result.current.fingerprint).toBeNull();
  });
});
