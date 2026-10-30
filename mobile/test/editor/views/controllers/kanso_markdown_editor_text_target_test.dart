// Kanso — kanso_markdown_editor_text_target_test.dart
// Created 2026-10-29.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/editor_extras/models/data/markdown_editor_text_target.dart';

import '../../../helpers/editor_harness.dart';

void main() {
  testWidgets('real editor revisions join into one undo step', (tester) async {
    final harness = await pumpEditor(tester, text: '');
    final target = MarkdownEditorTextTarget(
      controller: harness.controller,
      noteId: harness.noteId,
    );
    addTearDown(target.dispose);

    target.revise(
      const TextRange(start: 0, end: 0),
      'Hel',
      joining: true,
    );
    await harness.settle(tester);
    expect(harness.controller.noteText, 'Hel');

    target.revise(
      const TextRange(start: 0, end: 3),
      'Hello',
      joining: true,
    );
    await harness.settle(tester);
    expect(harness.controller.noteText, 'Hello');
    expect(harness.controller.history.canUndo, isTrue);

    expect(harness.controller.undo(), isTrue);
    await harness.settle(tester);
    expect(
      harness.controller.noteText,
      isEmpty,
      reason: 'the whole live phrase is one undo step',
    );
  });

  testWidgets('a committed phrase and the next phrase are separate steps', (
    tester,
  ) async {
    final harness = await pumpEditor(tester, text: '');
    final target = MarkdownEditorTextTarget(
      controller: harness.controller,
      noteId: harness.noteId,
    );
    addTearDown(target.dispose);

    target.revise(
      const TextRange(start: 0, end: 0),
      'First',
      joining: true,
    );
    await harness.settle(tester);
    target
      ..commitRevision()
      ..revise(
        const TextRange(start: 5, end: 5),
        ' second',
        joining: true,
      );
    await harness.settle(tester);
    expect(harness.controller.noteText, 'First second');

    expect(harness.controller.undo(), isTrue);
    await harness.settle(tester);
    expect(harness.controller.noteText, 'First');
  });
}
