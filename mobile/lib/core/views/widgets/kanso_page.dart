// Kanso — kanso_page.dart.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

/// A page body with explicit Kanso-owned background and keyboard behavior.
///
/// The body intentionally owns its own safe areas, just as it did under
/// [Scaffold]. When [resizeToAvoidBottomInset] is true, the body gives up the
/// space occupied by the software keyboard while keeping the original
/// [MediaQuery] available to descendants.
class KansoPage extends StatelessWidget {
  const KansoPage({
    required this.body,
    this.backgroundColor,
    this.resizeToAvoidBottomInset = true,
    super.key,
  });

  final Widget body;
  final Color? backgroundColor;
  final bool resizeToAvoidBottomInset;

  @override
  Widget build(BuildContext context) {
    final keyboard = resizeToAvoidBottomInset
        ? MediaQuery.viewInsetsOf(context).bottom
        : 0.0;
    final content = keyboard == 0
        ? body
        : AnimatedPadding(
            duration: const Duration(milliseconds: 150),
            curve: Curves.easeOut,
            padding: EdgeInsets.only(bottom: keyboard),
            child: body,
          );
    final theme = Theme.of(context);
    return ColoredBox(
      color: backgroundColor ?? theme.scaffoldBackgroundColor,
      child: DefaultTextStyle(
        style: theme.textTheme.bodyMedium!,
        child: MediaQuery.removeViewInsets(
          context: context,
          removeBottom: resizeToAvoidBottomInset,
          child: content,
        ),
      ),
    );
  }
}
