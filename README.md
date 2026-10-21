# Kanso

A local-first Markdown notes app for macOS, with Windows and Linux builds, built with [GPUI](https://www.gpui.rs) through
[gpui-kit](https://github.com/longbridge/gpui-kit).

[kansoapp.app](https://kansoapp.app) has downloads and a tour; the website's source is in [`site/`](site).

**Writing**
- Brackets, quotes and Markdown marks pair as you type; lists and tasks continue on Return and
  indent with Tab; tables line up as you edit them.
- Smart paste turns a pasted link into Markdown and keeps code as code; ⇧⌘V pastes plain text.
- Typewriter scrolling, a word goal per note, focus mode, and optional Vim keys.

**Linking**
- `[[Wiki-links]]` with autocomplete, backlinks with their context, unlinked mentions, and an
  outline of the note's headings beside the editor.
- ⌘P finds notes by title and by heading.

**Capture**
- Today's note (⌘D) in a Journal notebook, and templates with `{{date}}`, `{{time}}` and `{{title}}`.
- Quick capture from anywhere with ⌃⌥Space, or `kanso capture` from a terminal, into an Inbox.
- Nested tags (`work/clients`) shown as a tree, with `#` completion in the editor.

**Preview**
- Mermaid diagrams, math, and Excalidraw drawings, drawn in the note's theme.
- Images resize from the preview or with `![alt|300](image.png)`.

**Tasks**
- Due dates (`- [ ] Send the draft @2026-10-25`), a Tasks view of what's overdue, due today and
  coming up, and reminders as notifications.

**Finding and changing**
- Search with filters, find and replace across every note with one undo, and each note's
  version history with what changed since each version.

**Sharing**
- Export a note as a web page or PDF, or publish it as an end-to-end encrypted link: the key
  stays in the link, so the server only holds ciphertext.

**Locked notes**
- Lock a note with a password (**Lock note…** in the palette or the Note menu). Its text is sealed
  on this device, stays sealed in sync, and opens on your other devices with the same password.

**Sync, agents and the rest**
- **End-to-end encrypted sync.** An account on a Cloudflare Worker keeps your devices in step.
  Each device encrypts notes, titles, IDs and attachments before they leave it, with a key the
  server never sees. Conflicts keep both versions: the losing one stays in the note's history.
- **A local API for agents.** `http://127.0.0.1:19840` is a REST API over your notes, and `/mcp`
  on the same port is a native MCP endpoint. Its REST API matches Inkdrop's local server, so
  clients written for that work too.
- **History and backups.** Earlier versions of each note, including every version replaced by the
  API, sync or an import, and a daily database backup.
- **Imports.** Bring in a folder of Markdown files, an Obsidian vault or a Logseq graph, or an
  export from Notion, Evernote, Bear, Joplin, Standard Notes, Simplenote, UpNote or Inkdrop, with
  their notebooks, tags, links and attachments. The source is only read, you see what will come
  in before anything is written, and importing again picks up newer edits without reverting notes
  you changed in Kanso.
- Themes, light and dark, and updates that install themselves.

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

### Importing

**File › Import…** (also in the Kanso menu and the command palette) asks where your notes are,
then shows what it found before writing anything: how many notes, notebooks, tags and attachments,
what is new or changed since a last import, and what it will skip and why. **Import** brings them
in with a progress bar and a report. The source is only ever read. Each import is keyed by where
a note came from (a vault-relative path, or the other app's own ID), so importing the same source
again updates its notes instead of duplicating them, and leaves alone any note you changed or
deleted in Kanso since. **File › Open…** brings in a single `.md` or `.txt` file (and the pictures
it links to) as a note in the Inbox and opens it; opening it again updates that note.

| Source | Pick | What comes in |
|---|---|---|
| Markdown folder | the folder | Every `.md`, `.markdown` and `.txt` file. Subfolders become nested notebooks. YAML front matter gives the title, tags, aliases and created/updated dates; otherwise the first `# heading`, then the file name, is the title and the file's dates are used. Other front-matter keys stay at the top of the note. Linked pictures and files become attachments; links between the files become links between notes. |
| Obsidian vault | the vault's folder | All of the above, with the file name as the title, `[[wiki-links]]` (headings and aliases kept), `![[embeds]]` of pictures and files (`\|300` widths kept; embedded notes become links), `#tags` and `#nested/tags` in the text and front matter, and the attachment folder from Settings › Files and links. Callouts stay as Markdown. `.obsidian/`, `.trash/` and `.canvas` files are skipped and listed. |
| Logseq graph | the graph's folder | Pages and journals, `title::`, `tags::` and `alias::` properties, `[[page refs]]`, and `a___b` namespaced pages as `a/b`. The `logseq/` folder is skipped. |
| Notion | the `.zip` from Settings › Export (Markdown & CSV), or its folder | Pages with the 32-character IDs taken off names and titles, sub-pages in a notebook named after their page, links between pages (including `notion.so` links to pages in the export) and files. A database becomes a note with its table, the first column linking its rows; its rows are notes with their properties. Large exports split into several zips work as they come. |
| Evernote | an `.enex` file, or a folder of them | Each `.enex` is a notebook. Notes become Markdown (headings, lists, checklists, tables, code blocks, links), with their tags, dates and attachments. |
| Bear | a `.bear2bk` backup, or an Export Notes folder (TextBundle or Markdown) | Notes with their pictures and files, `#tags`, `#nested/tags` and `#multi word tags#`, pinned, archived (in a notebook of their own) and trashed notes. |
| Joplin | a `.jex` file, or a RAW export folder | Notebooks and sub-notebooks, tags, attachments and links between notes. |
| Standard Notes | a decrypted backup (`.txt`/`.json`, or the `.zip`) | Notes, nested tags, pinned, archived and trashed notes. Encrypted backups can't be read; export a decrypted one. |
| Simplenote | the `.zip` from Export Notes, or its `notes.json` | Notes (the first line is the title), tags, pinned and trashed notes. |
| UpNote | a folder from Export › Markdown | Read as a Markdown folder. |
| Inkdrop | found on this computer | Notes, notebooks, tags, statuses and attachments, keeping their IDs (below). |

Attachments larger than 10 MB, the most a Kanso attachment can be, are skipped and listed, and
links to them stay as they were. Apple Notes has no export of its own; tools that save its notes
as Markdown files make a folder Kanso can import.

From a terminal, with Kanso closed, `kanso import <kind> <path>` does the same (`kanso import help`
lists the kinds); `--dry-run` prints the summary and writes nothing.

**Importing from Inkdrop.** Choosing Inkdrop in **Import…** (or `kanso import-inkdrop`) copies an
Inkdrop library in, keeping its IDs and revisions. The local API listens on port 19840; if another app already uses that port, the
plug icon at the bottom of the sidebar turns amber, and Kanso keeps trying until it's free or you
choose another port in Settings › Local API. To keep MCP clients you had set up for Inkdrop, **Use
settings from Inkdrop…** on that page takes its port, username and password; macOS asks you to allow
access to the Keychain item "inkdrop Safe Storage", where Inkdrop keeps them encrypted.

**Sync.** Settings › Account › **Create account** picks a username and a password of at least 12
characters; this device's notes are encrypted and uploaded. Kanso then shows a **recovery key**: save
it somewhere safe (a password manager, or printed). Kanso can't reset a forgotten password. The
recovery key sets a new one (**Forgot your password?**), and a new key replaces it afterwards. On
your other devices, **Sign in** with the same username and password. A device that already has notes asks
whether to add them to the account or replace them with the account's notes (after saving a backup
of its library). Signing out keeps every note on the device. The Account page also lists your devices,
signs one out, changes the password and deletes the account.

The default server is the owner's Worker, `kanso.antonlabs.cc`, which takes new
accounts only when its owner opens sign-ups. To run your own, deploy the Worker in
[cloud/](cloud/README.md) to your Cloudflare account and choose **Use a different server…** before
creating the account. An account belongs to its server's address, so pick the final URL first.

### Locked notes

**Lock note…** (palette, or the Note menu) seals the open note's text with your lock password. The
first time, Kanso asks you to set one, at least 8 characters; on another device, or once notes are
locked, it asks for that password instead. **Kanso can't recover a forgotten lock password**, and the
notes locked with it can't be read without it.

Once you type the password, the notes locked with it open until **Lock notes now**, quitting, or ten
minutes without opening or saving one. **Remove lock…** keeps a note's text as plain text again, and
**Change lock password…** seals every locked note again with a new password.

What locking does and doesn't do:

- The text is sealed with XChaCha20-Poly1305 under a key Argon2id derives from the password (256 MiB,
  three passes), bound to the note, and kept in the note itself (`kansoLock`). The note's body in the
  database is empty, so search, backlinks, tasks, reminders, related notes and the local API (REST
  and MCP) never see it. Sync carries the sealed text inside the note's own encrypted document.
- Locking deletes the note's earlier versions, its embedding and the search index's words, and SQLite
  overwrites the space they took. Daily backups made before you locked it still hold the text until
  they are replaced, after 14 days.
- The title, notebook, tags, status and dates stay visible, and pictures and files attached to a
  locked note aren't sealed.
- The local API reads a locked note with an empty body. It can move, retitle or tag the note, but
  can't change its text, take the lock off, or delete it outright (moving it to the trash still works).
- Export All Notes leaves locked notes out, and a locked note can't be published.
- Every device needs this version of Kanso: an older one shows a locked note as empty.

### Shortcuts

On Windows and Linux, Ctrl takes the place of ⌘ (Ctrl+K, Ctrl+Shift+F and so on), and Alt the place
of ⌥.

| | |
|---|---|
| ⌘K | Command palette: every command, notebook, tag and status, plus note search |
| ⌘P | Open a note by title |
| ⌘N / ⇧⌘N | New note / notebook |
| ⌘D | Today's note |
| ⌃⌥Space | Quick capture from any app (change it in Settings › General) |
| ⇧⌘F | Search all notes (`book:`, `tag:`, `status:`, `title:`, `body:`, `"phrases"`, `-exclude`) |
| ⇧⌘H | Find and replace in all notes |
| ⌘L | Search the list that is showing |
| ↑ ↓ or J K in the list, Return | Move through notes, edit the selected one |
| ⇧⌘L | Back to the note list |
| ⌥⌘↓ / ⌥⌘↑ | Next / previous note from anywhere |
| ⌘1 / ⌘2 / ⌘3 | Edit / split (drag the divider) / preview |
| ⇧⌘D | Focus mode: only the note; Escape leaves it |
| ⌘\ | Fold the sidebar to icons, or open it again |
| ⌥⌘I | Outline and backlinks beside the note |
| ⇧⌘V | Paste as plain text |
| ⇧⌘P | Pin or unpin |
| ⌘S | Save now (notes also save on their own) |
| ⌘, | Settings, inside the window; Escape goes back to the notes |

### Command line

```bash
kanso                       # open the app
kanso serve                 # only the API (REST and /mcp), no window
kanso import obsidian ~/Vault --dry-run   # what an import would bring in
kanso import notion ~/Downloads/Export.zip  # markdown, obsidian, notion, evernote, bear, …
kanso import-inkdrop        # import from Inkdrop; --credentials also copies its API settings
kanso capture "Call Sam"    # save a note to the Inbox (--book <name> picks the notebook)
pbpaste | kanso capture     # with no text, it reads standard input
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

## Licence

Kanso is free software under the [GNU Affero General Public License, version 3 or any later
version](LICENSE) (`AGPL-3.0-or-later`): the app, its crates and the sync Worker in [cloud/](cloud/).
You may use, study, change and share it. If you share a changed Kanso, or run a changed Worker as a
service other people use, you must offer them its source under the same licence.

Contributions are welcome. By opening a pull request you agree that your contribution is licensed
under the same terms.

Kanso's dependencies keep their own licences (MIT, Apache-2.0, BSD and similar; a few files under
MPL-2.0).
