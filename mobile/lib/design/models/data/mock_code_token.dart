// Kanso — mock_code_token.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:freezed_annotation/freezed_annotation.dart';

import 'mock_token_kind.dart';

part 'mock_code_token.freezed.dart';

/// A piece of a code line and what it is.
@freezed
abstract class MockCodeToken with _$MockCodeToken {
  const factory MockCodeToken(String text, MockTokenKind kind) = _MockCodeToken;
}
