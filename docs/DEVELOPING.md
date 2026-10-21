# Developing Kanso

## Kanso and Kanso Dev

`/Applications/Kanso.app` only ever comes from a published release, and
updates itself from there ([RELEASING.md](RELEASING.md)). To try a change in
a real app without touching it, build **Kanso Dev**:

```sh
scripts/bundle-macos.sh --dev             # target/kanso-dev/release/Kanso Dev.app
scripts/bundle-macos.sh --dev --install   # and copy it to ~/Applications/Kanso Dev.app
```

The flavour is chosen when Kanso is compiled: the `dev-flavour` feature of
`kanso-app` (`cargo run -p kanso-app --features dev-flavour` works too).
`crates/kanso-app/src/flavour.rs` holds everything that differs:

| | Kanso | Kanso Dev |
|---|---|---|
| Name | Kanso | Kanso Dev, with a DEV badge in the sidebar and an amber DEV band on its icon |
| Bundle ID, login item | `app.kanso.Kanso` | `app.kanso.Kanso.dev` |
| Data folder | `~/Library/Application Support/Kanso`, `%APPDATA%\Kanso`, `~/.local/share/kanso` | `…/Kanso Dev`, `%APPDATA%\Kanso Dev`, `~/.local/share/kanso-dev` |
| Keychain service | `Kanso` | `Kanso Dev` |
| Local API port (new config) | 19840 | 19841 |
| Updates | from releases | off: Settings › Updates says "Development build — updates are off" |

So Kanso Dev never opens or writes the release's library, config or account,
and never replaces itself or `/Applications/Kanso.app`. `--install` is only
for `--dev`, and installs to `~/Applications`; the release workflow builds
the normal flavour. `KANSO_DATA_DIR` or `--data-dir` still point either
flavour at another folder.

A plain `cargo run -p kanso-app` is the normal flavour, but a debug build: it
checks for updates and shows them, and never installs one (it offers the
release page).

## Tests

```sh
PROPTEST_RNG_SEED=20261017 cargo test --workspace
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo fmt --all --check
```

New logic gets property tests alongside example tests; the seed is pinned so
a failure in CI reproduces here. The views are checked as PNGs without
capturing the screen:

```sh
KANSO_SNAPSHOT_DIR=/tmp/shots cargo test -q -p kanso-app snapshots -- --ignored --nocapture
KANSO_SNAPSHOT_ONLY=dark-update-ready,light-settings-updates ...   # a few
```

Voice input's real transcription test needs the Parakeet model (about 670 MB)
in a folder of its own; it is skipped otherwise:

```sh
KANSO_VOICE_MODEL_DIR=~/Library/Application\ Support/Kanso\ Dev/models/parakeet-tdt-0.6b-v3 \
    cargo test --release -p kanso-voice --test model -- --ignored --nocapture
```

ONNX Runtime is linked into the app statically; its build script downloads
pyke's prebuilt archive once and checks it. On macOS the bundle carries the
microphone entitlement (`packaging/macos/Kanso.entitlements`), without which
the hardened runtime keeps the microphone from Kanso and macOS never asks.
