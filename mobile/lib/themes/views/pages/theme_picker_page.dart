// Kanso — theme_picker_page.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/utils/context_extensions.dart';
import '../../../shell/views/widgets/shell_back_button.dart';
import '../../../shell/views/widgets/shell_page.dart';

/// Settings › Theme: the themes the app can be drawn in.
class ThemePickerPage extends StatelessWidget {
  const ThemePickerPage({super.key});

  @override
  Widget build(BuildContext context) {
    return ShellPage(
      title: context.l10n.themesTitle,
      leading: const ShellBackButton(),
      backgroundColor: context.colors.app,
    );
  }
}
