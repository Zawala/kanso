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

**Taking over from Inkdrop.** Quit Inkdrop so Kanso can use port 19840 (until then the status bar
says the port is in use, and Kanso keeps trying). In Settings › Local API, **Copy from Inkdrop…**
takes Inkdrop's port, username and password; macOS asks you to allow access to the Keychain item
"Inkdrop Safe Storage", which Inkdrop uses to encrypt them. MCP clients configured for Inkdrop then
work without changes.

**Sync.** Deploy the Worker (see [cloud/README.md](cloud/README.md)), then in Settings › Sync set the
Worker URL and access token and turn on **Sync automatically**.

### Shortcuts

| | |
|---|---|
| ⌘N / ⇧⌘N | New note / notebook |
| ⇧⌘F or ⌘L | Search notes (`book:`, `tag:`, `status:`, `title:`, `body:`, `"phrases"`, `-exclude`) |
| ↑ ↓ or J K in the list, Return | Move through notes, edit the selected one |
| ⌥⌘↓ / ⌥⌘↑ | Next / previous note from anywhere |
| ⌘1 / ⌘2 / ⌘3 | Editor only / editor and preview / preview only |
| ⌘\ | Toggle the sidebar |
| ⇧⌘P | Pin or unpin |
| ⌘S | Save now (notes also save on their own) |
| ⌘, | Settings |

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
