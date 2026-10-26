// Kanso — renderer_suite.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/renderer.dart';
import 'package:kanso/core/bridge/api/types.dart';

import 'open_library.dart';

/// A small theme file: a name, whether it is light or dark, and three colours;
/// the rest is the defaults'.
const String themeJson = '''
{
  "schema_version": 1,
  "name": "Suite Light",
  "appearance": "light",
  "ui": {"editor": "#FCFAF4", "text": "#26241F", "accent": "#5E7895"}
}
''';

/// Drawing, through the bridge: every call of `KansoRenderer`.
void rendererSuite(OpenLibrary open) {
  group('KansoRenderer', () {
    late PaletteDto palette;

    setUpAll(() async {
      palette = (await KansoRenderer.themeTokens(themeJson: themeJson)).palette;
    });

    test('a theme file is fitted for drawing', () async {
      final tokens = await KansoRenderer.themeTokens(themeJson: themeJson);
      expect(tokens.name, 'Suite Light');
      expect(tokens.dark, isFalse);
      expect(tokens.ui.map((color) => color.name), contains('editor'));
      expect(tokens.syntax, isNotEmpty);
      expect(tokens.palette.background, 0xFCFAF4);
      await expectLater(
        KansoRenderer.themeTokens(themeJson: 'not json'),
        throwsA(isA<KansoError_Invalid>()),
      );
    });

    test(
      'highlighting gives runs inside the range, with their tokens',
      () async {
        const text = '# Title\n\nSome `code` here.\n';
        final spans = await KansoRenderer.highlight(
          text: text,
          start: 0,
          end: text.length,
        );
        expect(spans, isNotEmpty);
        for (final span in spans) {
          expect(span.range.start, lessThan(span.range.end));
          expect(span.range.end, lessThanOrEqualTo(text.length));
          expect(span.tokens, isNotEmpty);
        }
      },
    );

    test('math is drawn as an svg', () async {
      final inline = await KansoRenderer.mathSvg(
        source: 'x^2 + y^2',
        display: false,
        palette: palette,
      );
      expect(inline.markup, contains('<svg'));
      expect(inline.width, greaterThan(0));
      expect(inline.baseline, isNotNull);
      final block = await KansoRenderer.mathSvg(
        source: r'\frac{a}{b}',
        display: true,
        palette: palette,
      );
      expect(block.markup, contains('<svg'));
    });

    test('a diagram is drawn as an svg, and a broken one is refused', () async {
      final scene = await KansoRenderer.diagramSvg(
        kind: DiagramKind.excalidraw,
        source:
            '{"type":"excalidraw","version":2,"elements":[{"id":"a",'
            '"type":"rectangle","x":0,"y":0,"width":120,"height":60,'
            '"strokeColor":"#1e1e1e","backgroundColor":"transparent",'
            '"fillStyle":"solid","strokeWidth":1,"roughness":0,"seed":1}],'
            '"appState":{}}',
        palette: palette,
      );
      expect(scene.markup, contains('<svg'));
      await expectLater(
        KansoRenderer.diagramSvg(
          kind: DiagramKind.mermaid,
          source: 'this is not a diagram ::: [[',
          palette: palette,
        ),
        throwsA(isA<KansoError_Invalid>()),
      );
    });

    test('a note is previewed as a page in the palette', () async {
      final library = await open();
      final id = await makeNote(
        library,
        'Preview',
        '# Heading\n\nA **bold** word.',
      );
      final html = await library.renderer().previewHtml(
        id: id,
        palette: palette,
      );
      expect(html, contains('<h1'));
      expect(html, contains('bold'));
    });

    test('fonts are taken once', () async {
      expect(await KansoRenderer.setFonts(fonts: const []), isA<bool>());
    });
  });
}
