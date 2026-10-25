// Kanso — kanso_empty_state.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../configs/kanso_spacing.dart';
import '../../utils/context_extensions.dart';

/// A quiet message for a place with nothing in it yet: a title, one line
/// that says what will appear and, when there is something to do, the
/// [action] that does it. No picture: the words are enough.
class KansoEmptyState extends StatelessWidget {
  const KansoEmptyState({
    required this.title,
    required this.message,
    this.action,
    super.key,
  });

  final String title;
  final String message;

  /// The next step, usually a `KansoButton`.
  final Widget? action;

  /// The widest the message grows, so its lines stay short.
  static const double maxWidth = 300;

  @override
  Widget build(BuildContext context) {
    final typography = context.typography;
    final next = action;
    return Center(
      child: Padding(
        padding: const EdgeInsets.all(KansoSpacing.md),
        child: ConstrainedBox(
          constraints: const BoxConstraints(maxWidth: maxWidth),
          child: Column(
            mainAxisSize: MainAxisSize.min,
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: <Widget>[
              Text(
                title,
                textAlign: TextAlign.center,
                style: typography.heading2,
              ),
              const SizedBox(height: KansoSpacing.xs),
              Text(
                message,
                textAlign: TextAlign.center,
                style: typography.secondary,
              ),
              if (next != null) ...<Widget>[
                const SizedBox(height: KansoSpacing.lg),
                Center(child: next),
              ],
            ],
          ),
        ),
      ),
    );
  }
}
