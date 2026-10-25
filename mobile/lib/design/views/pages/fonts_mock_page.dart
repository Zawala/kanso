// Kanso — fonts_mock_page.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/utils/context_extensions.dart';
import '../widgets/fonts_pane.dart';

/// Settings › Fonts on a phone, full screen. With [scrolled] it is shown at
/// its foot. It needs the app's `AppearanceBloc` above it.
class FontsMockPage extends StatelessWidget {
  const FontsMockPage({this.scrolled = false, super.key});

  final bool scrolled;

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      resizeToAvoidBottomInset: false,
      backgroundColor: context.colors.app,
      body: FontsPane(startAtEnd: scrolled),
    );
  }
}
