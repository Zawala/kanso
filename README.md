# Kanso

A local-first Markdown notes app for macOS, built with [GPUI](https://www.gpui.rs) through
[gpui-kit](https://github.com/longbridge/gpui-kit), as a replacement for Inkdrop.

- **Imports Inkdrop.** Notes, notebooks, tags, statuses and attachments come over from Inkdrop's
  database, which is only read. Running the import again picks up newer Inkdrop edits and never
  reverts notes you changed in Kanso.
- **Serves Inkdrop's local API.** `http://127.0.0.1:19840` answers like Inkdrop's local server, so
  `@inkdropapp/mcp-server` and other Inkdrop API clients keep working unchanged. `/mcp` on the same
  port is a native MCP endpoint with the same 17 tools.
- **Syncs through Cloudflare.** A Worker you deploy to your own account (D1 for documents, R2 for
  attachments). Conflicts keep both versions: the losing one stays in the note's history.
- **Keeps history.** Earlier versions of each note, including every version replaced by the API,
  sync or an import, and a daily database backup.

## Using it

```bash
scripts/bundle-macos.sh
```

Builds an ad-hoc signed `target/release/Kanso.app`; copy it to `/Applications`. On first run with an
empty library, Kanso offers to import from Inkdrop.

**Taking over from Inkdrop.** Quit Inkdrop so Kanso can use port 19840 (until then the plug icon at
the bottom of the sidebar turns amber and says the port is in use, and Kanso keeps trying). In
Settings › Local API, **Copy from Inkdrop…** takes Inkdrop's port, username and password; macOS asks
you to allow access to the Keychain item "inkdrop Safe Storage", which Inkdrop uses to encrypt them.
MCP clients configured for Inkdrop then work without changes.

**Sync.** Deploy the Worker (see [cloud/README.md](cloud/README.md)), then in Settings › Sync set the
Worker URL and access token and turn on **Sync automatically**.

### Shortcuts

| | |
|---|---|
| ⌘K | Command palette: every command, notebook, tag and status, plus note search |
| ⌘P | Open a note by title |
| ⌘N / ⇧⌘N | New note / notebook |
| ⇧⌘F | Search all notes (`book:`, `tag:`, `status:`, `title:`, `body:`, `"phrases"`, `-exclude`) |
| ⌘L | Search the list that is showing |
| ↑ ↓ or J K in the list, Return | Move through notes, edit the selected one |
| ⇧⌘L | Back to the note list |
| ⌥⌘↓ / ⌥⌘↑ | Next / previous note from anywhere |
| ⌘1 / ⌘2 / ⌘3 | Edit / split (drag the divider) / preview |
| ⇧⌘D | Focus mode: only the note; Escape leaves it |
| ⌘\ | Fold the sidebar to icons, or open it again |
| ⇧⌘P | Pin or unpin |
| ⌘S | Save now (notes also save on their own) |
| ⌘, | Settings, inside the window; Escape goes back to the notes |

### Command line

```bash
kanso                       # open the app
kanso serve                 # only the API (REST and /mcp), no window
kanso import-inkdrop        # import from Inkdrop; --credentials also copies its API settings
```

Every command takes `--data-dir <dir>`. The default is `~/Library/Application Support/Kanso`, or
`$KANSO_DATA_DIR`.

## Development

```bash
PROPTEST_RNG_SEED=20261017 cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
(cd cloud && npm ci && npm test)
```

| Crate | |
|---|---|
| `kanso-core` | Document types, revisions, the search query language, task and link parsing |
| `kanso-store` | SQLite store: writes, search, history, change feed, backups, sync bookkeeping |
| `kanso-import` | Inkdrop (PouchDB on SQLite) importer and Electron safeStorage decryption |
| `kanso-server` | Inkdrop-compatible HTTP API and the native MCP endpoint |
| `kanso-sync` | Cloudflare sync client |
| `kanso-app` | The GPUI app and the `kanso` command |
| `cloud/` | The sync Worker (TypeScript, D1, R2) |

Design notes and decisions are in [docs/PLAN.md](docs/PLAN.md).
