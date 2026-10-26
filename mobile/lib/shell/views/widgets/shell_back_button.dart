// Kanso — shell_back_button.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:go_router/go_router.dart';

import '../../../core/configs/kanso_icons.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_icon_button.dart';

/// The chevron that goes back one page.
class ShellBackButton extends StatelessWidget {
  const ShellBackButton({super.key});

  @override
  Widget build(BuildContext context) {
    return KansoIconButton(
      icon: KansoIcons.chevronLeft,
      semanticLabel: context.l10n.settingsBack,
      onPressed: () => context.pop(),
    );
  }
}
