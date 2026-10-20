/**
 * The home page's proof of end-to-end encryption: what you type on the left is
 * sealed in this tab as you type, and the right shows exactly what a server
 * would store. Nothing here touches the network. Sealing is asynchronous, so
 * each result carries the input's revision and stale results are dropped.
 */
import { KeyRound, RefreshCw } from "lucide-react";
import { useCallback, useEffect, useRef, useState } from "react";
import { cn } from "../lib/cn";
import { type DemoKey, newDemoKey, sealDemo, toBase64 } from "../lib/seal";

export const SAMPLE_NOTE = `# Sunday

- [ ] Call Mum about the trip @2026-10-25
- [ ] Book the plumber
- Ideas for the [[Garden plan]]: tomatoes along the south wall

The spare key is under the blue pot.`;

interface Sealed {
  readonly text: string;
  readonly bytes: number;
}

export interface CipherDemoProps {
  /** Seeds the note; tests pass short text. */
  initialText?: string;
  /** Injected for tests. */
  crypto?: Crypto;
}

export function CipherDemo({ initialText = SAMPLE_NOTE, crypto = globalThis.crypto }: CipherDemoProps) {
  const [text, setText] = useState(initialText);
  const [key, setKey] = useState<DemoKey | null>(null);
  const [sealed, setSealed] = useState<Sealed | null>(null);
  const [failed, setFailed] = useState(false);
  const revision = useRef(0);

  const rekey = useCallback(() => {
    newDemoKey(crypto).then(setKey, () => setFailed(true));
  }, [crypto]);

  useEffect(rekey, [rekey]);

  useEffect(() => {
    if (key === null) return;
    const mine = ++revision.current;
    sealDemo(key, text, crypto).then(
      (bytes) => {
        if (mine === revision.current) setSealed({ text: toBase64(bytes), bytes: bytes.length });
      },
      () => setFailed(true),
    );
  }, [key, text, crypto]);

  const plainBytes = new TextEncoder().encode(text).length;

  return (
    <div className="grid gap-px overflow-hidden rounded-[0.9rem] bg-sumi-rule lg:grid-cols-2">
      <div className="flex flex-col bg-paper text-ink">
        <div className="flex items-center justify-between border-b border-rule px-5 py-3.5 sm:px-7">
          <label htmlFor="cipher-note" className="type-eyebrow text-ink-soft">
            Your note · on your computer
          </label>
          <span className="text-[0.8125rem] text-ink-faint tabular-nums">{plainBytes} bytes</span>
        </div>
        <textarea
          id="cipher-note"
          value={text}
          onChange={(e) => setText(e.target.value)}
          spellCheck={false}
          rows={11}
          className="min-h-[18rem] flex-1 resize-none bg-transparent px-5 py-5 font-mono text-[0.9375rem] leading-[1.75] text-ink outline-none focus-visible:outline-none sm:px-7"
        />
      </div>

      <div className="flex flex-col bg-sumi-raised text-on-sumi">
        <div className="flex items-center justify-between border-b border-sumi-rule px-5 py-3.5 sm:px-7">
          <span className="type-eyebrow text-on-sumi-soft">What the server stores</span>
          <span className="text-[0.8125rem] text-on-sumi-soft tabular-nums">
            {sealed ? `${sealed.bytes} bytes` : "sealing…"}
          </span>
        </div>
        <p
          data-testid="ciphertext"
          aria-label="The note, encrypted"
          className={cn(
            "min-h-[18rem] flex-1 px-5 py-5 font-mono text-[0.9375rem] leading-[1.75] break-all text-on-sumi/85 sm:px-7",
            failed && "text-shu",
          )}
        >
          {failed ? "This browser couldn't seal the note: it needs the Web Crypto API (any current browser has it)." : sealed?.text}
        </p>
        <div className="flex flex-wrap items-center justify-between gap-3 border-t border-sumi-rule px-5 py-3.5 text-[0.8125rem] text-on-sumi-soft sm:px-7">
          <span className="inline-flex items-center gap-2">
            <KeyRound className="size-4 text-shu" aria-hidden="true" />
            Key <span className="font-mono text-on-sumi tabular-nums">{key ? key.fingerprint : "········"}</span> never leaves this
            tab
          </span>
          <button
            type="button"
            onClick={rekey}
            className="inline-flex items-center gap-2 rounded-full border border-sumi-rule px-3.5 py-1.5 text-on-sumi transition-colors duration-200 hover:border-on-sumi-soft"
          >
            <RefreshCw className="size-3.5" aria-hidden="true" /> New key
          </button>
        </div>
      </div>
    </div>
  );
}
