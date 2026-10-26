// Kanso — kanso_shortcut_bindings.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';

import '../models/data/kanso_intent.dart';

/// The keyboard shortcuts of the browsing screens on [platform], as the
/// desktop's: "secondary" is Command on Apple devices and Control elsewhere.
///
/// | Shortcut | Does |
/// | --- | --- |
/// | secondary + `[` and `]` | back and forward through the notes opened |
/// | secondary + shift + F | search all notes |
/// | secondary + `,` | settings |
/// | secondary + `\` | show or hide the library |
/// | secondary + option + down and up | the next and the previous note |
/// | secondary + shift + P | pin or unpin the open note |
///
/// Not yet: a new note, find in a note, the modes of a note and the command
/// palette, which come with what they act on.
Map<ShortcutActivator, Intent> kansoShortcutBindings(TargetPlatform platform) {
  final apple =
      platform == TargetPlatform.iOS || platform == TargetPlatform.macOS;
  SingleActivator secondary(
    LogicalKeyboardKey key, {
    bool shift = false,
    bool alt = false,
  }) {
    return SingleActivator(
      key,
      meta: apple,
      control: !apple,
      shift: shift,
      alt: alt,
    );
  }

  return <ShortcutActivator, Intent>{
    secondary(LogicalKeyboardKey.bracketLeft): const BackIntent(),
    secondary(LogicalKeyboardKey.bracketRight): const ForwardIntent(),
    secondary(LogicalKeyboardKey.keyF, shift: true): const SearchAllIntent(),
    secondary(LogicalKeyboardKey.comma): const SettingsIntent(),
    secondary(LogicalKeyboardKey.backslash): const ToggleLibraryIntent(),
    secondary(LogicalKeyboardKey.arrowDown, alt: true): const NextNoteIntent(),
    secondary(LogicalKeyboardKey.arrowUp, alt: true):
        const PreviousNoteIntent(),
    secondary(LogicalKeyboardKey.keyP, shift: true): const PinNoteIntent(),
  };
}
