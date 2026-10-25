# Kanso for iPhone, iPad and Android

The Flutter app of [Kanso](../README.md), tracked in
[#48](https://github.com/iamngoni/kanso/issues/48). This folder holds the
scaffold: the design system (colours, type, icons, spacing, motion), the
shared appearance state, the test and golden-image tooling, and CI. The
screens come in later steps.

- Flutter 3.44.8 (stable), Dart 3.12, strict analysis.
- iOS and iPadOS 17 or later; Android 10 (API 29) or later; phones and tablets.
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

## Check

```sh
dart format --output=none --set-exit-if-changed lib test
flutter analyze
flutter test
```

## Design proposal

Until the real screens are built, the app opens on a gallery of static mock
screens (`lib/design/`) for the owner to review: the library, note list, note
(preview and edit), search, settings, theme picker, a locked note and an error
card, for a phone and for a tablet in landscape, with a light, dark or system
switch. The mock data is fixed (`lib/design/models/data/mock_data.dart`). The
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
with no tolerance. Against the stored file a diff of up to 0.5% of the pixels
is accepted (`test/helpers/kanso_golden_comparator.dart`), because the images
are generated on one Mac and compared on another, and text rasterisation
differs slightly between machines. Only one file is kept per screen.

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
| `flutter_bloc` | Widgets for BLoCs | MIT |
| `freezed_annotation` | Annotations for generated models, events and states | MIT |
| `json_annotation` | Annotations for generated JSON conversion | BSD-3-Clause |
| `get_it` | Service locator | MIT |
| `injectable` | Registration annotations for GetIt | MIT |
| `flutter_svg` | Draws the Lucide icons | MIT |
| `intl` | Localisation (ARB) support | BSD-3-Clause |
| `flutter_localizations` | Flutter's own localisation delegates (Flutter SDK) | BSD-3-Clause |

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

Bundled files:

| Files | Used for | Licence |
| --- | --- | --- |
| Instrument Sans (`assets/fonts/InstrumentSans-*.ttf`) | Interface and note text | SIL Open Font Licence 1.1 (`assets/fonts/OFL-InstrumentSans.txt`) |
| JetBrains Mono (`assets/fonts/JetBrainsMono-*.ttf`) | Code | SIL Open Font Licence 1.1 (`assets/fonts/OFL-JetBrainsMono.txt`) |
| Lucide icons (`assets/icons/lucide/*.svg`) | Icons, the same set as the desktop app | ISC, with some icons MIT (`assets/icons/lucide/LICENSE`) |

The fonts are static instances (weights 400, 500, 600, 700 and italic 400 of
Instrument Sans; 400 and 700 of JetBrains Mono) cut from the variable fonts
that Google Fonts publishes, at the default width. The licences are added to
Flutter's licence page at start-up (`lib/core/utils/license_registration.dart`).
