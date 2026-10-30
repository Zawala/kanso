// Kanso — native_preview_test.dart
// Created 2026-10-30.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:dartz/dartz.dart';
import 'package:flutter/gestures.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_svg/flutter_svg.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/types.dart' hide Table, TableRow;
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/core/models/data/theme_selection.dart';
import 'package:kanso/core/state/appearance/appearance_bloc.dart';
import 'package:kanso/injection.dart';
import 'package:kanso/note/state/preview/preview_bloc.dart';
import 'package:kanso/note/views/widgets/note_preview.dart';
import 'package:kanso/note/views/widgets/preview/native_preview.dart';
import 'package:kanso/note/views/widgets/preview/preview_list.dart';
import 'package:mocktail/mocktail.dart';

import '../../../../helpers/kanso_test_app.dart';
import '../../../../helpers/library_fixtures.dart';
import '../../../../helpers/mock_repositories.dart';
import '../../../../helpers/seeded_appearance_bloc.dart';
import '../../../../helpers/theme_tokens.dart';

/// A preview palette nothing else uses, so a colour found in the preview
/// can only have come from it.
const PaletteDto _palette = PaletteDto(
  dark: false,
  background: 0xFAFAFA,
  text: 0x111111,
  muted: 0x777777,
  accent: 0x1E66F5,
  border: 0xCCCCCC,
  surface: 0xEEEEEE,
  heading: 0x8839EF,
  quote: 0x40A02B,
  taskDone: 0x999999,
  inlineCodeBackground: 0xE0E0E0,
);

Color _rgb(int value) => Color(0xFF000000 | value);

PreviewInline _text(
  String text, {
  bool bold = false,
  bool italic = false,
  bool strike = false,
  bool code = false,
}) => PreviewInline.text(
  text: text,
  bold: bold,
  italic: italic,
  strike: strike,
  code: code,
);

PreviewBlock _paragraph(List<PreviewInline> inlines) =>
    PreviewBlock.paragraph(inlines: inlines);

const String _svg =
    '<svg xmlns="http://www.w3.org/2000/svg" width="40" height="20" '
    'viewBox="0 0 40 20"><rect width="40" height="20" fill="#000"/></svg>';

/// The [TextSpan] whose text is [text] in the rich texts on screen.
TextSpan _span(WidgetTester tester, String text) {
  for (final rich in tester.widgetList<RichText>(find.byType(RichText))) {
    TextSpan? found;
    rich.text.visitChildren((span) {
      if (span is TextSpan && span.text == text) {
        found = span;
        return false;
      }
      return true;
    });
    if (found != null) {
      return found!;
    }
  }
  throw StateError('No span "$text"');
}

/// The style the span [text] is drawn in: its own over its parents'.
TextStyle _styleOf(WidgetTester tester, String text) {
  for (final rich in tester.widgetList<RichText>(find.byType(RichText))) {
    TextStyle? found;
    void walk(InlineSpan span, TextStyle inherited) {
      final style = inherited.merge(span.style);
      if (span is TextSpan) {
        if (span.text == text) {
          found = style;
        }
        for (final child in span.children ?? const <InlineSpan>[]) {
          walk(child, style);
        }
      }
    }

    walk(rich.text, const TextStyle());
    if (found != null) {
      return found!;
    }
  }
  throw StateError('No span "$text"');
}

void main() {
  late List<String> tapped;
  late int edits;

  setUp(() {
    tapped = <String>[];
    edits = 0;
  });

  Future<GlobalKey<NativePreviewState>> pump(
    WidgetTester tester,
    List<PreviewBlock> blocks, {
    double below = 0,
  }) async {
    final key = GlobalKey<NativePreviewState>();
    await tester.pumpWidget(
      KansoTestApp(
        home: SingleChildScrollView(
          child: Column(
            children: <Widget>[
              NativePreview(
                key: key,
                blocks: blocks,
                palette: _palette,
                onLink: tapped.add,
                onEdit: () => edits++,
              ),
              SizedBox(height: below),
            ],
          ),
        ),
      ),
    );
    await tester.pump();
    return key;
  }

  group('NativePreview', () {
    testWidgets('a paragraph keeps its bold, italic, struck and code runs', (
      tester,
    ) async {
      await pump(tester, <PreviewBlock>[
        _paragraph(<PreviewInline>[
          _text('plain '),
          _text('strong', bold: true),
          _text('slanted', italic: true),
          _text('gone', strike: true),
          _text('x = 1', code: true),
        ]),
      ]);
      expect(_styleOf(tester, 'strong').fontWeight, FontWeight.w700);
      expect(_styleOf(tester, 'slanted').fontStyle, FontStyle.italic);
      expect(
        _styleOf(tester, 'gone').decoration,
        TextDecoration.lineThrough,
      );
      expect(
        _styleOf(tester, 'x = 1').backgroundColor,
        _rgb(_palette.inlineCodeBackground),
      );
      expect(_styleOf(tester, 'plain ').color, _rgb(_palette.text));
    });

    testWidgets('a tap on a link hands its address over', (tester) async {
      await pump(tester, <PreviewBlock>[
        _paragraph(<PreviewInline>[
          _text('See '),
          PreviewInline.link(
            url: 'kanso-tag:work',
            inlines: <PreviewInline>[_text('work')],
          ),
        ]),
      ]);
      final recognizer = _span(tester, 'work').recognizer;
      expect(recognizer, isA<TapGestureRecognizer>());
      await tester.tapOnText(find.textRange.ofSubstring('work'));
      expect(tapped, <String>['kanso-tag:work']);
    });

    testWidgets(
      'headings, links and quotes are drawn in the preview palette',
      (tester) async {
        await pump(tester, <PreviewBlock>[
          PreviewBlock.heading(
            level: 2,
            id: 'plan',
            inlines: <PreviewInline>[_text('Plan')],
          ),
          _paragraph(<PreviewInline>[
            PreviewInline.link(
              url: 'https://example.com',
              inlines: <PreviewInline>[_text('a link')],
            ),
          ]),
          PreviewBlock.quote(
            blocks: <PreviewBlock>[
              _paragraph(<PreviewInline>[_text('quoted')]),
            ],
          ),
        ]);
        expect(_styleOf(tester, 'Plan').color, _rgb(_palette.heading!));
        expect(_styleOf(tester, 'a link').color, _rgb(_palette.accent));
        expect(_styleOf(tester, 'quoted').color, _rgb(_palette.quote));
      },
    );

    testWidgets('a task box asks to tick its task; one without a place '
        "can't be ticked", (tester) async {
      await pump(tester, <PreviewBlock>[
        PreviewBlock.list(
          ordered: false,
          start: 1,
          tight: true,
          items: <PreviewListItem>[
            PreviewListItem(
              task: const PreviewTaskBox(checked: false, index: 4),
              blocks: <PreviewBlock>[
                _paragraph(<PreviewInline>[_text('Sweep')]),
              ],
            ),
            PreviewListItem(
              task: const PreviewTaskBox(checked: true),
              blocks: <PreviewBlock>[
                _paragraph(<PreviewInline>[_text('Dust')]),
              ],
            ),
          ],
        ),
      ]);
      final boxes = find.byType(PreviewTaskCheckbox);
      expect(boxes, findsNWidgets(2));
      await tester.tap(boxes.first);
      await tester.tap(boxes.last);
      expect(tapped, <String>['kanso-task:4']);
      // A ticked task's text is in the done colour.
      expect(_styleOf(tester, 'Dust').color, _rgb(_palette.taskDone));
    });

    testWidgets('a code block shows its language and copies by its number', (
      tester,
    ) async {
      await pump(tester, <PreviewBlock>[
        const PreviewBlock.code(
          language: 'rust',
          runs: <PreviewCodeRun>[
            PreviewCodeRun(
              text: 'fn',
              color: 0xD20F39,
              bold: true,
              italic: false,
            ),
            PreviewCodeRun(text: ' main() {}', bold: false, italic: false),
          ],
          copy: 3,
        ),
      ]);
      expect(find.text('rust'), findsOneWidget);
      expect(_styleOf(tester, 'fn').color, _rgb(0xD20F39));
      final semantics = tester.ensureSemantics();
      await tester.tap(find.bySemanticsLabel('Copy code'));
      semantics.dispose();
      expect(tapped, <String>['kanso-copy:3']);
    });

    testWidgets('a heading can be scrolled to by its id', (tester) async {
      final key = await pump(tester, <PreviewBlock>[
        for (var at = 0; at < 40; at++)
          _paragraph(<PreviewInline>[_text('Line $at')]),
        PreviewBlock.heading(
          level: 1,
          id: 'end',
          inlines: <PreviewInline>[_text('The end')],
        ),
      ], below: 2000);
      final heading = key.currentState!.headingKey('end');
      expect(heading?.currentContext, isNotNull);
      final before = tester.getTopLeft(find.byKey(heading!)).dy;
      unawaited(key.currentState!.scrollTo('end'));
      await tester.pumpAndSettle();
      final after = tester.getTopLeft(find.byKey(heading)).dy;
      expect(after, lessThan(before));
      expect(after, closeTo(0, 1));
    });

    testWidgets('a figure is its SVG', (tester) async {
      await pump(tester, <PreviewBlock>[
        const PreviewBlock.figure(
          kind: PreviewFigureKind.diagram,
          svg: PreviewSvg(markup: _svg, width: 40, height: 20),
        ),
      ]);
      expect(find.byType(SvgPicture), findsOneWidget);
      expect(tester.getSize(find.byType(SvgPicture)), const Size(40, 20));
    });

    testWidgets('a table shows its header and cells', (tester) async {
      await pump(tester, <PreviewBlock>[
        PreviewBlock.table(
          aligns: const <PreviewAlign>[PreviewAlign.left, PreviewAlign.right],
          head: <PreviewCell>[
            PreviewCell(inlines: <PreviewInline>[_text('Fruit')]),
            PreviewCell(inlines: <PreviewInline>[_text('Count')]),
          ],
          rows: <PreviewRow>[
            PreviewRow(
              cells: <PreviewCell>[
                PreviewCell(inlines: <PreviewInline>[_text('Apple')]),
                PreviewCell(inlines: <PreviewInline>[_text('3')]),
              ],
            ),
          ],
        ),
      ]);
      expect(find.byType(Table), findsOneWidget);
      for (final text in <String>['Fruit', 'Count', 'Apple', '3']) {
        expect(find.text(text, findRichText: true), findsOneWidget);
      }
      expect(_styleOf(tester, 'Fruit').fontWeight, FontWeight.w600);
    });

    testWidgets("a query view's title opens its note", (tester) async {
      await pump(tester, <PreviewBlock>[
        const PreviewBlock.query(
          columns: <String>['Title', 'Status'],
          rows: <PreviewQueryRow>[
            PreviewQueryRow(
              url: 'inkdrop://note/a',
              title: 'Alpha',
              cells: <String>['active'],
            ),
          ],
        ),
        const PreviewBlock.query(
          columns: <String>['Title'],
          rows: <PreviewQueryRow>[],
        ),
      ]);
      expect(find.text('Status'), findsOneWidget);
      expect(find.text('active'), findsOneWidget);
      expect(find.text('No notes match.'), findsOneWidget);
      await tester.tap(find.text('Alpha'));
      expect(tapped, <String>['inkdrop://note/a']);
    });

    testWidgets('a source that failed shows its badge and its code', (
      tester,
    ) async {
      await pump(tester, <PreviewBlock>[
        const PreviewBlock.failed(
          badge: 'Unsupported diagram',
          detail: 'Kanso does not draw gantt.',
          language: 'gantt',
          code: 'gantt\n  title Plan',
          copy: 1,
        ),
        const PreviewBlock.queryInvalid(
          code: 'where nonsense',
          message: 'Unknown field.',
        ),
      ]);
      expect(find.text('Unsupported diagram'), findsOneWidget);
      expect(find.text('gantt\n  title Plan'), findsOneWidget);
      expect(find.text('gantt'), findsOneWidget);
      expect(find.text('Query'), findsOneWidget);
      expect(find.text('Unknown field.'), findsOneWidget);
      final semantics = tester.ensureSemantics();
      await tester.tap(find.bySemanticsLabel('Copy code'));
      semantics.dispose();
      expect(tapped, <String>['kanso-copy:1']);
    });

    testWidgets('a double tap asks to edit', (tester) async {
      await pump(tester, <PreviewBlock>[
        _paragraph(<PreviewInline>[_text('Some text to read')]),
      ]);
      final at = tester.getCenter(find.byType(NativePreview));
      await tester.tapAt(at);
      await tester.pump(const Duration(milliseconds: 50));
      await tester.tapAt(at);
      await tester.pumpAndSettle();
      expect(edits, 1);
    });
  });

  group('NotePreview', () {
    late MockLibraryRepository library;
    late MockRendererRepository renderer;
    late MockThemesRepository themes;

    setUpAll(() {
      registerFallbackValue(_palette);
      registerFallbackValue(
        const PreviewOptions(size: 16, utcOffsetSecs: 0, code: []),
      );
    });

    setUp(() {
      library = MockLibraryRepository();
      renderer = MockRendererRepository();
      themes = MockThemesRepository();
      when(
        () => library.changes(),
      ).thenAnswer((_) => const Stream<List<String>>.empty());
      when(() => library.note(any())).thenAnswer(
        (_) async => Right<ApplicationError, NoteView>(mockOpenNote),
      );
      when(() => themes.fit(any(), any(), any())).thenAnswer(
        (_) async => Right<ApplicationError, ThemeTokens>(
          tokensFor(fakePalette('paper')),
        ),
      );
      when(() => renderer.previewBlocks(any(), any(), any())).thenAnswer(
        (_) async => Right<ApplicationError, List<PreviewBlock>>(
          <PreviewBlock>[
            _paragraph(<PreviewInline>[_text('Drawn natively')]),
          ],
        ),
      );
      getIt.registerFactory<PreviewBloc>(
        () => PreviewBloc(
          library,
          renderer,
          themes,
          MockWebLinkRepository(),
          MockUndoNoticeBloc(),
          MockHapticsRepository(),
        ),
      );
      addTearDown(() => getIt.unregister<PreviewBloc>());
    });

    testWidgets(
      'draws the note in the chosen preview theme, fitted with the rest',
      (tester) async {
        final appearance = SeededAppearanceBloc(
          const AppearanceState(
            selection: ThemeSelection(
              followSystem: false,
              syntax: 'gallery-dracula',
              preview: 'gallery-paper',
            ),
          ),
        );
        addTearDown(appearance.close);
        await tester.pumpWidget(
          KansoTestApp(
            brightness: Brightness.dark,
            home: BlocProvider<AppearanceBloc>.value(
              value: appearance,
              child: CustomScrollView(
                slivers: <Widget>[
                  NotePreview(note: mockOpenNote, lines: const []),
                ],
              ),
            ),
          ),
        );
        await tester.pumpAndSettle();
        verify(
          () => themes.fit('kanso-dark', 'gallery-dracula', 'gallery-paper'),
        ).called(1);
        expect(find.byType(NativePreview), findsOneWidget);
        expect(
          find.text('Drawn natively', findRichText: true),
          findsOneWidget,
        );
        verifyNever(() => renderer.builtinThemeTokens(any()));
      },
    );
  });
}
