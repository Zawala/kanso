/**
 * Tests for CipherDemo with Node's webcrypto injected: the ciphertext
 * appears and tracks typing, byte counts follow the layout, "New key"
 * changes the fingerprint, and a browser without Web Crypto shows the
 * failure message instead of crashing.
 */
import { webcrypto } from "node:crypto";
import { act, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it } from "vitest";
import { NONCE_BYTES, TAG_BYTES } from "../lib/seal";
import { CipherDemo, SAMPLE_NOTE } from "./CipherDemo";

const crypto = webcrypto as unknown as Crypto;
const cipher = () => screen.getByTestId("ciphertext");
const fingerprint = () => screen.getByText(/^[0-9a-f]{8}$|^·+$/);

describe("CipherDemo", () => {
  it("shows sealing… and then the ciphertext with sizes", async () => {
    render(<CipherDemo initialText="hello" crypto={crypto} />);
    expect(screen.getByText("sealing…")).toBeInTheDocument();
    await waitFor(() => expect(cipher().textContent).not.toBe(""));
    expect(screen.getByText("5 bytes")).toBeInTheDocument();
    expect(screen.getByText(`${NONCE_BYTES + 5 + TAG_BYTES} bytes`)).toBeInTheDocument();
    expect(cipher().textContent).not.toContain("hello");
    expect(fingerprint().textContent).toMatch(/^[0-9a-f]{8}$/);
  });

  it("starts with the sample note when none is given", async () => {
    render(<CipherDemo crypto={crypto} />);
    expect(screen.getByRole("textbox")).toHaveValue(SAMPLE_NOTE);
    await waitFor(() => expect(cipher().textContent).not.toBe(""));
  });

  it("re-seals as you type", async () => {
    const user = userEvent.setup();
    render(<CipherDemo initialText="a" crypto={crypto} />);
    await waitFor(() => expect(cipher().textContent).not.toBe(""));
    const before = cipher().textContent;
    await user.type(screen.getByRole("textbox"), "bcd");
    await waitFor(() => expect(screen.getByText(`${NONCE_BYTES + 4 + TAG_BYTES} bytes`)).toBeInTheDocument());
    expect(screen.getByRole("textbox")).toHaveValue("abcd");
    expect(cipher().textContent).not.toBe(before);
  });

  it("makes a new key when asked, changing the fingerprint", async () => {
    const user = userEvent.setup();
    render(<CipherDemo initialText="note" crypto={crypto} />);
    await waitFor(() => expect(fingerprint().textContent).toMatch(/^[0-9a-f]{8}$/));
    const first = fingerprint().textContent;
    const sealedFirst = cipher().textContent;
    await user.click(screen.getByRole("button", { name: /New key/ }));
    await waitFor(() => expect(fingerprint().textContent).not.toBe(first));
    await waitFor(() => expect(cipher().textContent).not.toBe(sealedFirst));
  });

  it("shows an explanation when the key cannot be made", async () => {
    const broken = { getRandomValues: () => { throw new Error("no"); }, subtle: undefined } as unknown as Crypto;
    render(<CipherDemo initialText="x" crypto={broken} />);
    await act(async () => {});
    await waitFor(() => expect(cipher()).toHaveTextContent(/needs the Web Crypto API/));
    expect(cipher()).toHaveClass("text-shu");
  });

  it("shows the explanation when sealing fails", async () => {
    let calls = 0;
    const flaky = new Proxy(crypto, {
      get(target, prop) {
        if (prop === "subtle") {
          const subtle = target.subtle;
          return new Proxy(subtle, {
            get(s, p) {
              if (p === "encrypt") return () => Promise.reject(new Error(`fail ${++calls}`));
              const v = Reflect.get(s, p) as unknown;
              return typeof v === "function" ? (v as (...a: unknown[]) => unknown).bind(s) : v;
            },
          });
        }
        const v = Reflect.get(target, prop) as unknown;
        return typeof v === "function" ? (v as (...a: unknown[]) => unknown).bind(target) : v;
      },
    }) as Crypto;
    render(<CipherDemo initialText="x" crypto={flaky} />);
    await waitFor(() => expect(cipher()).toHaveTextContent(/needs the Web Crypto API/));
  });

  it("drops stale results when typing outpaces sealing", async () => {
    const user = userEvent.setup();
    render(<CipherDemo initialText="" crypto={crypto} />);
    await waitFor(() => expect(cipher().textContent).not.toBe(""));
    await user.type(screen.getByRole("textbox"), "quick brown fox");
    const bytes = NONCE_BYTES + "quick brown fox".length + TAG_BYTES;
    await waitFor(() => expect(screen.getByText(`${bytes} bytes`)).toBeInTheDocument());
  });
});
