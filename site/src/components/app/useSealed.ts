/**
 * Server view's sealing: while it's on, every given text is sealed under a
 * key made in this tab (lib/seal) and returned as base64, the way the sync
 * server would hold it. `rekey` makes a new key, so every ciphertext changes.
 */
import { useEffect, useRef, useState } from "react";
import { type DemoKey, newDemoKey, sealDemo, toBase64 } from "../../lib/seal";

export interface Sealed {
  /** Base64 ciphertext for each text sealed so far. */
  readonly sealed: ReadonlyMap<string, string>;
  /** Bytes of ciphertext for each text. */
  readonly sizes: ReadonlyMap<string, number>;
  readonly fingerprint: string | null;
  readonly rekey: () => void;
}

export function useSealed(on: boolean, texts: readonly string[]): Sealed {
  const key = useRef<DemoKey | null>(null);
  const [generation, setGeneration] = useState(0);
  const [state, setState] = useState<Omit<Sealed, "rekey">>({ sealed: new Map(), sizes: new Map(), fingerprint: null });
  const joined = texts.join("\u0000");

  useEffect(() => {
    if (!on) return;
    let live = true;
    const list = joined.split("\u0000");
    (async () => {
      if (key.current === null) key.current = await newDemoKey();
      const k = key.current;
      const out = await Promise.all(list.map((t) => sealDemo(k, t)));
      if (!live) return;
      setState({
        sealed: new Map(list.map((t, i) => [t, toBase64(out[i] as Uint8Array)])),
        sizes: new Map(list.map((t, i) => [t, (out[i] as Uint8Array).length])),
        fingerprint: k.fingerprint,
      });
    })().catch(() => undefined);
    return () => {
      live = false;
    };
  }, [on, joined, generation]);

  return {
    ...state,
    rekey: () => {
      key.current = null;
      setGeneration((g) => g + 1);
    },
  };
}
