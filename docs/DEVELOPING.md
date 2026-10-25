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

## Mobile targets

The core the mobile app shares (`kanso-core`, `kanso-store`, `kanso-engine`,
`kanso-render`, `kanso-collab`, `kanso-import`, `kanso-sync` and `kanso-ai`,
the last two without their desktop features, plus `kanso-tls`) builds for
`aarch64-apple-ios`, `aarch64-apple-ios-sim`, `aarch64-linux-android`,
`armv7-linux-androideabi` and `x86_64-linux-android`. Add the targets once
with `rustup target add`; Android needs the NDK (`ANDROID_NDK_HOME`, else the
newest in `~/Library/Android/sdk/ndk`), which `scripts/mobile/env.sh` points
the C compilers and bindgen at.

```sh
scripts/mobile-check.sh                  # cargo check for the five targets, the banned-dependency look, clippy
scripts/mobile-test.sh ios-sim           # the tests, in a booted iPhone simulator
scripts/mobile-test.sh android           # the tests, on the one device or emulator adb lists
scripts/mobile-size.sh                   # the size table: iOS and Android, full and minimal, unwinding and aborting
```

- `mobile-test.sh` runs each crate's library and integration tests (not
  doctests) through a Cargo runner: `KANSO_IOS_SIM_UDID` picks a simulator,
  `ANDROID_SERIAL` a device, and `KANSO_TEST_CRATES="kanso-sync kanso-ai"`
  narrows the run. On Android the test binaries and the fixtures they read are
  copied to `/data/local/tmp/kanso`, so fixture paths are read from
  `CARGO_MANIFEST_DIR` at run time, not when the test is built. An Apple-silicon
  Mac runs `aarch64-linux-android` in an arm64 emulator; CI runs an x86-64 one
  at API 29 (`.github/workflows/mobile-core.yml`, only for changes that can
  affect it).
- `cargo build --profile mobile` is the phones' release build: `opt-level =
  "z"`, fat LTO, one codegen unit, and `panic = "unwind"`, so the facade
  catches a panic and returns an error. The desktop's `[profile.release]` is
  unchanged. `crates/kanso-mobile-size` is a temporary stand-in for the
  library, only there to measure; the mobile library replaces it.
- rquickjs ships no bindings for phones, so `kanso-render` builds them with
  libclang (Xcode's, or the NDK's) on iOS and Android. A bindgen setting for
  your own Mac in `BINDGEN_EXTRA_CLANG_ARGS` would be passed to the phones'
  targets too; the scripts unset it.
- Phones check servers' certificates with the system (`kanso-tls`). On
  Android that needs the app's `Context` first; until the app hands it over,
  requests over HTTPS fail with an error. A bare `adb shell` test binary has no
  JVM, so the real certificate check on Android is tested in the app.
