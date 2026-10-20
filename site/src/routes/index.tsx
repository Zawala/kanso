/**
 * The home page: the promise, the proof (encryption you can watch happen), the
 * tour, agents, open source, and one last way to download. Data comes from
 * `getSiteData`; when GitHub can't be reached the buttons point to its
 * releases page instead.
 */
import { Link, createFileRoute } from "@tanstack/react-router";
import { ArrowRight, ArrowUpRight } from "lucide-react";
import { CHAPTERS, ChapterView } from "../components/Chapters";
import { CipherDemo } from "../components/CipherDemo";
import { CopyCommand } from "../components/CopyCommand";
import { DownloadCta } from "../components/DownloadCta";
import { Enso, Seal } from "../components/Enso";
import { Shot } from "../components/Shot";
import { REPOSITORY_URL } from "../lib/release";
import { getSiteData } from "../server/fns";

export const Route = createFileRoute("/")({
  loader: () => getSiteData(),
  component: Home,
});

const MCP_COMMAND = `claude mcp add --transport http kanso http://127.0.0.1:19840/mcp \\
  --header "Authorization: Basic $(printf '%s' 'USERNAME:PASSWORD' | base64)"`;

const PROMISES = [
  {
    numeral: "一",
    title: "Yours, on your computer.",
    body: "Your notes are Markdown, kept on your own machine, and everything works offline. An account is only for keeping your devices in step.",
  },
  {
    numeral: "二",
    title: "Encrypted before it leaves.",
    body: "Notes, titles, tags and files are sealed on your device with a key only your devices hold. The server stores ciphertext and can't read a word of it.",
  },
  {
    numeral: "三",
    title: "Native, and quick about it.",
    body: "Written in Rust and drawn by your graphics card, with no web browser hiding inside. It feels like the rest of your computer.",
  },
] as const;

function Home() {
  const { release, platform } = Route.useLoaderData();
  return (
    <>
      <Hero release={release} platform={platform} />
      <Promises />
      <Proof />
      <Features />
      <Agents />
      <OpenSource />
      <Begin release={release} platform={platform} />
    </>
  );
}

function Hero({ release, platform }: Parameters<typeof DownloadCta>[0]) {
  return (
    <section className="relative overflow-x-clip">
      <div className="page relative pt-[clamp(3rem,9vw,8rem)]">
        <div aria-hidden="true" className="pointer-events-none absolute top-[2vw] right-[-18vw] w-[min(64vw,44rem)] text-ink/[0.92] max-lg:opacity-[0.13] lg:top-[3.5rem] lg:right-[-12vw] xl:right-[-9vw]">
          <Enso />
          <Seal className="absolute top-[58%] right-[22%] hidden lg:inline-grid" />
        </div>
        <div className="relative max-w-[52rem] lg:max-w-[min(52rem,58vw)]">
          <p className="type-eyebrow rise flex items-center gap-3 text-ink-soft" style={{ ["--rise-step" as string]: 0 }}>
            <span className="size-2 rounded-full bg-shu" aria-hidden="true" />
            Notes, kept simple
          </p>
          <h1 className="type-hero rise mt-7" style={{ ["--rise-step" as string]: 1 }}>
            Write it down.
            <br />
            Keep it <span className="text-shu">yours.</span>
          </h1>
          <p className="type-lede rise mt-8 max-w-[38rem] text-ink-soft" style={{ ["--rise-step" as string]: 2 }}>
            Kanso is a calm, fast Markdown notebook for Mac, Windows and Linux. Your notes live on your computer and sync
            between your devices end-to-end encrypted.
          </p>
          <DownloadCta release={release} platform={platform} className="rise mt-10" />
        </div>
      </div>
      <div className="page rise mt-[clamp(4rem,8vw,7rem)]" style={{ ["--rise-step" as string]: 4 }}>
        <Shot
          eager
          shot={{ name: "split", alt: "Kanso's main window: notebooks on the left, a list of notes, and a note in split view with its Markdown beside the rendered page.", width: 2000, height: 1250 }}
          figure="Fig. 00"
          caption="Notebooks, notes and the page, with the Markdown beside what it renders."
          className="lg:mr-[-10vw]"
        />
      </div>
    </section>
  );
}

function Promises() {
  return (
    <section aria-labelledby="promises" className="page py-[clamp(6rem,13vw,11rem)]">
      <h2 id="promises" className="sr-only">
        What Kanso promises
      </h2>
      <ol className="grid gap-14 lg:grid-cols-3 lg:gap-16">
        {PROMISES.map((promise, i) => (
          <li key={promise.numeral} className={["reveal", i === 1 ? "lg:mt-20" : i === 2 ? "lg:mt-40" : ""].join(" ")}>
            <span aria-hidden="true" className="kanji-numeral text-[clamp(3.5rem,6vw,5rem)]">
              {promise.numeral}
            </span>
            <h3 className="mt-6 text-[clamp(1.6rem,1.3rem+1vw,2.1rem)] leading-tight">{promise.title}</h3>
            <p className="mt-4 max-w-sm text-ink-soft">{promise.body}</p>
          </li>
        ))}
      </ol>
    </section>
  );
}

function Proof() {
  return (
    <section aria-labelledby="proof" className="bg-sumi text-on-sumi">
      <div className="page py-[clamp(5rem,11vw,9rem)]">
        <div className="grid items-end gap-8 lg:grid-cols-12">
          <div className="lg:col-span-7">
            <p className="type-eyebrow text-shu">End-to-end encryption, shown</p>
            <h2 id="proof" className="type-section mt-5">
              This is all our server ever&nbsp;sees.
            </h2>
          </div>
          <p className="text-on-sumi-soft lg:col-span-5">
            Type in the note below. It's sealed right here in your browser as you type, the way Kanso seals every note before
            it syncs. Nothing on this page is sent anywhere.
          </p>
        </div>
        <div className="mt-12">
          <CipherDemo />
        </div>
        <p className="mt-6 max-w-3xl text-[0.9375rem] text-on-sumi-soft">
          This demonstration uses your browser's AES-256-GCM. The app seals what it syncs with XChaCha20-Poly1305, under a key
          only your devices hold.{" "}
          <Link to="/privacy" className="ink-link text-on-sumi">
            What the server can and can't see
          </Link>
        </p>
      </div>
    </section>
  );
}

function Features() {
  return (
    <section id="features" aria-labelledby="features-title" className="page py-[clamp(6rem,13vw,11rem)]">
      <div className="max-w-4xl">
        <p className="type-eyebrow text-ink-faint">Inside Kanso</p>
        <h2 id="features-title" className="type-section mt-5">
          Everything a note needs.
          <br />
          <span className="text-ink-faint">Nothing it doesn't.</span>
        </h2>
      </div>
      <div className="mt-[clamp(4rem,9vw,8rem)] grid gap-[clamp(7rem,13vw,12rem)]">
        {CHAPTERS.map((chapter) => (
          <ChapterView key={chapter.id} chapter={chapter} />
        ))}
      </div>
    </section>
  );
}

function Agents() {
  return (
    <section aria-labelledby="agents" className="bg-paper-deep">
      <div className="page grid gap-12 py-[clamp(5rem,11vw,9rem)] lg:grid-cols-12 lg:gap-16">
        <div className="lg:col-span-5">
          <p className="type-eyebrow text-ink-faint">For your agents</p>
          <h2 id="agents" className="type-section mt-5">
            Your notes, ready for your&nbsp;agents.
          </h2>
        </div>
        <div className="min-w-0 lg:col-span-7 lg:pt-3">
          <p className="type-lede text-ink-soft">
            Kanso runs a local API and an MCP server on your own computer. Point Claude, Codex, Cursor or any MCP client at
            it, and they can search, read and write your notes, on 127.0.0.1, with your password, never through a cloud.
          </p>
          <div className="mt-9 rounded-[0.9rem] bg-sumi p-2 shadow-window">
            <CopyCommand command={MCP_COMMAND} label="the command that adds Kanso to Claude Code" />
          </div>
          <p className="mt-5 text-[0.9375rem] text-ink-faint">
            Your API username and password are in Settings › Local API. Twenty-four tools cover notes, notebooks, tags and
            files, plus outlines, tasks, templates, daily notes and related notes.
          </p>
        </div>
      </div>
    </section>
  );
}

function OpenSource() {
  return (
    <section aria-labelledby="open" className="page grid gap-10 py-[clamp(6rem,13vw,11rem)] lg:grid-cols-12">
      <div className="lg:col-span-6 lg:col-start-2">
        <p className="type-eyebrow text-ink-faint">Open source</p>
        <h2 id="open" className="type-section mt-5">
          Free, and staying that&nbsp;way.
        </h2>
      </div>
      <div className="lg:col-span-4 lg:col-start-8 lg:pt-12">
        <p className="text-ink-soft">
          Kanso is free software under the AGPL-3.0. Read the code, open an issue, send a pull request: it gets better every
          week. Would you rather hold your own data? The sync server is one Cloudflare Worker you can run on your own
          account.
        </p>
        <div className="mt-8 flex flex-col items-start gap-3 font-medium">
          <a href={REPOSITORY_URL} className="ink-link inline-flex items-center gap-2">
            Read the code on GitHub <ArrowUpRight className="size-4" aria-hidden="true" />
          </a>
          <a href={`${REPOSITORY_URL}/tree/master/cloud`} className="ink-link inline-flex items-center gap-2">
            Run your own server <ArrowUpRight className="size-4" aria-hidden="true" />
          </a>
        </div>
      </div>
    </section>
  );
}

function Begin({ release, platform }: Parameters<typeof DownloadCta>[0]) {
  return (
    <section aria-labelledby="begin" className="relative overflow-hidden bg-sumi text-on-sumi">
      <div aria-hidden="true" className="pointer-events-none absolute -right-[12vw] -bottom-[18vw] w-[min(70vw,52rem)] text-on-sumi/[0.07]">
        <Enso animate={false} />
      </div>
      <div className="page relative py-[clamp(6rem,14vw,12rem)]">
        <h2 id="begin" className="type-hero max-w-4xl">
          Begin with a blank&nbsp;page.
        </h2>
        <p className="type-lede mt-8 max-w-xl text-on-sumi-soft">
          Kanso works offline, updates itself, and costs nothing.
        </p>
        <DownloadCta release={release} platform={platform} tone="sumi" className="mt-10" />
        <Link to="/download" className="ink-link mt-12 inline-flex items-center gap-2 text-[0.9375rem] text-on-sumi-soft">
          Every build, with checksums <ArrowRight className="size-4" aria-hidden="true" />
        </Link>
      </div>
    </section>
  );
}
