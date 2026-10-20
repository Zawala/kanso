/**
 * The tour of what Kanso does, told as numbered chapters. Each chapter picks
 * one of a few layouts so the page has rhythm instead of a grid of identical
 * cards; every picture is a real screenshot of the app. Claims here must stay
 * true of the shipped app: change the copy in the same change as the feature.
 */
import { cn } from "../lib/cn";
import { Shot, type ShotSource } from "./Shot";

const WINDOW = { width: 2000, height: 1250 } as const;

export type ChapterLayout =
  /** Text in a narrow column, the screenshot beside it, sides alternating. */
  | { kind: "beside"; shot: ShotSource; caption: string; flip?: boolean }
  /** Text above in two columns, the screenshot full width below. */
  | { kind: "wide"; shot: ShotSource; caption: string }
  /** A main screenshot with a second one laid over its corner. */
  | { kind: "layered"; shot: ShotSource; over: ShotSource; caption: string }
  /** A small window floating beside the text. */
  | { kind: "floating"; shot: ShotSource; caption: string };

export interface Chapter {
  readonly id: string;
  readonly number: string;
  readonly name: string;
  readonly title: string;
  readonly body: string;
  readonly points: readonly string[];
  readonly layout: ChapterLayout;
}

export const CHAPTERS: readonly Chapter[] = [
  {
    id: "write",
    number: "01",
    name: "Write",
    title: "Markdown that keeps out of your way.",
    body: "Brackets and quotes pair as you type. Lists carry on when you press Return and indent with Tab. Tables line themselves up while you edit them. Paste a link and it becomes Markdown; paste code and it stays code.",
    points: ["Focus mode and typewriter scrolling", "A word goal for each note", "Vim keys, if you want them", "Themes, light and dark"],
    layout: {
      kind: "beside",
      shot: { name: "focus-word-goal", dark: false, alt: "Kanso in focus mode: a page of writing with a word count of 614 out of 1,000.", ...WINDOW },
      caption: "Focus mode hides everything but the page. The ring counts toward today's goal.",
    },
  },
  {
    id: "link",
    number: "02",
    name: "Link",
    title: "Notes that know about each other.",
    body: "Type [[ and choose a note. Every note shows what links to it, with the sentence around each link, and the mentions that could become links. The graph draws the whole web, and related notes are found by meaning, worked out on your computer.",
    points: ["[[Wiki-links]] with autocomplete", "Backlinks and unlinked mentions", "An outline of every heading", "A graph of your notes"],
    layout: {
      kind: "layered",
      shot: { name: "inspector-backlinks", alt: "A note open in Kanso with its backlinks listed in the inspector on the right.", ...WINDOW },
      over: { name: "graph", alt: "Kanso's graph view: notes as dots joined by their links.", ...WINDOW },
      caption: "Backlinks beside the note, and the graph of everything it touches.",
    },
  },
  {
    id: "capture",
    number: "03",
    name: "Capture",
    title: "From a thought to a note in two keys.",
    body: "Press ⌃⌥Space anywhere and a quick note opens over whatever you're doing. ⌘↩ files it in your Inbox and you're back where you were. ⌘D opens today's page in your journal, and templates fill in the date, the time and the title for you.",
    points: ["Quick capture from any app", "A daily note, one key away", "Templates with {{date}} and {{title}}", "Nested tags like #home/garden"],
    layout: {
      kind: "floating",
      shot: { name: "capture", alt: "Kanso's quick note window with a short note about a lease, filed to the Inbox.", width: 2000, height: 1071 },
      caption: "The quick note window, over any app.",
    },
  },
  {
    id: "plan",
    number: "04",
    name: "Plan",
    title: "Tasks live where you wrote them.",
    body: "Give any checkbox a date, like - [ ] Send the draft @2026-10-25, and it appears in Tasks under overdue, today or coming up. Reminders arrive as notifications, and ticking a task off in the list ticks it in the note.",
    points: ["Due dates in plain text", "A Tasks view across every note", "Reminders as notifications"],
    layout: {
      kind: "wide",
      shot: { name: "tasks", alt: "Kanso's Tasks view: overdue, today and upcoming tasks gathered from every note.", ...WINDOW },
      caption: "Tasks from every note in one list, beside the note they came from.",
    },
  },
  {
    id: "explain",
    number: "05",
    name: "Explain",
    title: "Diagrams, maths and drawings, in plain text.",
    body: "Mermaid diagrams, maths and Excalidraw drawings render right in the preview, in your note's theme. Images resize with a drag, or with ![alt|300] in the Markdown. It's all still text you can read anywhere.",
    points: ["Mermaid flowcharts and sequences", "Maths, inline and in blocks", "Excalidraw drawings", "Export to a web page or PDF"],
    layout: {
      kind: "beside",
      flip: true,
      shot: { name: "rich-split", dark: false, alt: "A note in split view: Markdown on the left, rendered maths and Mermaid diagrams on the right.", ...WINDOW },
      caption: "Split view: the Markdown and what it draws, side by side.",
    },
  },
  {
    id: "find",
    number: "06",
    name: "Find",
    title: "Find anything. Change everything at once.",
    body: "⌘P jumps to any note by its title or by a heading inside it. Search narrows by notebook, tag and status. Find and replace works across every note and undoes in one step. On a Mac, even the words in your screenshots and photos are searchable.",
    points: ["Jump by title or heading", "Search with filters", "Replace across every note", "Text in images, on macOS"],
    layout: {
      kind: "beside",
      shot: { name: "palette-headings", dark: false, alt: "Kanso's quick open listing notes and the headings inside them.", ...WINDOW },
      caption: "Quick open matches headings inside notes as well as titles.",
    },
  },
  {
    id: "remember",
    number: "07",
    name: "Remember",
    title: "Every version, kept.",
    body: "Each note keeps its earlier versions and shows exactly what changed since each one. Restore any of them with a click. The whole library is backed up every day, on your computer.",
    points: ["History for every note", "What changed, line by line", "Daily backups"],
    layout: {
      kind: "beside",
      flip: true,
      shot: { name: "history", dark: false, alt: "A note's version history with the changes since an earlier version highlighted.", ...WINDOW },
      caption: "A note's history, with what changed since the version you picked.",
    },
  },
  {
    id: "share",
    number: "08",
    name: "Share",
    title: "Publish a note. Keep the key.",
    body: "Turn a note into a web page that anyone with the link can read. It's encrypted before it leaves your computer, and the key lives only in the link, so the server holding the page can't read it. Links are short and preview with the note's title.",
    points: ["End-to-end encrypted pages", "Short links that preview nicely", "Update or unpublish any time"],
    layout: {
      kind: "beside",
      shot: { name: "publish-published", dark: false, alt: "Kanso's publish sheet showing a note's link and when it was published.", ...WINDOW },
      caption: "The publish sheet: copy the link, update the page, or take it down.",
    },
  },
];

function ChapterText({ chapter, className }: { chapter: Chapter; className?: string }) {
  return (
    <div className={className}>
      <p className="type-eyebrow flex items-baseline gap-3 text-ink-faint">
        <span className="font-display text-[0.95rem] tracking-normal text-shu">{chapter.number}</span>
        {chapter.name}
      </p>
      <h3 className="type-chapter mt-4 text-ink">{chapter.title}</h3>
      <p className="mt-5 text-ink-soft">{chapter.body}</p>
      <ul className="mt-7 grid gap-x-8 gap-y-2.5 border-t border-rule pt-6 text-[0.9375rem] text-ink-soft sm:grid-cols-2">
        {chapter.points.map((point) => (
          <li key={point} className="flex gap-3">
            <span aria-hidden="true" className="mt-[0.6em] size-1.5 shrink-0 rounded-full bg-shu" />
            {point}
          </li>
        ))}
      </ul>
    </div>
  );
}

export function ChapterView({ chapter }: { chapter: Chapter }) {
  const { layout } = chapter;
  switch (layout.kind) {
    case "beside":
      return (
        <article id={chapter.id} className="reveal grid items-center gap-10 lg:grid-cols-12 lg:gap-14">
          <ChapterText chapter={chapter} className={cn("lg:col-span-4", layout.flip && "lg:order-2 lg:col-start-9")} />
          <Shot
            shot={layout.shot}
            figure={`Fig. ${chapter.number}`}
            caption={layout.caption}
            className={cn("lg:col-span-8", layout.flip ? "lg:order-1 lg:col-start-1 lg:-ml-[6vw]" : "lg:-mr-[6vw]")}
          />
        </article>
      );
    case "wide":
      return (
        <article id={chapter.id} className="reveal">
          <ChapterText chapter={chapter} className="max-w-3xl" />
          <Shot shot={layout.shot} figure={`Fig. ${chapter.number}`} caption={layout.caption} className="mt-12 lg:mx-[-3vw]" />
        </article>
      );
    case "layered":
      return (
        <article id={chapter.id} className="reveal grid items-center gap-10 lg:grid-cols-12 lg:gap-14">
          <ChapterText chapter={chapter} className="lg:col-span-4 lg:col-start-9 lg:order-2" />
          <div className="relative pb-[18%] lg:order-1 lg:col-span-8 lg:col-start-1">
            <Shot shot={layout.shot} className="w-[88%]" />
            <Shot shot={layout.over} className="absolute right-0 bottom-0 w-[52%]" />
            <p className="mt-6 flex gap-3 text-[0.9375rem] leading-snug text-ink-faint lg:absolute lg:-bottom-12 lg:left-0">
              <span className="font-display font-bold text-ink-soft">Fig. {chapter.number}</span>
              {layout.caption}
            </p>
          </div>
        </article>
      );
    case "floating":
      return (
        <article id={chapter.id} className="reveal grid items-center gap-10 lg:grid-cols-12 lg:gap-14">
          <ChapterText chapter={chapter} className="lg:col-span-5" />
          <div className="relative lg:col-span-6 lg:col-start-7">
            <div aria-hidden="true" className="absolute inset-y-[-8%] inset-x-[-1rem] -z-10 rounded-[2rem] bg-paper-deep lg:inset-x-[-8%]" />
            <Shot shot={layout.shot} figure={`Fig. ${chapter.number}`} caption={layout.caption} className="mx-auto max-w-[34rem] rotate-[-1.5deg]" />
          </div>
        </article>
      );
  }
}
