# Kanso for iPhone, iPad and Android

The Flutter app of [Kanso](../README.md), tracked in
[#48](https://github.com/iamngoni/kanso/issues/48). This folder holds the
scaffold: the design system (colours, type, icons, spacing, motion), the
shared appearance state, the test and golden-image tooling, and CI, and the
Rust core behind a bridge (`rust/`, `lib/core/bridge/`). The screens come in
later steps.

- Flutter 3.44.8 (stable), Dart 3.12, strict analysis.
- iOS and iPadOS 17 or later; Android 10 (API 29) or later; phones and tablets.
- The Rust toolchain (rustup; the versions are pinned in `rust/rust-toolchain.toml`)
  and, for Android, the NDK Flutter uses: the library is built by
  `hook/build.dart` during `flutter run`, `build` and `test`.
- iOS builds use Swift packages for plugins (no CocoaPods; `pubspec.yaml` turns
  it on for this project) and run on Apple silicon simulators only
  (`ios/Flutter/*.xcconfig` excludes x86_64).
- Bundle id `app.kanso.Kanso` on iOS, application id `app.kanso.kanso` on
  Android.

## Run

```sh
cd mobile
flutter pub get
dart run build_runner build --delete-conflicting-outputs
flutter gen-l10n
flutter run
```

`flutter run` starts the production entry point (`lib/main.dart` calls
`lib/main_production.dart`). `flutter run -t lib/main_development.dart`
starts the development one. There are no native flavours: one binary, two Dart
entry points.

Generated code (`*.freezed.dart`, `*.g.dart`, `injection.config.dart` and the
localisation classes in `lib/gen/l10n/`) is committed. CI regenerates it and
fails if the result differs, so run the two generators and commit after
changing a Freezed class, an injectable class or an ARB file.

## The Rust core

The notes, locked notes, sync and drawing are `crates/kanso-mobile`, the same
core the desktop app is built on. `rust/` wraps it for Dart with
[flutter_rust_bridge](https://github.com/fzyzcjy/flutter_rust_bridge) (2.13.0,
pinned): `rust/src/api/` is what Dart sees, and the generated Dart is
`lib/core/bridge/` (committed). After changing `rust/src/api/`:

```sh
cargo install flutter_rust_bridge_codegen --version 2.13.0 --locked
flutter_rust_bridge_codegen generate   # from this folder
dart run build_runner build            # Freezed for the generated enums
```

`hook/build.dart` builds the library as a native asset, with the phones'
profile (`[profile.mobile]` in the root `Cargo.toml`) and the compilers of
`../scripts/mobile/env.sh`. `flutter test` runs it too; tests that call the
core open the library it built (`test/helpers/core_library.dart`). The app
opens its library at `kanso/library.db` in the application support folder
(`lib/core/configs/core_module.dart`); the background sync opens the same file.
The home page lists a "Rust core" row that opens `lib/core_check/`, real calls
to the library.

## Check

```sh
dart format --output=none --set-exit-if-changed lib test
flutter analyze
flutter test
```

## Design proposal

Until the real screens are built, the app opens on a gallery of static mock
screens (`lib/design/`) for the owner to review: the library, note list (and
its loading and empty states), note (preview and edit), search, settings,
fonts, theme picker, a locked note and an error card, for a phone and for a
tablet in landscape, with a light, dark or system switch. The mock data is
fixed (`lib/design/models/data/mock_data.dart`). The Fonts screen is live:
choosing a font or a text size there restyles every note in the running app
(`AppearanceBloc`; nothing is saved yet). The
components the screens are made of (`lib/core/views/widgets/`) are the ones
the real app will use. The images in `test/design/views/pages/goldens/` are
the proposal as pictures.

## Golden images

Golden tests render a widget as iOS and as Android and compare both renders
with the same file, so any difference between the platforms fails. The helper
is `expectGoldenOnBothPlatforms` in `test/helpers/golden.dart`; the images sit
in a `goldens/` folder next to the test that wrote them. Text scale is 1 and
animations are off. The bundled fonts are loaded in `test/flutter_test_config.dart`.

The two renders are also compared with each other pixel by pixel, in memory,
with no tolerance. Against the stored file, anti-aliasing is ignored
(`test/helpers/kanso_golden_comparator.dart`): a pixel counts as changed only
when a channel moves by more than 32 of 255, and at most 0.3% of the pixels
may change. The images are generated on one Mac and compared on another, and
text rasterisation differs slightly between machines. Only one file is kept per screen.
Shadows are drawn blurred, as on a device (the test binding would otherwise
draw them as hard slabs).

Update the images after an intended visual change, look at them, and commit
them:

```sh
flutter test --update-goldens
```

Images are generated on macOS and compared on macOS in CI; do not update them
on another system.

## Conventions

Dart code follows the owner's Flutter rules (the `ngoni-flutter` skill):
BLoC only, one folder per BLoC under `state/<bloc_name>/`, Freezed for models,
events and states, GetIt with Injectable, one public widget per file,
relative imports, ARB localisations for all visible copy, and the lint
configuration in `analysis_options.yaml` (Very Good Analysis with strict
casts, inference and raw types).

Colours are not chosen here. `lib/core/configs/kanso_colors.dart` holds the
tokens of Kanso's own theme files, and `test/core/configs/kanso_colors_test.dart`
reads `../themes/kanso-dark.json` and `kanso-light.json` so the app and the
files cannot drift apart.

## Dependencies

Every dependency is MIT, BSD or Apache licensed.

| Package | Used for | Licence |
| --- | --- | --- |
| `bloc` | BLoC state machines | MIT |
| `go_router` | Routes, deep links and the back button (BSD-3-Clause, flutter.dev) | BSD-3-Clause |
| `bloc_concurrency` | `sequential` and `restartable` event transformers | MIT |
| `flutter_bloc` | Widgets for BLoCs | MIT |
| `freezed_annotation` | Annotations for generated models, events and states | MIT |
| `json_annotation` | Annotations for generated JSON conversion | BSD-3-Clause |
| `get_it` | Service locator | MIT |
| `injectable` | Registration annotations for GetIt | MIT |
| `flutter_svg` | Draws the Lucide icons | MIT |
| `intl` | Localisation (ARB) support | BSD-3-Clause |
| `flutter_localizations` | Flutter's own localisation delegates (Flutter SDK) | BSD-3-Clause |
| `flutter_rust_bridge` | Calls the Rust core (2.13.0, pinned) | MIT |
| `flutter_rust_bridge_hooks` | Builds the Rust core as a native asset (2.13.0, pinned); brings `native_toolchain_rust` (MIT) and `hooks` (BSD-3-Clause) | MIT |
| `code_assets` | The build hook reads the target platform | BSD-3-Clause |
| `path` | The build hook's file paths | BSD-3-Clause |
| `dartz` | `Either` for repository results | MIT |
| `path_provider` | The folder the library lives in | BSD-3-Clause |

The Rust library adds `core-foundation` and `core-foundation-sys` (Apple
targets only, MIT or Apache-2.0, already in the iOS tree through
`security-framework`) to mark the library's folder as excluded from backups.

Development only:

| Package | Used for | Licence |
| --- | --- | --- |
| `build_runner` | Runs the code generators | BSD-3-Clause |
| `freezed` | Generates models, events and states | MIT |
| `json_serializable` | Generates JSON conversion | BSD-3-Clause |
| `injectable_generator` | Generates the GetIt registrations | MIT |
| `very_good_analysis` | Lint rules | MIT |
| `bloc_test` | BLoC tests | MIT |
| `mocktail` | Mocks | MIT |
| `glados` | Property-based tests | MIT |
| `integration_test` | Tests on a simulator or an emulator (Flutter SDK) | BSD-3-Clause |

Bundled files:

| Files | Used for | Licence |
| --- | --- | --- |
| Instrument Sans (`assets/fonts/InstrumentSans-*.ttf`) | Interface; the default note font | SIL Open Font Licence 1.1 (`assets/fonts/OFL-InstrumentSans.txt`) |
| Inter (`assets/fonts/Inter-Variable*.ttf`) | A note font (sans) | SIL Open Font Licence 1.1 (`assets/fonts/OFL-Inter.txt`) |
| Source Serif 4 (`assets/fonts/SourceSerif4-Variable*.ttf`) | A note font (serif) | SIL Open Font Licence 1.1 (`assets/fonts/OFL-SourceSerif4.txt`) |
| Literata (`assets/fonts/Literata-Variable*.ttf`) | A note font (serif) | SIL Open Font Licence 1.1 (`assets/fonts/OFL-Literata.txt`) |
| JetBrains Mono (`assets/fonts/JetBrainsMono-*.ttf`) | The default code font | SIL Open Font Licence 1.1 (`assets/fonts/OFL-JetBrainsMono.txt`) |
| IBM Plex Mono (`assets/fonts/IBMPlexMono-*.ttf`) | A code (and note) font | SIL Open Font Licence 1.1 (`assets/fonts/OFL-IBMPlexMono.txt`) |
| Lucide icons (`assets/icons/lucide/*.svg`) | Icons, the same set as the desktop app | ISC, with some icons MIT (`assets/icons/lucide/LICENSE`) |

Instrument Sans and JetBrains Mono are static instances (weights 400, 500,
600, 700 and italic 400 of Instrument Sans; 400 and 700 of JetBrains Mono) cut
from the variable fonts that Google Fonts publishes, at the default width.
Inter, Source Serif 4 and Literata are variable fonts, one file for the roman
and one for the italic, weight axis (`wght`) kept in full; the optical-size
axis was dropped with fonttools' instancer (it stays at the font's default)
to save about 2 MB. Flutter draws the weight a style asks for from the `wght`
axis, so styles use plain `FontWeight`s. IBM Plex Mono is published as static
files; 400, 500, 600, 700 and italic 400 are bundled. All are SIL Open Font
Licence 1.1, taken from `github.com/google/fonts` (`ofl/`); together the new
families add about 4.2 MB of font files.

Notes are set in the editor, preview and code fonts the person chooses
(`KansoFonts`), scaled by the text size (`KansoTextSize`: note text only, on
top of the device's own text size, which is never clamped); the interface
stays in Instrument Sans. A note font can also be "System", the device's own
font (San Francisco on iOS, the system font on Android): the one setting that
looks different per platform on purpose, so no golden uses it.

The licences are added to Flutter's licence page at start-up
(`lib/core/utils/license_registration.dart`).
