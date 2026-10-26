// Kanso — property.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:math';
import 'dart:ui';

import 'package:glados/glados.dart'
    show
        Any,
        AnyUtils,
        BoolAny,
        CombinableAny,
        ExploreConfig,
        Generator,
        GeneratorUtils,
        IntAnys,
        ListAnys,
        StringAnys;
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/models/data/kanso_font_family.dart';
import 'package:kanso/core/models/data/kanso_fonts.dart';
import 'package:kanso/core/models/data/kanso_text_size.dart';
import 'package:kanso/core/models/data/preferences.dart';
import 'package:kanso/core/models/data/theme_selection.dart';

/// Property tests use this configuration so a failure in CI reproduces
/// locally: the random source has a fixed seed.
ExploreConfig kansoExploreConfig() {
  return ExploreConfig(random: Random(20261025));
}

extension KansoAny on Any {
  /// Any opaque sRGB colour.
  Generator<Color> get opaqueColor => combine3(
    intInRange(0, 256),
    intInRange(0, 256),
    intInRange(0, 256),
    (int r, int g, int b) => Color.fromARGB(255, r, g, b),
  );
}

extension KansoFontsAny on Any {
  /// Any font family, the device's own included.
  Generator<KansoFontFamily> get fontFamily => choose(KansoFontFamily.values);

  /// Any text size.
  Generator<KansoTextSize> get textSize => choose(KansoTextSize.values);

  /// Any combination of the three fonts.
  Generator<KansoFonts> get fonts => combine3(
    fontFamily,
    fontFamily,
    fontFamily,
    (KansoFontFamily editor, KansoFontFamily preview, KansoFontFamily code) =>
        KansoFonts(editorFont: editor, previewFont: preview, codeFont: code),
  );

  /// A text scale a device may set, from 0.5 to 3.5.
  Generator<double> get deviceTextScale =>
      intInRange(50, 351).map((percent) => percent / 100);
}

/// The characters Markdown and search care about, plus letters and space.
const List<String> _markdownAlphabet = <String>[
  'a',
  'B',
  'y',
  's',
  'n',
  'c',
  ' ',
  '*',
  '`',
  '[',
  ']',
  '(',
  ')',
];

extension KansoMarkdownAny on Any {
  /// Short strings made of the characters Markdown and search care about, so
  /// marks pair up, break and overlap often.
  Generator<String> get markdownish {
    return listWithLengthInRange<String>(
      0,
      14,
      choose(_markdownAlphabet),
    ).map((chars) => chars.join());
  }
}

extension KansoErrorAny on Any {
  /// Any error the Rust library can fail with, with any message.
  Generator<KansoError> get kansoError => combine2(
    intInRange(0, 9),
    nonEmptyLetterOrDigits,
    (int kind, String text) => switch (kind) {
      0 => const KansoError.notFound(),
      1 => const KansoError.locked(),
      2 => KansoError.conflict(currentRev: text),
      3 => KansoError.invalid(message: text),
      4 => KansoError.storage(message: text),
      5 => KansoError.network(message: text),
      6 => const KansoError.signedOut(),
      7 => KansoError.auth(message: text),
      _ => KansoError.internal(message: text),
    },
  );
}

/// Theme names: a few, so picks collide with the slots now and then.
const List<String> _themeIds = <String>[
  'kanso-light',
  'kanso-dark',
  'nord',
  'github-light',
  'dracula',
];

extension KansoThemeSelectionAny on Any {
  /// Any built-in theme's name, out of five.
  Generator<String> get themeId => choose(_themeIds);

  /// Any selection of themes.
  Generator<ThemeSelection> get themeSelection => combine4(
    this.bool,
    themeId,
    themeId,
    themeId,
    (bool follow, String ui, String light, String dark) => ThemeSelection(
      followSystem: follow,
      ui: ui,
      light: light,
      dark: dark,
    ),
  );

  /// Any preferences.
  Generator<Preferences> get preferences => combine6(
    themeSelection,
    fonts,
    textSize,
    choose(SortKey.values),
    this.bool,
    this.bool,
    (
      ThemeSelection themes,
      KansoFonts fonts,
      KansoTextSize size,
      SortKey sort,
      bool descending,
      bool seen,
    ) => Preferences(
      themeSelection: themes,
      fonts: fonts,
      textSize: size,
      sort: sort,
      descending: descending,
      backupNoticeSeen: seen,
    ),
  );
}
