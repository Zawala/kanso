/**
 * Privacy, in plain words: what this website collects (nothing), and what
 * Kanso's sync server can and can't see. It summarises docs/ACCOUNTS.md
 * ("What the server can and can't see") and must change when that does.
 */
import { createFileRoute } from "@tanstack/react-router";
import { REPOSITORY_URL } from "../lib/release";

export const Route = createFileRoute("/privacy")({
  head: () => ({
    meta: [
      { title: "Privacy — Kanso" },
      { name: "description", content: "What Kanso's website and sync server can and can't see." },
    ],
    links: [{ rel: "canonical", href: "https://kansoapp.app/privacy" }],
  }),
  component: Privacy,
});

const CAN_SEE = [
  ["Your account", "Your username, when the account was made, and whether it's active."],
  ["Sizes and times", "How many notes, notebooks, tags and files you have, roughly how big each is, and when each was written."],
  ["Your devices", "How many devices are signed in and when each was last seen, to the hour. Not their names: those are encrypted."],
  ["The network", "Your IP address, as any server does while handling a request. It is kept only as a keyed hash in rate-limit records, for at most two days."],
] as const;

const CANNOT_SEE = [
  "Note titles or text",
  "Notebook and tag names",
  "Which notes are in which notebook or carry which tag",
  "Links between notes",
  "File names or contents",
  "Your password, your recovery key or the key your notes are encrypted with",
] as const;

function Privacy() {
  return (
    <article className="page pt-[clamp(3.5rem,9vw,7rem)] pb-[clamp(5rem,12vw,9rem)]">
      <p className="type-eyebrow text-shu">Privacy</p>
      <h1 className="type-section mt-5 max-w-3xl">We'd rather not know.</h1>
      <p className="type-lede mt-6 max-w-2xl text-ink-soft">
        Kanso keeps your notes on your computer. If you sync, they are encrypted on your device first, so the server holds
        only what it needs to pass them between your devices.
      </p>

      <div className="mt-[clamp(3.5rem,8vw,6rem)] grid gap-16 lg:grid-cols-12">
        <section aria-labelledby="site" className="lg:col-span-4">
          <h2 id="site" className="text-[clamp(1.5rem,1.25rem+0.9vw,2rem)]">
            This website
          </h2>
          <p className="mt-4 text-ink-soft">
            No analytics, no cookies, no trackers, no forms. The server reads your browser's User-Agent to suggest the right
            download and then forgets it.
          </p>
        </section>

        <section aria-labelledby="server" className="lg:col-span-7 lg:col-start-6">
          <h2 id="server" className="text-[clamp(1.5rem,1.25rem+0.9vw,2rem)]">
            The sync server
          </h2>
          <h3 className="type-eyebrow mt-8 text-ink-faint">It can't see</h3>
          <ul className="mt-4 grid gap-x-8 gap-y-2.5 sm:grid-cols-2">
            {CANNOT_SEE.map((item) => (
              <li key={item} className="flex gap-3 text-ink">
                <span aria-hidden="true" className="mt-[0.6em] size-1.5 shrink-0 rounded-full bg-shu" />
                {item}
              </li>
            ))}
          </ul>
          <h3 className="type-eyebrow mt-12 text-ink-faint">It can see</h3>
          <dl className="mt-4 grid gap-5">
            {CAN_SEE.map(([term, detail]) => (
              <div key={term} className="grid gap-1 border-t border-rule pt-4 sm:grid-cols-[11rem_1fr] sm:gap-6">
                <dt className="font-medium text-ink">{term}</dt>
                <dd className="text-ink-soft">{detail}</dd>
              </div>
            ))}
          </dl>
          <h3 className="type-eyebrow mt-12 text-ink-faint">Published notes</h3>
          <p className="mt-4 text-ink-soft">
            A published note is encrypted too, and its key lives only in the link you share. To give the link a short address
            and a preview, Kanso sends the note's title, and never its key or text, to the link service.
          </p>
          <p className="mt-10 text-[0.9375rem] text-ink-faint">
            Every detail, including how the encryption works, is in{" "}
            <a href={`${REPOSITORY_URL}/blob/master/docs/ACCOUNTS.md`} className="ink-link">
              the accounts design document
            </a>
            . You can also run the sync server on your own Cloudflare account.
          </p>
        </section>
      </div>
    </article>
  );
}
