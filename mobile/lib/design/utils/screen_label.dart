// Kanso — screen_label.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import '../../gen/l10n/app_localizations.dart';
import '../models/data/design_screen.dart';

/// The name of a mock screen.
extension DesignScreenLabel on AppLocalizations {
  /// What the gallery calls [screen].
  String screenName(DesignScreen screen) {
    return switch (screen) {
      DesignScreen.library => screenLibrary,
      DesignScreen.libraryScrolled => screenLibraryScrolled,
      DesignScreen.noteList => screenNoteList,
      DesignScreen.loadingList => screenLoadingList,
      DesignScreen.emptyNotebook => screenEmptyNotebook,
      DesignScreen.notePreview => screenNotePreview,
      DesignScreen.noteEdit => screenNoteEdit,
      DesignScreen.search => screenSearch,
      DesignScreen.settings => screenSettings,
      DesignScreen.settingsScrolled => screenSettingsScrolled,
      DesignScreen.fonts => screenFonts,
      DesignScreen.fontsScrolled => screenFontsScrolled,
      DesignScreen.themePicker => screenThemePicker,
      DesignScreen.themePickerScrolled => screenThemePickerScrolled,
      DesignScreen.lockedNote => screenLockedNote,
      DesignScreen.error => screenError,
    };
  }
}
