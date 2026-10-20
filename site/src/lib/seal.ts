/**
 * Seals text in the browser for the home page's "what the server sees"
 * demonstration, with the Web Crypto API: AES-256-GCM under a random key
 * that is created in the tab, never exported and never sent anywhere.
 * The app's sync seals with XChaCha20-Poly1305; the idea shown is the same.
 */

export const NONCE_BYTES = 12;
export const TAG_BYTES = 16;

/** A key for the demo, with a short fingerprint so a new key is visible. */
export interface DemoKey {
  readonly key: CryptoKey;
  /** Eight hex digits from the key's SHA-256, for display only. */
  readonly fingerprint: string;
}

/** A fresh, non-extractable key. */
export async function newDemoKey(crypto: Crypto = globalThis.crypto): Promise<DemoKey> {
  const raw = crypto.getRandomValues(new Uint8Array(32));
  const digest = new Uint8Array(await crypto.subtle.digest("SHA-256", raw));
  const key = await crypto.subtle.importKey("raw", raw, { name: "AES-GCM" }, false, ["encrypt", "decrypt"]);
  raw.fill(0);
  return { key, fingerprint: toHex(digest.subarray(0, 4)) };
}

/** `nonce || ciphertext || tag`, with a new random nonce every time. */
export async function sealDemo(key: DemoKey, text: string, crypto: Crypto = globalThis.crypto): Promise<Uint8Array<ArrayBuffer>> {
  const nonce = crypto.getRandomValues(new Uint8Array(NONCE_BYTES));
  const sealed = new Uint8Array(await crypto.subtle.encrypt({ name: "AES-GCM", iv: nonce }, key.key, new TextEncoder().encode(text)));
  const out = new Uint8Array(NONCE_BYTES + sealed.length);
  out.set(nonce);
  out.set(sealed, NONCE_BYTES);
  return out;
}

/** The text `sealDemo` sealed; throws if the bytes were changed or the key is another. */
export async function openDemo(key: DemoKey, sealed: Uint8Array<ArrayBuffer>, crypto: Crypto = globalThis.crypto): Promise<string> {
  const nonce = sealed.subarray(0, NONCE_BYTES);
  const body = sealed.subarray(NONCE_BYTES);
  const plain = await crypto.subtle.decrypt({ name: "AES-GCM", iv: nonce }, key.key, body);
  return new TextDecoder().decode(plain);
}

export function toHex(bytes: Uint8Array): string {
  return Array.from(bytes, (b) => b.toString(16).padStart(2, "0")).join("");
}

export function toBase64(bytes: Uint8Array): string {
  let binary = "";
  for (const b of bytes) binary += String.fromCharCode(b);
  return btoa(binary);
}
