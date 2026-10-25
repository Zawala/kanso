// Kanso — theme_picker_mock_page.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/utils/context_extensions.dart';
import '../../models/data/design_layout.dart';
import '../widgets/theme_picker_pane.dart';

/// The theme picker on a phone, full screen. With [scrolled] it is shown at
/// its end.
class ThemePickerMockPage extends StatelessWidget {
  const ThemePickerMockPage({this.scrolled = false, super.key});

  final bool scrolled;

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      resizeToAvoidBottomInset: false,
      backgroundColor: context.colors.app,
      body: ThemePickerPane(layout: DesignLayout.phone, startAtEnd: scrolled),
    );
  }
}
