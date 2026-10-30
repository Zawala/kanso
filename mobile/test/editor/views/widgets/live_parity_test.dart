// Kanso — live_parity_test.dart
// Created 2026-10-30.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:io';
import 'dart:ui' as ui;

import 'package:dartz/dartz.dart' show Right;
import 'package:flutter/material.dart';
import 'package:flutter/rendering.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/configs/kanso_palette.dart';
import 'package:kanso/editor/models/repos/impl/rust_editor_repository.dart';
import 'package:kanso/editor/utils/live_document/live_layout_bridge.dart';
import 'package:kanso/editor/views/controllers/kanso_markdown_controller.dart';
import 'package:kanso/editor/views/controllers/live_layout_controller.dart';
import 'package:kanso/editor/views/widgets/live_document_editor.dart';
import 'package:kanso/note/views/widgets/preview/native_preview.dart';

import '../../../helpers/core_library.dart';
import '../../../helpers/kanso_test_app.dart';

/// A note with every kind of block Live draws.
const String richNote = r'''
---
title: Rich
---
# Heading one

Paragraph with **bold**, *italic*, ~~struck~~, `code`, a [link](https://kanso.app), a [[Missing note]] and inline math $x^2$ in it.

## Heading two

### Heading three

#### Heading four

##### Heading five

###### Heading six

- First bullet
  - Nested bullet
    - Deeper bullet
- Second bullet

1. One
2. Two
3. Three

- [ ] Open task @2026-11-02
- [x] Done task

> Quote paragraph one.
>
> Quote paragraph two.

---

```js
const answer = 42;
```

| Name | Value |
|------|------:|
| One  |     1 |

$$
E = mc^2
$$

```mermaid
graph TD; A-->B;
```

![An image](https://example.com/image.png)

Last paragraph.
''';

/// Where the pictures of the comparison are written: `KANSO_SHOTS` when
/// set, else nowhere.
String? get shotsFolder => Platform.environment['KANSO_SHOTS'];

/// The text of every laid-out paragraph under [root], with where it is.
List<(String, Rect)> paragraphsIn(RenderObject root) {
  final found = <(String, Rect)>[];
  void visit(RenderObject object) {
    if (object is RenderParagraph && object.hasSize) {
      final text = object.text.toPlainText().replaceAll('￼', '').trim();
      if (text.isNotEmpty) {
        final at = object.localToGlobal(Offset.zero);
        found.add((text, at & object.size));
      }
    }
    object.visitChildren(visit);
  }

  visit(root);
  return found;
}

Future<void> settle(WidgetTester tester, {int rounds = 12}) async {
  for (var i = 0; i < rounds; i++) {
    await tester.runAsync(
      () => Future<void>.delayed(const Duration(milliseconds: 60)),
    );
    await tester.pump();
  }
}

Future<ui.Image> capture(WidgetTester tester, GlobalKey key) async {
  final boundary =
      key.currentContext!.findRenderObject()! as RenderRepaintBoundary;
  return (await tester.runAsync(boundary.toImage))!;
}

Future<void> save(WidgetTester tester, ui.Image image, String name) async {
  final folder = shotsFolder;
  if (folder == null) {
    return;
  }
  await tester.runAsync(() async {
    final bytes = await image.toByteData(format: ui.ImageByteFormat.png);
    await Directory(folder).create(recursive: true);
    await File('$folder/$name.png').writeAsBytes(bytes!.buffer.asUint8List());
  });
}

void main() {
  for (final (width, brightness) in <(double, Brightness)>[
    (393, Brightness.light),
    (320, Brightness.dark),
  ]) {
    parity(width, brightness);
  }
}

void parity(double width, Brightness brightness) {
  const height = 2600.0;
  const gutter = 20.0;
  final dark = brightness == Brightness.dark;
  final palette =
      (dark ? KansoPalette.kansoDark : KansoPalette.kansoLight).preview;
  final name = '${width.round()}_${brightness.name}';

  testWidgets('Live draws a note as the preview does, off the caret, '
      '${width.round()} wide, ${brightness.name}', (tester) async {
    tester.view
      ..physicalSize = Size(width, height)
      ..devicePixelRatio = 1;
    addTearDown(tester.view.reset);
    final made = (await tester.runAsync(() async {
      await initCoreForTests();
      final library = await openTestLibrary();
      final saved = await library.create(
        note: const NewNote(title: 'Rich', body: richNote),
      );
      final editor = await library.edit(id: saved.id);
      final snapshot = await editor.snapshot();
      final renderer = library.renderer();
      const options = PreviewOptions(
        size: 18,
        utcOffsetSecs: 0,
        code: <SyntaxStyleRow>[],
      );
      // The preview of the text the editor has: the note without its
      // front matter, which the editor shows as properties.
      final blocks = [
        for (final sourced in await renderer.previewBlocksForText(
          id: saved.id,
          text: snapshot.text,
          palette: palette,
          options: options,
        ))
          sourced.block,
      ];
      return (library, saved.id, editor, snapshot, renderer, blocks, options);
    }))!;
    final (_, noteId, editor, snapshot, renderer, blocks, options) = made;

    // The preview.
    final previewKey = GlobalKey();
    await tester.pumpWidget(
      KansoTestApp(
        brightness: brightness,
        home: RepaintBoundary(
          key: previewKey,
          child: ColoredBox(
            color: Color(0xFF000000 | palette.background),
            child: CustomScrollView(
              slivers: <Widget>[
                SliverPadding(
                  padding: const EdgeInsets.symmetric(horizontal: gutter),
                  sliver: NativePreview(
                    blocks: blocks,
                    palette: palette,
                    onLink: (_) {},
                  ),
                ),
              ],
            ),
          ),
        ),
      ),
    );
    await settle(tester);
    final previewParagraphs = paragraphsIn(
      previewKey.currentContext!.findRenderObject()!,
    );
    final previewImage = await capture(tester, previewKey);
    await save(tester, previewImage, 'preview_$name');

    // Live, the caret on the blank line after the last paragraph.
    final document = RustEditorDocument(editor);
    final controller = KansoMarkdownController(
      document: document,
      snapshot: snapshot,
      themeId: 'kanso-light',
      caret: snapshot.text.length,
      unwindowed: true,
    );
    final layout = LiveLayoutController(
      controller: controller,
      previewSource: (text) async {
        final sourced = await renderer.previewBlocksForText(
          id: noteId,
          text: text,
          palette: palette,
          options: options,
        );
        return Right(livePreviewBlocksOf(sourced, text));
      },
      previewDelay: Duration.zero,
    );
    final focus = FocusNode();
    addTearDown(() {
      layout.dispose();
      controller.dispose();
      focus.dispose();
    });
    final liveKey = GlobalKey();
    final editorKey = GlobalKey<LiveDocumentEditorState>();
    await tester.pumpWidget(
      KansoTestApp(
        brightness: brightness,
        home: RepaintBoundary(
          key: liveKey,
          child: ColoredBox(
            color: Color(0xFF000000 | palette.background),
            child: CustomScrollView(
              slivers: <Widget>[
                SliverPadding(
                  padding: const EdgeInsets.symmetric(horizontal: gutter),
                  sliver: LiveDocumentEditor(
                    key: editorKey,
                    controller: controller,
                    layout: layout,
                    focusNode: focus,
                    palette: palette,
                  ),
                ),
              ],
            ),
          ),
        ),
      ),
    );
    await settle(tester);
    expect(layout.settled, isTrue);
    expect(layout.previews, isNotEmpty);
    final liveParagraphs = paragraphsIn(
      liveKey.currentContext!.findRenderObject()!,
    );
    final liveImage = await capture(tester, liveKey);
    await save(tester, liveImage, 'live_$name');

    // Every paragraph of the preview is in Live, in the same place.
    final misses = <String>[];
    var from = 0;
    for (final (text, rect) in previewParagraphs) {
      final at = liveParagraphs.indexWhere((p) => p.$1 == text, from);
      if (at < 0) {
        misses.add('missing "$text" at $rect');
        continue;
      }
      from = at + 1;
      final live = liveParagraphs[at].$2;
      if ((live.left - rect.left).abs() > 0.01 ||
          (live.top - rect.top).abs() > 0.01 ||
          (live.width - rect.width).abs() > 0.01 ||
          (live.height - rect.height).abs() > 0.01) {
        misses.add('"$text": preview $rect, live $live');
      }
    }
    if (shotsFolder != null) {
      await tester.runAsync(
        () => File(
          '$shotsFolder/rects_$name.txt',
        ).writeAsString('${misses.join('\n')}\n'),
      );
    }
    expect(misses, isEmpty, reason: misses.join('\n'));

    // And the two are the same picture, pixel for pixel.
    final pixels = (await tester.runAsync(() async {
      final a = await previewImage.toByteData();
      final b = await liveImage.toByteData();
      return (a!.buffer.asUint8List(), b!.buffer.asUint8List());
    }))!;
    var differing = 0;
    for (var at = 0; at < pixels.$1.length; at += 4) {
      if (pixels.$1[at] != pixels.$2[at] ||
          pixels.$1[at + 1] != pixels.$2[at + 1] ||
          pixels.$1[at + 2] != pixels.$2[at + 2] ||
          pixels.$1[at + 3] != pixels.$2[at + 3]) {
        differing++;
      }
    }
    expect(differing, 0, reason: '$differing pixels differ');
  });
}
