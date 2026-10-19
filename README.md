# Kanso

A local-first Markdown notes app for macOS, with Windows and Linux builds, built with [GPUI](https://www.gpui.rs) through
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

## Download

Builds for macOS (Apple silicon), Windows (x64) and Linux (x64 and arm64) are on the
[Releases page](https://github.com/iamngoni/kanso/releases). `SHA256SUMS` lists each download's
checksum (`shasum -a 256 -c SHA256SUMS --ignore-missing` checks the ones you have).

**macOS** (`Kanso-<version>-macos-arm64.dmg`): open the disk image and drag Kanso to Applications.
The app is ad-hoc signed and not notarized, so the first time macOS says it can't check it for
malware. Click Done, then System Settings › Privacy & Security › **Open Anyway** next to "Kanso was
blocked", and confirm (or Control-click Kanso in Applications and choose Open). Each new build has a
different signature, so macOS may ask once per build to let Kanso use its Keychain items; choose
Always Allow.

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
empty library, Kanso offers to import from Inkdrop.

## Using it

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

**Other people's Macs.** The builds are ad-hoc signed and not notarized; see [Download](#download)
for getting past Gatekeeper the first time.

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
| `kanso-server` | Inkdrop-compatible HTTP API and the native MCP endpoint |
| `kanso-sync` | Accounts and end-to-end encrypted sync, and the `kanso-sync` command (`verify`, `estimate`, `unbind`) |
| `kanso-app` | The GPUI app and the `kanso` command |
| `cloud/` | The sync Worker (TypeScript, D1, R2) and its admin script |

### Cutting a release

[.github/workflows/release.yml](.github/workflows/release.yml) builds the four downloads on GitHub's
runners and attaches them, with `SHA256SUMS`, to a **draft** release. To cut one:

1. Set `version` under `[workspace.package]` in `Cargo.toml` to the new version (the workflow stops
   if the tag and this disagree), and commit it on `master`.
2. Tag and push the tag:

   ```bash
   git tag v0.2.0 && git push origin v0.2.0
   ```

3. When the workflow finishes, open the draft on the Releases page, edit the generated notes and
   press **Publish release**.

Running the workflow by hand (Actions › Release › Run workflow) builds the same downloads, versioned
`<version>-dev.<commit>`, as workflow artifacts without making a release. The macOS build is ad-hoc
signed, not notarized; the Windows build isn't code-signed.

Design notes and decisions are in [docs/PLAN.md](docs/PLAN.md); accounts and encrypted sync in
[docs/ACCOUNTS.md](docs/ACCOUNTS.md).
