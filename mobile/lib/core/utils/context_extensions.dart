// Kanso — context_extensions.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../gen/l10n/app_localizations.dart';
import '../configs/kanso_colors.dart';
import '../configs/kanso_syntax_colors.dart';
import '../configs/kanso_typography.dart';

/// Short access to Kanso's tokens and strings from a [BuildContext].
extension KansoBuildContext on BuildContext {
  /// The interface colours of the current theme.
  KansoColors get colors => Theme.of(this).extension<KansoColors>()!;

  /// The Markdown and code colours of the current theme.
  KansoSyntaxColors get syntaxColors =>
      Theme.of(this).extension<KansoSyntaxColors>()!;

  /// The type scale of the current theme.
  KansoTypography get typography =>
      Theme.of(this).extension<KansoTypography>()!;

  /// The app's strings, in the current language.
  AppLocalizations get l10n => AppLocalizations.of(this);
}
