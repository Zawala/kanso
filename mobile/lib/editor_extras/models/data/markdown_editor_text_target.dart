// Kanso — markdown_editor_text_target.dart
// Created 2026-10-28.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/foundation.dart';
import 'package:flutter/services.dart';

import '../../../core/bridge/api/types.dart';
import '../../../editor/views/controllers/kanso_markdown_controller.dart';
import 'editor_text_target.dart';

/// Adapts the windowed Markdown field to whole-note feature edits.
///
/// Ranges and selections exposed to features are always whole-note UTF-16
/// offsets. Writes go through the editor document's Rust command path so
/// they retain autosave, highlighting, and one-step undo behavior.
class MarkdownEditorTextTarget extends ChangeNotifier
    implements EditorTextTarget {
  MarkdownEditorTextTarget({required this.controller, required this.noteId}) {
    controller.addListener(changed);
  }

  final KansoMarkdownController controller;

  @override
  final String noteId;

  int _revision = 0;
  bool _active = true;
  @override
  int get revision => _revision;

  @override
  int get windowOrigin => controller.windowStart;

  @override
  bool get active => _active;

  @override
  TextEditingValue get value => TextEditingValue(
    text: controller.noteText,
    selection: controller.noteSelection,
  );

  @override
  Listenable get changes => this;

  void changed() {
    if (_active) {
      _revision++;
      notifyListeners();
    }
  }

  /// Marks a document reload or replacement even when the visible text is
  /// byte-for-byte equal to the previous snapshot.
  void invalidate() {
    if (_active) {
      _revision++;
      notifyListeners();
    }
  }

  @override
  void replace(TextRange range, String text) {
    if (!_active ||
        !range.isValid ||
        range.start < 0 ||
        range.end < range.start ||
        range.end > controller.noteLength) {
      return;
    }
    controller.run(
      EditCommand.insert(
        range: Range16(start: range.start, end: range.end),
        text: text,
      ),
    );
  }

  /// What the last [revise] put in, while it can still be joined.
  _Revised? _revised;

  @override
  void commitRevision() {
    _revised = null;
  }

  @override
  void revise(TextRange range, String text, {required bool joining}) {
    final last = _revised;
    _revised = null;
    if (!_active || !_fits(range)) {
      return;
    }
    if (joining &&
        last != null &&
        last.version == controller.version &&
        range.start >= last.start &&
        range.end <= last.start + last.inserted.length &&
        _join(last, range, text)) {
      return;
    }
    final removed = controller.noteText.substring(range.start, range.end);
    final before = controller.version;
    replace(range, text);
    if (controller.version != before) {
      _revised = _Revised(
        start: range.start,
        removed: removed,
        inserted: text,
        version: controller.version,
      );
    }
  }

  /// Takes back [last], the only step since, and makes it again with
  /// [range] of it replaced by [text]: one step for both. False, with the
  /// note as it was, when the step taken back was not [last].
  bool _join(_Revised last, TextRange range, String text) {
    final inserted = last.inserted.replaceRange(
      range.start - last.start,
      range.end - last.start,
      text,
    );
    if (!controller.undo()) {
      return false;
    }
    final note = controller.noteText;
    final end = last.start + last.removed.length;
    if (end > note.length || note.substring(last.start, end) != last.removed) {
      // Something else was undone: put it back.
      controller.redo();
      return false;
    }
    if (inserted == last.removed) {
      // Back to what was there before: no step at all.
      return true;
    }
    replace(TextRange(start: last.start, end: end), inserted);
    _revised = _Revised(
      start: last.start,
      removed: last.removed,
      inserted: inserted,
      version: controller.version,
    );
    return true;
  }

  bool _fits(TextRange range) =>
      range.isValid &&
      range.start >= 0 &&
      range.end >= range.start &&
      range.end <= controller.noteLength;

  @override
  void insertAtSelection(String text) {
    final selection = controller.noteSelection;
    if (selection.isValid &&
        selection.start >= 0 &&
        selection.end <= controller.noteLength) {
      replace(TextRange(start: selection.start, end: selection.end), text);
      return;
    }
    replace(TextRange.collapsed(controller.noteLength), text);
  }

  /// Stops future writes and makes any open suggestion stale.
  @override
  void dispose() {
    if (!_active) {
      return;
    }
    controller.removeListener(changed);
    _active = false;
    _revision++;
    notifyListeners();
    super.dispose();
  }
}

/// A revision put in at [start], replacing [removed] with [inserted], which
/// left the document at [version].
class _Revised {
  const _Revised({
    required this.start,
    required this.removed,
    required this.inserted,
    required this.version,
  });

  final int start;
  final String removed;
  final String inserted;
  final int version;
}
