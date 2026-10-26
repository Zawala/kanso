// Kanso — list_options.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:freezed_annotation/freezed_annotation.dart';

import '../../bridge/api/types.dart';

part 'list_options.freezed.dart';

/// How a list of notes is ordered and narrowed, as the desktop's list menu
/// has it: what to sort by and which way, a status to narrow to, and
/// whether to list pinned notes only.
///
/// The sort and its direction are one choice for every place and are saved;
/// the narrowing starts over in each place.
@freezed
abstract class ListOptions with _$ListOptions {
  const factory ListOptions({
    /// What the list is sorted by.
    @Default(SortKey.updated) SortKey sort,

    /// Whether it runs from the largest value to the smallest: the newest
    /// first, Z to A.
    @Default(true) bool descending,

    /// The one status to list, as the library writes it (`onHold`); null
    /// lists any.
    String? statusOnly,

    /// Whether to list pinned notes only.
    @Default(false) bool pinnedOnly,
  }) = _ListOptions;

  const ListOptions._();

  /// The query that lists [place] with these options.
  ListQuery queryFor(ListPlace place) => ListQuery(
    place: place,
    statusOnly: statusOnly,
    pinnedOnly: pinnedOnly,
    sort: sort,
    descending: descending,
  );

  /// These options sorted by [key], in the direction the desktop gives a
  /// new sort: dates newest first, titles A to Z.
  ListOptions sortedBy(SortKey key) =>
      copyWith(sort: key, descending: key != SortKey.title);

  /// Whether anything narrows the list.
  bool get narrowed => statusOnly != null || pinnedOnly;

  /// Whether a list of [place] is sorted by the person's choice: Recent is
  /// always newest edit first.
  static bool sorts(ListPlace place) => place is! ListPlace_Recent;

  /// Whether a list of [place] can be narrowed to pinned notes: Pinned is
  /// that already.
  static bool narrowsToPinned(ListPlace place) => place is! ListPlace_Pinned;
}
