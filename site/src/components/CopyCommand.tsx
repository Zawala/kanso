/**
 * A shell command shown as text with a copy button. Copying uses the
 * Clipboard API and says when it worked or couldn't; the command stays
 * selectable either way.
 */
import { Check, Copy } from "lucide-react";
import { useEffect, useState } from "react";
import { cn } from "../lib/cn";

type CopyState = "idle" | "copied" | "failed";

export interface CopyCommandProps {
  command: string;
  label: string;
  className?: string;
  /** Injected for tests. */
  clipboard?: Pick<Clipboard, "writeText"> | undefined;
}

export function CopyCommand({ command, label, className, clipboard }: CopyCommandProps) {
  const [state, setState] = useState<CopyState>("idle");

  useEffect(() => {
    if (state === "idle") return;
    const timer = setTimeout(() => setState("idle"), 2000);
    return () => clearTimeout(timer);
  }, [state]);

  const copy = () => {
    const target = clipboard ?? globalThis.navigator?.clipboard;
    if (!target) {
      setState("failed");
      return;
    }
    target.writeText(command).then(
      () => setState("copied"),
      () => setState("failed"),
    );
  };

  return (
    <div className={cn("group relative rounded-[0.7rem] border border-sumi-rule bg-sumi-raised", className)}>
      <pre className="overflow-x-auto px-5 py-4 pr-28 font-mono text-[0.875rem] leading-relaxed text-on-sumi">
        <code>{command}</code>
      </pre>
      <button
        type="button"
        onClick={copy}
        aria-label={`Copy ${label}`}
        className="absolute top-3 right-3 inline-flex items-center gap-1.5 rounded-full border border-sumi-rule px-3 py-1 text-[0.8125rem] text-on-sumi-soft transition-colors duration-200 hover:border-on-sumi-soft hover:text-on-sumi"
      >
        {state === "copied" ? <Check className="size-3.5 text-shu" aria-hidden="true" /> : <Copy className="size-3.5" aria-hidden="true" />}
        <span aria-live="polite">{state === "copied" ? "Copied" : state === "failed" ? "Select to copy" : "Copy"}</span>
      </button>
    </div>
  );
}
