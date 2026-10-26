// Kanso — bucket.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:freezed_annotation/freezed_annotation.dart';

part 'bucket.freezed.dart';

/// What a header of the note list groups: the pinned notes, or the notes of
/// one day.
@freezed
sealed class Bucket with _$Bucket {
  /// The pinned notes.
  const factory Bucket.pinned() = PinnedBucket;

  /// The notes of one local day, numbered in days since 1970-01-01.
  const factory Bucket.day(int localDay) = DayBucket;
}
