// Kanso — tablet_mock_page.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/configs/kanso_sizes.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_scrim.dart';
import '../../../core/views/widgets/kanso_sheet.dart';
import '../../models/data/design_layout.dart';
import '../../models/data/design_screen.dart';
import '../../models/data/mock_data.dart';
import '../widgets/empty_notebook_pane.dart';
import '../widgets/fonts_pane.dart';
import '../widgets/library_pane.dart';
import '../widgets/loading_list_pane.dart';
import '../widgets/locked_note_pane.dart';
import '../widgets/no_note_selected_pane.dart';
import '../widgets/note_edit_pane.dart';
import '../widgets/note_list_pane.dart';
import '../widgets/note_preview_pane.dart';
import '../widgets/search_pane.dart';
import '../widgets/settings_pane.dart';
import '../widgets/signed_out_card.dart';
import '../widgets/tablet_layout.dart';
import '../widgets/theme_picker_pane.dart';

/// A tablet screen in landscape: the library, the list and the note side by
/// side, with a sheet or a card over them when the screen is a modal one.
///
/// It is drawn at 1180 by 820 points; on anything smaller it is scaled down
/// to fit.
class TabletMockPage extends StatelessWidget {
  const TabletMockPage({required this.screen, super.key});

  final DesignScreen screen;

  /// The size the layout is drawn at.
  static const Size designSize = Size(1180, 820);

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
    const tablet = DesignLayout.tablet;
    final scrolled =
        screen == DesignScreen.settingsScrolled ||
        screen == DesignScreen.fontsScrolled ||
        screen == DesignScreen.themePickerScrolled;
    final Widget list = switch (screen) {
      DesignScreen.search => const SearchPane(),
      DesignScreen.loadingList => const LoadingListPane(layout: tablet),
      DesignScreen.emptyNotebook => const EmptyNotebookPane(layout: tablet),
      _ => NoteListPane(
        layout: tablet,
        selectedTitle: screen == DesignScreen.lockedNote
            ? MockData.lockedNoteTitle
            : MockData.openNoteTitle,
      ),
    };
    final Widget note = switch (screen) {
      DesignScreen.noteEdit => const NoteEditPane(layout: tablet),
      DesignScreen.lockedNote => const LockedNotePane(layout: tablet),
      DesignScreen.loadingList ||
      DesignScreen.emptyNotebook => const NoNoteSelectedPane(),
      _ => const NotePreviewPane(layout: tablet),
    };
    final Widget? overlay = switch (screen) {
      DesignScreen.settings || DesignScreen.settingsScrolled => KansoSheet(
        height: designSize.height - 2 * KansoSizes.touchTarget,
        child: SettingsPane(startAtEnd: scrolled),
      ),
      DesignScreen.fonts || DesignScreen.fontsScrolled => KansoSheet(
        height: designSize.height - 2 * KansoSizes.touchTarget,
        child: FontsPane(startAtEnd: scrolled),
      ),
      DesignScreen.themePicker ||
      DesignScreen.themePickerScrolled => KansoSheet(
        height: designSize.height - 2 * KansoSizes.touchTarget,
        child: ThemePickerPane(layout: tablet, startAtEnd: scrolled),
      ),
      DesignScreen.error => const SignedOutCard(),
      _ => null,
    };
    final content = SizedBox.fromSize(
      size: designSize,
      child: Scaffold(
        resizeToAvoidBottomInset: false,
        backgroundColor: colors.editor,
        body: Stack(
          children: <Widget>[
            Positioned.fill(
              child: TabletLayout(
                library: LibraryPane(
                  selectedNotebook: screen == DesignScreen.emptyNotebook
                      ? MockData.emptyNotebookName
                      : null,
                ),
                list: list,
                note: note,
              ),
            ),
            if (overlay != null) ...<Widget>[
              const Positioned.fill(child: KansoScrim()),
              Center(child: overlay),
            ],
          ],
        ),
      ),
    );
    return LayoutBuilder(
      builder: (context, constraints) {
        if (constraints.maxWidth >= designSize.width &&
            constraints.maxHeight >= designSize.height) {
          return content;
        }
        return FittedBox(child: content);
      },
    );
  }
}
