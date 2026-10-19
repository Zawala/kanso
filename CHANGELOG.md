# Changelog

Every release of Kanso, newest first. Each entry links its commit, and each
heading compares the release with the one before.

## [0.1.1](https://github.com/iamngoni/kanso/compare/v0.1.0...v0.1.1) - 2026-10-19

### Fixed

- **release:** Brace variables before an ellipsis in release.sh ([6f819da](https://github.com/iamngoni/kanso/commit/6f819da6a4ae2585478dad4c27d0eceb4c308ab8))

## [0.1.0](https://github.com/iamngoni/kanso/commits/v0.1.0) - 2026-10-19

The first release: a calm, local-first Markdown notes app that replaces
Inkdrop, built on GPUI for macOS, Windows and Linux. What follows sums up
the history so far; every commit is in [the history up to
v0.1.0](https://github.com/iamngoni/kanso/commits/v0.1.0).

### New

- Kanso updates itself from signed releases, says clearly when an update is ready, and shows what's new after it installs ([6bb7570](https://github.com/iamngoni/kanso/commit/6bb7570))
- Notes, notebooks within notebooks, tags and statuses in a SQLite store with trigram search, revisions and daily backups ([169d0e4](https://github.com/iamngoni/kanso/commit/169d0e4))
- Import from Inkdrop, inside the app or from the command line, keeping Kanso's own edits on a re-import ([e34c7bc](https://github.com/iamngoni/kanso/commit/e34c7bc), [e5479c4](https://github.com/iamngoni/kanso/commit/e5479c4))
- The editor with a live preview, side by side or alone, a draggable split and focus mode ([a531b38](https://github.com/iamngoni/kanso/commit/a531b38), [d0e68c2](https://github.com/iamngoni/kanso/commit/d0e68c2))
- The note list grouped by day, with search matches shown in the excerpts ([d7744b9](https://github.com/iamngoni/kanso/commit/d7744b9))
- The command palette and Open Note ([d268159](https://github.com/iamngoni/kanso/commit/d268159))
- Backlinks and each note's history, with a way back to any earlier version ([4b45548](https://github.com/iamngoni/kanso/commit/4b45548))
- Export a note, or every note, as Markdown ([0e35966](https://github.com/iamngoni/kanso/commit/0e35966))
- An Inkdrop-compatible local API and a native MCP endpoint, so Inkdrop's clients and MCP server keep working ([88ba474](https://github.com/iamngoni/kanso/commit/88ba474), [d5fd4d5](https://github.com/iamngoni/kanso/commit/d5fd4d5))
- Accounts with end-to-end encrypted sync between your computers, recovery keys, and a Settings page for the account and its devices ([f2d5a5a](https://github.com/iamngoni/kanso/commit/f2d5a5a), [666c77f](https://github.com/iamngoni/kanso/commit/666c77f), [b5fbac9](https://github.com/iamngoni/kanso/commit/b5fbac9))
- Settings inside the main window, and opening at login ([157cd61](https://github.com/iamngoni/kanso/commit/157cd61))
- Twenty themes, separate UI, syntax and preview themes, and importing Inkdrop and Zed themes ([1f9d3ff](https://github.com/iamngoni/kanso/commit/1f9d3ff), [8f050bc](https://github.com/iamngoni/kanso/commit/8f050bc))
- Code blocks colored in 23 languages, with a copy button ([3709232](https://github.com/iamngoni/kanso/commit/3709232), [27f54a0](https://github.com/iamngoni/kanso/commit/27f54a0))
- An app icon: an ink ensō with a vermilion seal ([c403439](https://github.com/iamngoni/kanso/commit/c403439))
- Builds for Windows and Linux as well as macOS ([62ec295](https://github.com/iamngoni/kanso/commit/62ec295))

### Improved

- A warm graphite and paper look and a calmer writing surface ([2e1b76b](https://github.com/iamngoni/kanso/commit/2e1b76b))
- Typing stays fast in long notes ([e0d7ad0](https://github.com/iamngoni/kanso/commit/e0d7ad0))

### Fixed

- Typed text is never lost, through saves, syncs and a note's revision moving underneath it ([f6a6583](https://github.com/iamngoni/kanso/commit/f6a6583), [9442143](https://github.com/iamngoni/kanso/commit/9442143))
- Sync keeps local edits through its own echoes, lost answers and restores ([7b0d30d](https://github.com/iamngoni/kanso/commit/7b0d30d), [652b439](https://github.com/iamngoni/kanso/commit/652b439), [3ab3f29](https://github.com/iamngoni/kanso/commit/3ab3f29))
- Recovery keys are kept until they're saved, and replaced with every password change ([645c5cd](https://github.com/iamngoni/kanso/commit/645c5cd), [ff8f89f](https://github.com/iamngoni/kanso/commit/ff8f89f))
- Auth keys, proofs and tokens stay out of logs and freed memory ([3bda3bd](https://github.com/iamngoni/kanso/commit/3bda3bd))
- Three rounds of review findings in the main window ([61f005e](https://github.com/iamngoni/kanso/commit/61f005e), [f325bfc](https://github.com/iamngoni/kanso/commit/f325bfc), [18b218d](https://github.com/iamngoni/kanso/commit/18b218d))

### Under the hood

- Property tests with a pinned seed across the workspace, and PNG snapshots of the real views ([ff9515f](https://github.com/iamngoni/kanso/commit/ff9515f))
- A release workflow that builds every platform into a draft release ([62ec295](https://github.com/iamngoni/kanso/commit/62ec295))
