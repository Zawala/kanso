// Kanso — kanso_font_family.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'kanso_font_category.dart';

/// A family of text the person can pick for writing, reading or code.
///
/// Every family but [system] is bundled with the app (SIL Open Font
/// Licence) and draws the same on every device. [system] is the one
/// deliberate exception: the device's own font, San Francisco on iOS and
/// the system font (Roboto on most devices) on Android.
enum KansoFontFamily {
  /// Kanso's own font, the default for writing and reading.
  instrumentSans(
    displayName: 'Instrument Sans',
    family: 'Instrument Sans',
    category: KansoFontCategory.sans,
  ),

  /// A neutral sans designed for screens.
  inter(
    displayName: 'Inter',
    family: 'Inter',
    category: KansoFontCategory.sans,
  ),

  /// A serif with optical sizes, for long reading.
  sourceSerif4(
    displayName: 'Source Serif 4',
    family: 'Source Serif 4',
    category: KansoFontCategory.serif,
  ),

  /// A serif made for e-readers.
  literata(
    displayName: 'Literata',
    family: 'Literata',
    category: KansoFontCategory.serif,
  ),

  /// Kanso's own monospaced font, the default for code.
  jetBrainsMono(
    displayName: 'JetBrains Mono',
    family: 'JetBrains Mono',
    category: KansoFontCategory.mono,
  ),

  /// A monospaced font with a typewriter's character.
  ibmPlexMono(
    displayName: 'IBM Plex Mono',
    family: 'IBM Plex Mono',
    category: KansoFontCategory.mono,
  ),

  /// The device's own font.
  system(
    displayName: 'System',
    family: null,
    category: KansoFontCategory.system,
  );

  const KansoFontFamily({
    required this.displayName,
    required this.family,
    required this.category,
  });

  /// The name shown in the settings (in English; "System" is translated by
  /// the interface).
  final String displayName;

  /// The family name registered with Flutter, or null for the device's.
  final String? family;

  /// The kind of family.
  final KansoFontCategory category;

  /// Whether the font ships in the app rather than coming from the device.
  bool get bundled => family != null;

  /// Whether the family is monospaced.
  bool get monospaced => category == KansoFontCategory.mono;

  /// The families offered for writing: every bundled one, then the
  /// device's.
  static const List<KansoFontFamily> editorChoices = <KansoFontFamily>[
    instrumentSans,
    inter,
    sourceSerif4,
    literata,
    jetBrainsMono,
    ibmPlexMono,
    system,
  ];

  /// The families offered for reading a note: the proportional ones.
  static const List<KansoFontFamily> previewChoices = <KansoFontFamily>[
    instrumentSans,
    inter,
    sourceSerif4,
    literata,
    system,
  ];

  /// The families offered for code: the monospaced ones.
  static const List<KansoFontFamily> codeChoices = <KansoFontFamily>[
    jetBrainsMono,
    ibmPlexMono,
  ];
}
