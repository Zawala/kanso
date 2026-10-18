# Kanso plan

Kanso is a local-first markdown notes app that replaces Inkdrop. It has a GPUI desktop UI,
serves Inkdrop's local HTTP API on `127.0.0.1:19840` so existing MCP setups keep working,
imports an Inkdrop database losslessly, and syncs through Cloudflare.

## Decisions

| Topic | Decision |
|---|---|
| UI | GPUI through `gpui-kit = "=0.7.3"` (gpui-component, `EditorState` for markdown source, base `TextView` for preview). Pinned exactly; upgrade on purpose. |
| API | Inkdrop local-server compatible REST on `127.0.0.1:19840` with Basic auth. The official `@inkdropapp/mcp-server` runs against it unchanged. |
| MCP | Native `/mcp` endpoint on the same port with the same 17 tool names and params, added after the REST API passes contract tests. |
| Credentials | Port, username and password are copied from Inkdrop's `config.json` at import time and kept in Kanso's own config. Never logged. |
| Storage | rusqlite (bundled), WAL. Each document stored as verbatim Inkdrop-shaped JSON with derived index columns. |
| Search | FTS5 `trigram` over title and body, the same design Inkdrop 5.9+ uses. |
| Sync | Cloudflare Worker + D1 (documents, change sequence) + R2 (attachment bytes). See "Sync". |
| Server placement | In the app process; the app stays alive with no window open. `kanso serve` runs headless. An `flock` on the data directory decides who owns the database and the port. |

## Workspace

```
crates/
  kanso-core     domain types, IDs, revs, search-query parser
  kanso-store    SQLite store, FTS, revisions, backups, sync state
  kanso-import   Inkdrop PouchDB importer
  kanso-server   axum REST API (+ /mcp later)
  kanso-sync     Cloudflare sync client
  kanso-app      GPUI app, binary `kanso`
cloud/           Cloudflare Worker (D1 + R2)
```

## Data model

Documents keep Inkdrop's IDs and shapes so `inkdrop://note/<id>` links and `inkdrop://file:<id>`
image embeds inside note bodies keep working, and API clients see what they expect.

- `note:<8 chars>`: `title, body, bookId, tags[], status (none|active|onHold|completed|dropped),
  share (private|public), pinned, doctype=markdown, createdAt, updatedAt, timestamp,
  numOfTasks, numOfCheckedTasks`, plus any unknown fields (e.g. `sourceTemplateId`), kept as-is.
- `book:<id>`: `name, parentBookId, order, createdAt, updatedAt`. `count` is derived.
- `tag:<id>`: `name, color, createdAt, updatedAt`. `count` is derived.
- `file:<id>`: `name, contentType, contentLength, publicIn[], md5digest, createdAt` and the bytes.
- `trash` is a pseudo-book ID. `book:inbox` style non-random IDs exist.
- Derived fields (`count`, `numOfTasks`, `numOfCheckedTasks`) are stripped on write and recomputed.

Store tables:

- `docs(id PK, type, rev, seq UNIQUE, deleted, json, synced_rev)` with generated, indexed columns
  for `book_id, status, updated_at, created_at, title, pinned`.
- `note_tags(note_id, tag_id)`, filled in the same transaction as the note.
- `blobs(file_id PK, md5, bytes)`.
- `revisions(doc_id, rev, json, at)`: one snapshot per write, autosave capped at one per 5 minutes
  per note. This is the undo net when an agent or sync overwrites a note.
- `notes_fts`: FTS5 `tokenize='trigram'` over title and body, excluding trash.
- Daily `VACUUM INTO` backups with rotation.

### Revs and conflicts

- `_rev` is `"{n}-{md5 of canonical json}"`. Only the current rev is kept (plus snapshots).
- `POST /notes`: no `_id` creates; unknown `_id` creates with that ID; existing `_id` with a
  missing or stale `_rev` returns 409.
- `POST /book:<id>` and `/tag:<id>`: partial merge; 409 only when a given `_rev` is stale.
- `GET /<id>?rev=`: the current rev returns the doc, any other rev returns 404.
- Every write runs in `BEGIN IMMEDIATE`.
- GUI: an external change to a note with a clean buffer reloads and keeps the cursor; with a
  dirty buffer, the buffer wins, the other version is snapshotted, and a banner says so.

## HTTP API contract

What `@inkdropapp/mcp-server` 1.5.0 calls (must work exactly):

| Call | Notes |
|---|---|
| `GET /{note,book,tag,file}:<id>` | `?rev=`, `?attachments=true` (files return `_attachments.index.data` base64) |
| `GET /notes?keyword&limit&sort&descending` | `sort`: updatedAt, createdAt, title. Default limit 50. Full `body` in results. Trash excluded. |
| `GET /books`, `GET /tags` | Full arrays |
| `POST /notes` | Create or full-document update (`_id` + `_rev`) |
| `POST /books`, `POST /tags`, `POST /files` | Create. Files carry `_attachments.index.{content_type,data}` |
| `POST /{book,tag}:<id>` | Partial merge, `_rev` optional |

All POSTs answer `{ok, id, rev}`. Errors are non-2xx. Request bodies up to 16 MB.

Documented but unused in 9 months of logs (later): `skip`, `sort=tagCount|rank`, `DELETE`,
`GET /`, `GET /_changes`, `/note/<id>` aliases.

### Search syntax

Terms and quoted phrases match title or body as case-insensitive substrings (trigram phrase
queries; terms under 3 characters fall back to an `instr` scan). Qualifiers: `book:Name`,
`book:"Two Words"`, `bookId:<id without prefix>`, `tag:Name`, `tagId:tag:<id>`, `status:onHold`,
`title:"..."`, `body:word`, and `-` to negate any of them. Backlinks search for
`inkdrop://note/<id>`. Priorities from 1,629 real queries: quoted phrases (49%), `title:` (39%),
`bookId:` (33%), `book:` (5%), everything else under 1%.

## Importer

- Requires Inkdrop to be quit. Copies `<hash>.sqlite` and its `-wal` to a temp dir and reads the copy.
  Never writes to Inkdrop's folder. Ignores the stale LevelDB folders, `mrview-*` and `fts.db`.
- Reads the winning, non-deleted revision of every `note:`, `book:`, `tag:` and `file:` doc
  (`document-store.winningseq` → `by-sequence`).
- Attachments live in `attach-store.body` as PouchDB binary strings (one char per byte).
  Decoding is verified against `_attachments.index.digest` (`md5-<base64>`) for every file.
- Skips `_design/*`, `config` and `_local` docs. Keeps `activity:` docs raw in an archive table.
- Idempotent (upsert by ID) and ends with a verification report against the source counts.
- Copies the server port and credentials from Inkdrop's `config.json`.

## Sync (Cloudflare)

- **Worker** (TypeScript, deployed with wrangler) exposing `/sync/push`, `/sync/changes`,
  `/files/<id>`. Auth is a single bearer secret (wrangler secret on the server, Keychain on the client).
- **D1**: `docs(id PK, rev, seq, deleted, json, updated_at)`; `seq` increases on every accepted write.
- **R2**: attachment bytes keyed by file ID.
- **Push**: every local doc whose `rev != synced_rev` is sent with its `base_rev` (`synced_rev`).
  The server accepts when `base_rev` matches its current rev (or the doc is new) and returns the
  new seq; otherwise it returns its version. Blobs are uploaded before their file docs.
- **Pull**: `GET /sync/changes?since=<seq>` pages through changes; applied docs get
  `synced_rev = rev` so they are not pushed back.
- **Conflicts**: last writer by `updatedAt` wins; the losing version goes into `revisions` and the
  GUI flags the note.
- The deploy token comes from the `IMNGONII_CLOUDFLARE_API_TOKEN` environment variable
  (exported in `~/.zshrc`), passed to wrangler as `CLOUDFLARE_API_TOKEN`. It is never written to the repo.

## Milestones

| | Scope | Done when |
|---|---|---|
| M0 | GPUI editor spike (timebox 2 days) | The go/no-go below passes |
| M1 | core, store, importer, revisions, backups | The real DB imports with matching counts and all attachments verified |
| M2 | REST API, search parser | The real `@inkdropapp/mcp-server` passes all 17 tools against Kanso; parser property tests pass |
| M3 | GPUI app MVP | Sidebar (notebooks, tags, statuses, trash), note list, editor + preview, search, pickers, links, images, autosave, external-change banner |
| M4 | Cloudflare sync | Two machines converge; conflicts keep both versions |
| M5 | Native `/mcp` endpoint | Same 17 tools, outputs match the Node server |
| M6 | Polish | Backlinks panel, Markdown export, settings, launch at login, revisions UI |

### Status (2026-10-17)

All milestones are built and tested.

- M0: the automated criteria pass (load, edit latency, preview). IME, press-and-hold accents, emoji
  and undo grouping still need a hands-on check.
- M1: the real library imported (823 notes, 43 notebooks, 28 tags, 15 attachments, all verified);
  a second import changes nothing.
- M2 and M5: the real `@inkdropapp/mcp-server` runs all 17 tools against Kanso, and `/mcp` gives the
  same results (`crates/kanso-server/tests/contract.rs`).
- M4: deployed to Cloudflare and checked end to end between two data folders.
- Inkdrop's stored task counts were stale on 21 notes; Kanso recounts from the body like GFM does.
- Inkdrop's API credentials are encrypted with Electron safeStorage, so copying them needs the
  "inkdrop Safe Storage" Keychain item, which macOS asks the user to allow.
- A code review of the app found ways to lose typed text (switching notes or quitting while the
  "changed elsewhere" question was open, ⌘Q, a note deleted elsewhere). All are fixed, and a
  model-based test (`prop_typed_text_is_never_lost`) runs random sequences of typing, outside
  edits and deletions, switching, answers and quitting against the real views.

### UI redesign (2026-10-18)

The app now follows the design spec and mockups from May 2026 (macOS desktop spec; mockups 01 sync
settings, 05 command palette, 06 split editor, 07 empty notebook):

- Warm graphite (dark) and paper (light) themes from the spec's palette, SF Pro body text at 16 px,
  SF Mono in split mode, and restrained Markdown highlighting.
- A transparent title bar with shared 52 pt top and 36 pt bottom bands across the panes. The global
  status bar is gone: sync and the local API show in the sidebar footer, and cursor and word counts
  in the editor footer.
- Sidebar: All Notes, Pinned, Recent (last 7 days), notebooks, `#tag` chips, a Status group and
  Trash. ⌘\ folds it to an icon rail.
- Note list grouped by day, with status glyphs, two-line excerpts (`kanso_core::excerpt`), match
  highlighting, one filter/sort menu and the empty-notebook block.
- Editor header with breadcrumb, Edit/Preview/Split, status and tag chips, a formatting toolbar,
  and a backlinks menu. Split has a draggable divider and loose scroll sync.
- Command palette (⌘K) and Open Note (⌘P) drawn inside the workspace so commands reach its handlers.
- Focus mode (⇧⌘D) and a first-run welcome.
- Settings moved into the main window (General, Editor, Appearance, Sync, Local API); the separate
  settings window is gone.

Left out on purpose, because Kanso has no such features yet: sketch blocks, skills, MCP
permissions, accounts and devices, templates and a Tasks view. The UI is checked headlessly by an
ignored snapshot test that renders PNGs (`KANSO_SNAPSHOT_DIR=<dir> cargo test -p kanso-app
snapshots -- --ignored`, optionally with `KANSO_SNAPSHOT_DB` pointing at a copy of a library).

### M0 go/no-go

- The 253 KB note opens in under 200 ms.
- p95 edit-to-frame under 16 ms at the start, middle and end of that note, with soft wrap on.
- Smooth scrolling.
- IME composition, press-and-hold accents and emoji work.
- Undo groups sensibly.
- Reloading a note can keep the cursor and scroll position.
- The preview renders that note, including `inkdrop://file:` images, in under 300 ms.

If IME or latency fails, fork only the input module.

## Risks

- gpui-kit is young and pins a weekly GPUI snapshot (`gpui-pre =0.3.9`). Exact pins and a
  committed `Cargo.lock`.
- Inkdrop still running holds port 19840. Kanso shows "API off: port in use" with a retry and never
  silently moves to another port.
- No live Inkdrop to compare search results against (it no longer loads the database since the
  subscription lapsed). Parser tests are built from the request history in Inkdrop's `server.log`.
- Testing: property tests (proptest) for the query parser, rev/conflict rules and attachment
  decoding; integration tests on temp databases; contract tests with the real MCP server.
