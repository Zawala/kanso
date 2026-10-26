// Kanso — kanso_syntax_style.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:freezed_annotation/freezed_annotation.dart';

part 'kanso_syntax_style.freezed.dart';

/// How a syntax token is set besides its colour: slanted or not, and how
/// heavy. A null attribute is the plain text's own.
@freezed
abstract class KansoSyntaxStyle with _$KansoSyntaxStyle {
  const factory KansoSyntaxStyle({
    /// Whether the token is italic; null leaves it as the text is.
    bool? italic,

    /// The weight, 100 to 900; null leaves it as the text is.
    int? weight,
  }) = _KansoSyntaxStyle;
}
