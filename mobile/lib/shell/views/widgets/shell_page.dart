// Kanso — shell_page.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_top_bar.dart';

/// A page with a bar and room for its [child]: the frame of the screens that
/// are routes of the app, before their content is built.
class ShellPage extends StatelessWidget {
  const ShellPage({
    required this.title,
    this.leading,
    this.trailing = const <Widget>[],
    this.backgroundColor,
    this.child,
    super.key,
  });

  final String title;
  final Widget? leading;
  final List<Widget> trailing;

  /// The page's colour; the editor's when null.
  final Color? backgroundColor;
  final Widget? child;

  @override
  Widget build(BuildContext context) {
    final background = backgroundColor ?? context.colors.editor;
    return Scaffold(
      resizeToAvoidBottomInset: false,
      backgroundColor: background,
      body: Column(
        children: <Widget>[
          KansoTopBar(
            backgroundColor: background,
            leading: leading,
            title: title,
            trailing: trailing,
          ),
          Expanded(child: child ?? const SizedBox.shrink()),
        ],
      ),
    );
  }
}
