// Kanso — neighbour_note.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

/// The id of the note one place on from [current] in [ids], the notes of a
/// list in order, or null when there is nowhere to go.
///
/// With no note open (or one that is not in the list) the first step forward
/// lands on the first note and the first step back on the last. A step
/// that would leave the list stays where it is.
String? neighbourNote(
  List<String> ids,
  String? current, {
  required bool forward,
}) {
  if (ids.isEmpty) {
    return null;
  }
  final at = current == null ? -1 : ids.indexOf(current);
  if (at < 0) {
    return forward ? ids.first : ids.last;
  }
  final to = forward ? at + 1 : at - 1;
  if (to < 0 || to >= ids.length) {
    return null;
  }
  return ids[to];
}
