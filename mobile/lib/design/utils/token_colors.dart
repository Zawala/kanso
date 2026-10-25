// Kanso — token_colors.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:ui';

import '../../core/configs/kanso_syntax_colors.dart';
import '../models/data/mock_token_kind.dart';

/// The syntax colour of a kind of code.
extension MockTokenColor on MockTokenKind {
  /// This kind's colour in [syntax].
  Color colorIn(KansoSyntaxColors syntax) {
    return switch (this) {
      MockTokenKind.plain => syntax.codeBlock,
      MockTokenKind.keyword => syntax.keyword,
      MockTokenKind.function => syntax.function,
      MockTokenKind.typeName => syntax.typeName,
      MockTokenKind.variable => syntax.variable,
      MockTokenKind.property => syntax.property,
      MockTokenKind.operator => syntax.operator,
      MockTokenKind.punctuation => syntax.punctuation,
      MockTokenKind.comment => syntax.comment,
    };
  }
}
