// Kanso — home_page.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_icon.dart';

/// The page the app opens on until the real screens arrive.
class HomePage extends StatelessWidget {
  const HomePage({super.key});

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      body: SafeArea(
        child: Center(
          child: Padding(
            padding: const EdgeInsets.all(KansoSpacing.lg),
            child: Column(
              mainAxisSize: MainAxisSize.min,
              children: <Widget>[
                KansoIcon(
                  KansoIcons.book,
                  size: KansoSpacing.xxl,
                  color: context.colors.accent,
                ),
                const SizedBox(height: KansoSpacing.md),
                Text(
                  context.l10n.appName,
                  style: context.typography.largeTitle,
                ),
                const SizedBox(height: KansoSpacing.xs),
                Text(
                  context.l10n.homePlaceholder,
                  style: context.typography.secondary,
                  textAlign: TextAlign.center,
                ),
              ],
            ),
          ),
        ),
      ),
    );
  }
}
