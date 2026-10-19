# Kanso

A local-first Markdown notes app for macOS, with Windows and Linux builds, built with [GPUI](https://www.gpui.rs) through
[gpui-kit](https://github.com/longbridge/gpui-kit).

- **Serves a local API for agents.** `http://127.0.0.1:19840` is a REST API over your notes, and
  `/mcp` on the same port is a native MCP endpoint with 17 tools. It speaks the same API as
  Inkdrop's local server, so clients written for that one work too.
- **Syncs end-to-end encrypted.** An account on a Cloudflare Worker (D1 for documents, R2 for
  attachments) keeps your Macs in step. Each Mac encrypts notes, titles, IDs and attachments before
  they leave it, with a key the server never sees. Conflicts keep both versions: the losing one
  stays in the note's history.
- **Keeps history.** Earlier versions of each note, including every version replaced by the API,
  sync or an import, and a daily database backup.
- **Imports your notes.** Kanso can copy in an Inkdrop library: notes, notebooks, tags, statuses
  and attachments. Its files are only read, and importing again picks up newer edits without
  reverting notes you changed in Kanso.

## Download

Builds for macOS (Apple silicon), Windows (x64) and Linux (x64 and arm64) are on the
[Releases page](https://github.com/iamngoni/kanso/releases). `SHA256SUMS` lists each download's
checksum (`shasum -a 256 -c SHA256SUMS --ignore-missing` checks the ones you have).

**macOS** (`Kanso-<version>-macos-arm64.dmg`): open the disk image and drag Kanso to Applications.
It is signed with a Developer ID and notarized by Apple, so it opens like any other Mac app.
Once installed, Kanso updates itself: it checks for new releases and installs them when you choose
Restart now.

**Windows** (`Kanso-<version>-windows-x64-setup.exe`): run the installer. It installs for you only,
into `%LOCALAPPDATA%\Programs\Kanso`, without asking for an administrator, and adds Kanso to Start
(and, if you tick it, the desktop); uninstall it from Settings › Apps. It isn't code-signed yet, so
SmartScreen may say "Windows protected your PC": choose More info › **Run anyway**. Prefer no
installer? `Kanso-<version>-windows-x64.zip` holds the same `kanso.exe` to put anywhere you can write
to. Either way Kanso updates itself. Notes live in `%APPDATA%\Kanso` and stay when Kanso is
uninstalled unless you ask for them to go; sign-ins go to the Windows Credential Manager.

**Linux** (`Kanso-<version>-linux-x64.tar.gz` or `-linux-arm64.tar.gz`, built on Ubuntu 22.04 so
glibc 2.35 or newer): unpack it and run `./install.sh`, which puts `kanso` in `~/.local/bin` and Kanso
in your applications menu (`./install.sh --uninstall` removes them). Kanso runs on Wayland or X11 and
needs Vulkan graphics drivers (Mesa's are fine). Notes live in `~/.local/share/kanso`
(`$XDG_DATA_HOME/kanso`); sign-ins go to the Secret Service (GNOME Keyring or KWallet), so one must be
running to stay signed in.

To build the macOS app yourself:

```bash
scripts/bundle-macos.sh
```

It builds an ad-hoc signed `target/release/Kanso.app`; copy it to `/Applications`. On Windows and
Linux, `cargo build --release -p kanso-app` builds `target/release/kanso`. On first run with an
empty library, Kanso offers to import any Inkdrop library it finds.

## Using it

**Importing from Inkdrop.** **Import from Inkdrop…**, in the Kanso menu and the command palette
(or `kanso import-inkdrop`), copies an Inkdrop library in. The local API listens on port 19840; if another app already uses that port, the
plug icon at the bottom of the sidebar turns amber, and Kanso keeps trying until it's free or you
choose another port in Settings › Local API. To keep MCP clients you had set up for Inkdrop, **Use
settings from Inkdrop…** on that page takes its port, username and password; macOS asks you to allow
access to the Keychain item "inkdrop Safe Storage", where Inkdrop keeps them encrypted.

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

### Shortcuts

On Windows and Linux, Ctrl takes the place of ⌘ (Ctrl+K, Ctrl+Shift+F and so on), and Alt the place
of ⌥.

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

Every command takes `--data-dir <dir>`. The default is `$KANSO_DATA_DIR` when set, otherwise
`~/Library/Application Support/Kanso` on macOS, `%APPDATA%\Kanso` on Windows and
`~/.local/share/kanso` (`$XDG_DATA_HOME/kanso`) on Linux.

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
| `kanso-server` | The local HTTP API (compatible with Inkdrop's) and the native MCP endpoint |
| `kanso-sync` | Accounts and end-to-end encrypted sync, and the `kanso-sync` command (`verify`, `estimate`, `unbind`) |
| `kanso-app` | The GPUI app and the `kanso` command |
| `cloud/` | The sync Worker (TypeScript, D1, R2) and its admin script |

### Cutting a release

On a clean, up-to-date `master`:

```bash
scripts/release.sh 0.2.0 --push
```

It bumps the version, writes the release's section of `CHANGELOG.md`, commits, tags `v0.2.0` and
pushes. The tag starts [.github/workflows/release.yml](.github/workflows/release.yml), which builds
every platform, signs and notarizes the Mac app, signs the update feed, checks every file and
publishes the release; installed copies then update themselves. If any check fails, nothing is
published. Running the workflow by hand (Actions › Release › Run workflow) builds the same
downloads, versioned `<version>-dev.<commit>`, as workflow artifacts without making a release.
[docs/RELEASING.md](docs/RELEASING.md) has the details, including `scripts/release.sh publish` for
a release stuck as a draft.

Design notes and decisions are in [docs/PLAN.md](docs/PLAN.md); accounts and encrypted sync in
[docs/ACCOUNTS.md](docs/ACCOUNTS.md).
