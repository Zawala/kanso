# Kanso

A local-first Markdown notes app for macOS, built with [GPUI](https://www.gpui.rs) through
[gpui-kit](https://github.com/longbridge/gpui-kit), as a replacement for Inkdrop.

- **Imports Inkdrop.** Notes, notebooks, tags, statuses and attachments come over from Inkdrop's
  database, which is only read. Running the import again picks up newer Inkdrop edits and never
  reverts notes you changed in Kanso.
- **Serves Inkdrop's local API.** `http://127.0.0.1:19840` answers like Inkdrop's local server, so
  `@inkdropapp/mcp-server` and other Inkdrop API clients keep working unchanged. `/mcp` on the same
  port is a native MCP endpoint with the same 17 tools.
- **Syncs end-to-end encrypted.** An account on a Cloudflare Worker (D1 for documents, R2 for
  attachments) keeps your Macs in step. Each Mac encrypts notes, titles, IDs and attachments before
  they leave it, with a key the server never sees. Conflicts keep both versions: the losing one
  stays in the note's history.
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

**Sync.** Settings › Account › **Create account** picks a username and a password of at least 12
characters; this Mac's notes are encrypted and uploaded. Kanso then shows a **recovery key**: save
it somewhere safe (a password manager, or printed). Kanso can't reset a forgotten password. The
recovery key sets a new one (**Forgot your password?**), and a new key replaces it afterwards. On
your other Macs, **Sign in** with the same username and password. A Mac that already has notes asks
whether to add them to the account or replace them with the account's notes (after saving a backup
of its library). Signing out keeps every note on the Mac. The Account page also lists your devices,
signs one out, changes the password and deletes the account.

The default server is the owner's Worker, `kanso.antonlabs.cc`, which takes new
accounts only when its owner opens sign-ups. To run your own, deploy the Worker in
[cloud/](cloud/README.md) to your Cloudflare account and choose **Use a different server…** before
creating the account. An account belongs to its server's address, so pick the final URL first.

**Other people's Macs.** The builds are ad-hoc signed and not notarized. On first launch,
Control-click Kanso and choose Open (or System Settings › Privacy & Security › Open Anyway). Each new
build has a different signature, so macOS may ask once per build to let Kanso use its Keychain
items; choose Always Allow.

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
cloud/scripts/e2e-local.sh   # the Worker under wrangler dev and the Rust client against it
```

| Crate | |
|---|---|
| `kanso-core` | Document types, revisions, the search query language, task and link parsing |
| `kanso-store` | SQLite store: writes, search, history, change feed, backups, sync bookkeeping |
| `kanso-import` | Inkdrop (PouchDB on SQLite) importer and Electron safeStorage decryption |
| `kanso-server` | Inkdrop-compatible HTTP API and the native MCP endpoint |
| `kanso-sync` | Accounts and end-to-end encrypted sync, and the `kanso-sync` command (`verify`, `estimate`, `unbind`) |
| `kanso-app` | The GPUI app and the `kanso` command |
| `cloud/` | The sync Worker (TypeScript, D1, R2) and its admin script |

Design notes and decisions are in [docs/PLAN.md](docs/PLAN.md); accounts and encrypted sync in
[docs/ACCOUNTS.md](docs/ACCOUNTS.md).
