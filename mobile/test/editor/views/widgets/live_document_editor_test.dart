// Kanso — live_document_editor_test.dart
// Created 2026-10-30.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart' show Right;
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/editor.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/configs/kanso_palette.dart';
import 'package:kanso/editor/models/repos/impl/rust_editor_repository.dart';
import 'package:kanso/editor/utils/live_document/live_layout_bridge.dart';
import 'package:kanso/editor/utils/live_document/live_rows.dart';
import 'package:kanso/editor/views/controllers/kanso_markdown_controller.dart';
import 'package:kanso/editor/views/controllers/live_layout_controller.dart';
import 'package:kanso/editor/views/widgets/live_document_editor.dart';
import 'package:kanso/note/views/widgets/preview/preview_code.dart';
import 'package:kanso/note/views/widgets/preview/preview_list.dart';

import '../../../helpers/core_library.dart';
import '../../../helpers/kanso_test_app.dart';

/// The Live editor on a note in the real core.
class LiveHarness {
  LiveHarness({
    required this.editor,
    required this.controller,
    required this.layout,
    required this.focus,
    required this.key,
  });

  final KansoEditor editor;
  final KansoMarkdownController controller;
  final LiveLayoutController layout;
  final FocusNode focus;
  final GlobalKey<LiveDocumentEditorState> key;

  LiveDocumentEditorState get state => key.currentState!;

  /// The text Rust has.
  Future<String> rustText(WidgetTester tester) async =>
      (await tester.runAsync(editor.snapshot))!.text;

  /// Lets the core's layout and the preview's blocks come in.
  Future<void> settle(WidgetTester tester) async {
    for (var i = 0; i < 8; i++) {
      await tester.runAsync(
        () => Future<void>.delayed(const Duration(milliseconds: 30)),
      );
      await tester.pump();
    }
  }

  /// What the platform's input method was last given.
  TextEditingValue get imeValue => state.currentTextEditingValue!;

  /// The input method changes its value to [value].
  Future<void> type(WidgetTester tester, TextEditingValue value) async {
    tester.testTextInput.updateEditingValue(value);
    await tester.pump();
  }

  /// The input method types [text] at its caret.
  Future<void> typeText(WidgetTester tester, String text) async {
    final value = imeValue;
    final at = value.selection.extentOffset;
    await type(
      tester,
      TextEditingValue(
        text: value.text.replaceRange(at, at, text),
        selection: TextSelection.collapsed(offset: at + text.length),
      ),
    );
  }
}

Future<LiveHarness> pumpLive(
  WidgetTester tester, {
  required String text,
  int? caret,
  bool focus = true,
}) async {
  tester.view
    ..physicalSize = const Size(393, 800)
    ..devicePixelRatio = 1;
  addTearDown(tester.view.reset);
  final palette = KansoPalette.kansoLight.preview;
  final made = (await tester.runAsync(() async {
    await initCoreForTests();
    final library = await openTestLibrary();
    final saved = await library.create(
      note: NewNote(title: 'Live', body: text),
    );
    final editor = await library.edit(id: saved.id);
    final snapshot = await editor.snapshot();
    return (saved.id, editor, snapshot, library.renderer());
  }))!;
  final (noteId, editor, snapshot, renderer) = made;
  final controller = KansoMarkdownController(
    document: RustEditorDocument(editor),
    snapshot: snapshot,
    themeId: 'kanso-light',
    caret: caret ?? snapshot.text.length,
    unwindowed: true,
  );
  final layout = LiveLayoutController(
    controller: controller,
    previewDelay: Duration.zero,
    previewSource: (text) async {
      final sourced = await renderer.previewBlocksForText(
        id: noteId,
        text: text,
        palette: palette,
        options: const PreviewOptions(
          size: 18,
          utcOffsetSecs: 0,
          code: <SyntaxStyleRow>[],
        ),
      );
      return Right(livePreviewBlocksOf(sourced, text));
    },
  );
  final node = FocusNode();
  final key = GlobalKey<LiveDocumentEditorState>();
  addTearDown(() {
    layout.dispose();
    controller.dispose();
    node.dispose();
  });
  await tester.pumpWidget(
    KansoTestApp(
      home: CustomScrollView(
        slivers: <Widget>[
          SliverPadding(
            padding: const EdgeInsets.symmetric(horizontal: 20),
            sliver: LiveDocumentEditor(
              key: key,
              controller: controller,
              layout: layout,
              focusNode: node,
              palette: palette,
            ),
          ),
        ],
      ),
    ),
  );
  final harness = LiveHarness(
    editor: editor,
    controller: controller,
    layout: layout,
    focus: node,
    key: key,
  );
  if (focus) {
    node.requestFocus();
    await tester.pump();
  }
  await harness.settle(tester);
  return harness;
}

void main() {
  testWidgets('the keyboard is given the caret block and types into it', (
    tester,
  ) async {
    final live = await pumpLive(tester, text: 'Hello\n\nWorld');
    // The caret's line, and the line before it.
    expect(live.imeValue.text, '\nWorld');
    expect(live.imeValue.selection, const TextSelection.collapsed(offset: 6));
    await live.typeText(tester, '!');
    expect(live.controller.text, 'Hello\n\nWorld!');
    expect(await live.rustText(tester), 'Hello\n\nWorld!');
    expect(live.controller.selection.extentOffset, 13);
  });

  testWidgets('a composition is kept and drawn, then committed', (
    tester,
  ) async {
    final live = await pumpLive(tester, text: 'Hello');
    await live.type(
      tester,
      const TextEditingValue(
        text: 'Hello wo',
        selection: TextSelection.collapsed(offset: 8),
        composing: TextRange(start: 6, end: 8),
      ),
    );
    expect(live.controller.isComposing, isTrue);
    expect(live.controller.value.composing, const TextRange(start: 6, end: 8));
    // The composed text is underlined.
    final spans = <TextSpan>[];
    for (final rich in tester.widgetList<RichText>(find.byType(RichText))) {
      rich.text.visitChildren((span) {
        if (span is TextSpan && span.text == 'wo') {
          spans.add(span);
        }
        return true;
      });
    }
    expect(spans, isNotEmpty);
    expect(
      spans.first.style?.decoration?.contains(TextDecoration.underline),
      isTrue,
    );
    await live.type(
      tester,
      const TextEditingValue(
        text: 'Hello world',
        selection: TextSelection.collapsed(offset: 11),
      ),
    );
    expect(live.controller.isComposing, isFalse);
    expect(await live.rustText(tester), 'Hello world');
  });

  testWidgets('marks are hidden off the caret and shown on it', (
    tester,
  ) async {
    final live = await pumpLive(
      tester,
      text: 'Some **bold** text\n\nOther',
    );
    String shown() => tester
        .widgetList<RichText>(find.byType(RichText))
        .map((rich) => rich.text.toPlainText())
        .join('|');
    expect(shown(), contains('Some bold text'));
    // The caret into the bold word: its marks show.
    live.controller.selection = const TextSelection.collapsed(offset: 8);
    await live.settle(tester);
    expect(shown(), contains('Some **bold** text'));
  });

  testWidgets('a tap puts the caret where the text was tapped', (
    tester,
  ) async {
    final live = await pumpLive(tester, text: '**Bold** start\n\nEnd');
    final target = find.textContaining('Bold start', findRichText: true);
    expect(target, findsOneWidget);
    final box = tester.getRect(target);
    // The start of the drawn text, past its hidden marks.
    await tester.tapAt(Offset(box.left + 1, box.center.dy));
    await tester.pump(const Duration(milliseconds: 400));
    expect(live.controller.selection.isCollapsed, isTrue);
    expect(live.controller.selection.extentOffset, 2);
  });

  testWidgets('a selection across blocks is drawn in each and deleted', (
    tester,
  ) async {
    final live = await pumpLive(tester, text: 'First block\n\nSecond block');
    live.controller.selection = const TextSelection(
      baseOffset: 6,
      extentOffset: 20,
    );
    await live.settle(tester);
    final rows = live.state.rowStates.values
        .where((row) => row.row.kind == LiveRowKind.text)
        .toList();
    final selected = rows.where((row) => row.boxesFor(6, 20).isNotEmpty).length;
    expect(selected, 2);
    await tester.sendKeyEvent(LogicalKeyboardKey.backspace);
    await tester.pump();
    expect(live.controller.text, 'First block');
    expect(await live.rustText(tester), 'First block');
  });

  testWidgets('Return splits a block, Backspace at its start joins it', (
    tester,
  ) async {
    final live = await pumpLive(tester, text: 'One\n\nTwo', caret: 3);
    await live.typeText(tester, '\n');
    await live.settle(tester);
    expect(live.controller.text, 'One\n\n\nTwo');
    expect(live.controller.selection.extentOffset, 4);
    // At the start of "Two", Backspace takes the line break before it.
    live.controller.selection = const TextSelection.collapsed(offset: 6);
    await live.settle(tester);
    await tester.sendKeyEvent(LogicalKeyboardKey.backspace);
    await tester.pump();
    expect(live.controller.text, 'One\n\nTwo');
    // And the keyboard's own Backspace deletes the line break it was given
    // before the line.
    final value = live.imeValue;
    final at = value.selection.extentOffset;
    expect(value.text.substring(at - 1, at), '\n');
    await live.type(
      tester,
      TextEditingValue(
        text: value.text.replaceRange(at - 1, at, ''),
        selection: TextSelection.collapsed(offset: at - 1),
      ),
    );
    expect(live.controller.text, 'One\nTwo');
    expect(await live.rustText(tester), 'One\nTwo');
  });

  testWidgets("a task's box ticks it and leaves the caret", (tester) async {
    final live = await pumpLive(tester, text: '- [ ] Buy milk\n\nEnd');
    live.controller.selection = const TextSelection.collapsed(offset: 18);
    await live.settle(tester);
    expect(find.byType(PreviewTaskCheckbox), findsOneWidget);
    await tester.tap(find.byType(PreviewTaskCheckbox));
    await tester.pump();
    expect(live.controller.text, '- [x] Buy milk\n\nEnd');
    expect(live.controller.selection.extentOffset, 18);
    expect(await live.rustText(tester), '- [x] Buy milk\n\nEnd');
  });

  testWidgets('a tap on a drawn block shows its source', (tester) async {
    final live = await pumpLive(
      tester,
      text: 'Intro\n\n```js\nconst a = 1;\n```\n\nEnd',
    );
    expect(find.byType(PreviewCode), findsOneWidget);
    await tester.tapAt(tester.getCenter(find.byType(PreviewCode)));
    await tester.pump(const Duration(milliseconds: 400));
    await live.settle(tester);
    expect(live.controller.selection.extentOffset, 7);
    expect(find.byType(PreviewCode), findsNothing);
    expect(find.textContaining('```js', findRichText: true), findsOneWidget);
  });

  testWidgets('undo takes back what was typed', (tester) async {
    final live = await pumpLive(tester, text: 'Hello');
    await live.typeText(tester, ' there');
    expect(live.controller.text, 'Hello there');
    await tester.sendKeyDownEvent(LogicalKeyboardKey.controlLeft);
    await tester.sendKeyEvent(LogicalKeyboardKey.keyZ);
    await tester.sendKeyUpEvent(LogicalKeyboardKey.controlLeft);
    await tester.pump();
    expect(live.controller.text, 'Hello');
    expect(await live.rustText(tester), 'Hello');
  });

  testWidgets('arrows move across blocks, by line and by character', (
    tester,
  ) async {
    final live = await pumpLive(tester, text: 'Top line\n\nBottom', caret: 2);
    await tester.sendKeyEvent(LogicalKeyboardKey.arrowDown);
    await tester.pump();
    await live.settle(tester);
    expect(live.controller.selection.extentOffset, greaterThanOrEqualTo(10));
    await tester.sendKeyEvent(LogicalKeyboardKey.arrowUp);
    await tester.pump();
    await live.settle(tester);
    expect(live.controller.selection.extentOffset, lessThan(9));
    final at = live.controller.selection.extentOffset;
    await tester.sendKeyDownEvent(LogicalKeyboardKey.shiftLeft);
    await tester.sendKeyEvent(LogicalKeyboardKey.arrowRight);
    await tester.sendKeyUpEvent(LogicalKeyboardKey.shiftLeft);
    await tester.pump();
    expect(
      live.controller.selection,
      TextSelection(baseOffset: at, extentOffset: at + 1),
    );
  });

  testWidgets('a long press selects a word and shows the menu', (
    tester,
  ) async {
    final live = await pumpLive(tester, text: 'Pick a word here');
    final target = find.textContaining('Pick a word', findRichText: true);
    final box = tester.getRect(target);
    await tester.longPressAt(Offset(box.left + 4, box.center.dy));
    await tester.pump();
    expect(
      live.controller.selection,
      const TextSelection(baseOffset: 0, extentOffset: 4),
    );
    expect(live.state.menuShown, isTrue);
    expect(find.text('Copy'), findsOneWidget);
  });

  testWidgets('copy, cut and paste go through the clipboard', (tester) async {
    String? clipboard;
    tester.binding.defaultBinaryMessenger.setMockMethodCallHandler(
      SystemChannels.platform,
      (call) async {
        switch (call.method) {
          case 'Clipboard.setData':
            clipboard =
                (call.arguments as Map<Object?, Object?>)['text'] as String?;
            return null;
          case 'Clipboard.getData':
            return <String, Object?>{'text': clipboard};
          case 'Clipboard.hasStrings':
            return <String, Object?>{'value': clipboard != null};
        }
        return null;
      },
    );
    addTearDown(
      () => tester.binding.defaultBinaryMessenger.setMockMethodCallHandler(
        SystemChannels.platform,
        null,
      ),
    );
    final live = await pumpLive(tester, text: 'First block\n\nSecond block');
    // Across the blocks.
    live.controller.selection = const TextSelection(
      baseOffset: 6,
      extentOffset: 19,
    );
    await tester.pump();
    live.state.copySelection(SelectionChangedCause.keyboard);
    await tester.pump();
    expect(clipboard, 'block\n\nSecond');
    live.state.cutSelection(SelectionChangedCause.keyboard);
    await tester.pump();
    expect(live.controller.text, 'First  block');
    expect(live.controller.selection, const TextSelection.collapsed(offset: 6));
    await live.state.pasteText(SelectionChangedCause.keyboard);
    await tester.pump();
    expect(live.controller.text, 'First block\n\nSecond block');
    expect(await live.rustText(tester), 'First block\n\nSecond block');
  });

  testWidgets('a long note lays out only the blocks on screen, and the '
      'caret is scrolled to', (tester) async {
    final text = List<String>.generate(
      1500,
      (i) => 'Paragraph number $i of a long note.',
    ).join('\n\n');
    final live = await pumpLive(tester, text: text, caret: 0);
    final rows = live.state.rows;
    expect(rows.length, greaterThan(1500));
    expect(live.state.rowStates.length, lessThan(100));
    // The caret to the end: its block is laid out and on the screen.
    live.controller.selection = TextSelection.collapsed(offset: text.length);
    for (var i = 0; i < 12; i++) {
      await tester.pump(const Duration(milliseconds: 120));
    }
    final caret = live.state.caretOnScreen();
    expect(caret, isNotNull);
    // On the screen (the note's last line, at the foot of the scroll).
    expect(caret!.top, greaterThanOrEqualTo(0));
    expect(caret.top, lessThan(800));
    expect(live.state.rowStates.length, lessThan(100));
  });
}
