// Kanso — tablet_mock_page.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/configs/kanso_sizes.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_scrim.dart';
import '../../../core/views/widgets/kanso_sheet.dart';
import '../../../shell/views/widgets/tablet_layout.dart';
import '../../models/data/design_layout.dart';
import '../../models/data/design_screen.dart';
import '../widgets/fonts_pane.dart';
import '../widgets/note_edit_pane.dart';
import '../widgets/settings_pane.dart';
import '../widgets/signed_out_card.dart';
import '../widgets/theme_picker_pane.dart';

/// A tablet screen in landscape: the library, the list and the note side by
/// side, as empty panes, with a sheet or a card over them when the screen is
/// a modal one.
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
    final Widget note = screen == DesignScreen.noteEdit
        ? const NoteEditPane(layout: tablet)
        : ColoredBox(color: colors.editor);
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
                library: ColoredBox(color: colors.sidebar),
                list: ColoredBox(color: colors.list),
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
