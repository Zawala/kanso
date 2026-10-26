// Kanso — fonts_page.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/utils/context_extensions.dart';
import '../widgets/fonts_pane.dart';

/// Settings › Fonts: the fonts of notes and the size of their text.
///
/// [atTextSize] opens it at the text size section.
class FontsPage extends StatelessWidget {
  const FontsPage({this.atTextSize = false, super.key});

  final bool atTextSize;

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      resizeToAvoidBottomInset: false,
      backgroundColor: context.colors.app,
      body: FontsPane(startAtEnd: atTextSize),
    );
  }
}
