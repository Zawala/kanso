// Kanso — kanso_swipe_action.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:ui';

import 'package:freezed_annotation/freezed_annotation.dart';

import '../../configs/kanso_icons.dart';

part 'kanso_swipe_action.freezed.dart';

/// One of the actions a swipe shows behind a row: a picture, a word, a
/// colour and what a tap does.
@freezed
abstract class KansoSwipeAction with _$KansoSwipeAction {
  const factory KansoSwipeAction({
    required KansoIcons icon,

    /// What it says, under the picture and to a screen reader.
    required String label,

    /// The colour behind it.
    required Color background,

    /// What a tap on it does.
    required VoidCallback onPressed,
  }) = _KansoSwipeAction;
}
