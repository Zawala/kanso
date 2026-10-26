// Kanso — library_fixtures.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:kanso/core/bridge/api/types.dart';

/// A note for tests, in the notebook [notebookId].
NoteView noteView(String id, {String notebookId = 'b_1'}) {
  return NoteView(
    id: id,
    rev: '1-$id',
    title: 'Note $id',
    body: 'The text of $id.',
    lock: const LockState.plain(),
    notebookId: notebookId,
    tags: const <String>[],
    status: 'active',
    pinned: false,
    createdMs: 1700000000000,
    updatedMs: 1700000000000,
  );
}

/// A notebook for tests.
NotebookRow notebookRow(
  String id, {
  String? name,
  int count = 0,
  int depth = 0,
  String? parentId,
  NotebookRole? role,
}) {
  return NotebookRow(
    id: id,
    name: name ?? 'Notebook $id',
    parentId: parentId,
    count: count,
    locked: false,
    role: role,
    depth: depth,
  );
}

/// A tag for tests.
TagRow tagRow(String id, {String? name, int count = 1}) {
  return TagRow(id: id, name: name ?? id, color: 0x2185D0, count: count);
}
