// Kanso — page_for_screen.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/widgets.dart';

import '../models/data/design_layout.dart';
import '../models/data/design_screen.dart';
import '../views/pages/error_mock_page.dart';
import '../views/pages/library_mock_page.dart';
import '../views/pages/locked_note_mock_page.dart';
import '../views/pages/note_edit_mock_page.dart';
import '../views/pages/note_list_mock_page.dart';
import '../views/pages/note_preview_mock_page.dart';
import '../views/pages/search_mock_page.dart';
import '../views/pages/settings_mock_page.dart';
import '../views/pages/tablet_mock_page.dart';
import '../views/pages/theme_picker_mock_page.dart';

/// The page that draws [screen] for [layout].
Widget pageForScreen(DesignScreen screen, DesignLayout layout) {
  if (layout == DesignLayout.tablet) {
    return TabletMockPage(screen: screen);
  }
  return switch (screen) {
    DesignScreen.library => const LibraryMockPage(),
    DesignScreen.libraryScrolled => const LibraryMockPage(scrolled: true),
    DesignScreen.noteList => const NoteListMockPage(),
    DesignScreen.notePreview => const NotePreviewMockPage(),
    DesignScreen.noteEdit => const NoteEditMockPage(),
    DesignScreen.search => const SearchMockPage(),
    DesignScreen.settings => const SettingsMockPage(),
    DesignScreen.settingsScrolled => const SettingsMockPage(scrolled: true),
    DesignScreen.themePicker => const ThemePickerMockPage(),
    DesignScreen.themePickerScrolled => const ThemePickerMockPage(
      scrolled: true,
    ),
    DesignScreen.lockedNote => const LockedNoteMockPage(),
    DesignScreen.error => const ErrorMockPage(),
  };
}
