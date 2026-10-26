// Kanso — fonts_page.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/utils/context_extensions.dart';
import '../../../shell/views/widgets/shell_back_button.dart';
import '../../../shell/views/widgets/shell_page.dart';

/// Settings › Fonts: the fonts of notes and the size of their text.
///
/// [atTextSize] opens it at the text size section.
class FontsPage extends StatelessWidget {
  const FontsPage({this.atTextSize = false, super.key});

  final bool atTextSize;

  @override
  Widget build(BuildContext context) {
    return ShellPage(
      title: context.l10n.fontsTitle,
      leading: const ShellBackButton(),
      backgroundColor: context.colors.app,
    );
  }
}
