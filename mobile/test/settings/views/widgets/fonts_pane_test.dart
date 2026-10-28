// Kanso — fonts_pane_test.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/configs/kanso_icons.dart';
import 'package:kanso/core/models/data/kanso_font_family.dart';
import 'package:kanso/core/state/appearance/appearance_bloc.dart';
import 'package:kanso/core/views/widgets/kanso_icon.dart';
import 'package:kanso/settings/views/widgets/continuous_text_size_control.dart';
import 'package:kanso/settings/views/widgets/font_choice_row.dart';
import 'package:kanso/settings/views/widgets/fonts_pane.dart';

import '../../../helpers/pump.dart';
import '../../../helpers/seeded_appearance_bloc.dart';

Future<AppearanceBloc> _pump(
  WidgetTester tester, {
  AppearanceBloc? bloc,
  bool settle = true,
}) async {
  final appearance = bloc ?? SeededAppearanceBloc(const AppearanceState());
  addTearDown(appearance.close);
  tester.view
    ..physicalSize = const Size(393, 3000)
    ..devicePixelRatio = 1;
  addTearDown(tester.view.reset);
  await pumpKanso(
    tester,
    BlocProvider<AppearanceBloc>.value(
      value: appearance,
      child: const SizedBox(width: 393, height: 3000, child: FontsPane()),
    ),
    settle: settle,
  );
  return appearance;
}

Finder _row(KansoFontFamily font, {int index = 0}) {
  return find
      .byWidgetPredicate(
        (widget) => widget is FontChoiceRow && widget.font == font,
      )
      .at(index);
}

void main() {
  group('FontsPane', () {
    testWidgets('lists the choices of each section, the right families', (
      tester,
    ) async {
      await _pump(tester);
      expect(find.text('Editor font'), findsOneWidget);
      expect(find.text('Preview font'), findsOneWidget);
      expect(find.text('Code font'), findsOneWidget);
      expect(find.text('Text size'), findsOneWidget);
      expect(
        find.byType(FontChoiceRow),
        findsNWidgets(
          KansoFontFamily.editorChoices.length +
              KansoFontFamily.previewChoices.length +
              KansoFontFamily.codeChoices.length,
        ),
      );
      expect(find.text('Add a font\u2026'), findsNothing);
      expect(find.text('Google Font'), findsOneWidget);
      expect(find.text('Download and use'), findsOneWidget);
    });

    testWidgets('shows loading and offline feedback for Google Fonts', (
      tester,
    ) async {
      await _pump(
        tester,
        bloc: SeededAppearanceBloc(
          const AppearanceState(googleFontLoading: true),
        ),
        settle: false,
      );
      expect(find.text('Loading font…'), findsOneWidget);
      await tester.pumpWidget(const SizedBox.shrink());
      final bloc = SeededAppearanceBloc(
        const AppearanceState(
          googleFontError: 'offline',
          googleFontOffline: true,
        ),
      );
      await _pump(tester, bloc: bloc);
      expect(
        find.text(
          'You’re offline. Connect to download this font; saved fonts remain '
          'available offline.',
        ),
        findsOneWidget,
      );
    });

    testWidgets('checks the font in use in each section', (tester) async {
      await _pump(tester);
      final checked = tester
          .widgetList<FontChoiceRow>(find.byType(FontChoiceRow))
          .where((row) => row.selected)
          .map((row) => row.font)
          .toList();
      expect(checked, <KansoFontFamily>[
        KansoFontFamily.instrumentSans,
        KansoFontFamily.instrumentSans,
        KansoFontFamily.jetBrainsMono,
      ]);
    });

    testWidgets('every bundled row sets its name in its own family', (
      tester,
    ) async {
      await _pump(tester);
      for (final font in KansoFontFamily.values) {
        if (!font.bundled) {
          continue;
        }
        final name = find.descendant(
          of: _row(font),
          matching: find.text(font.displayName),
        );
        expect(tester.widget<Text>(name).style!.fontFamily, font.family);
      }
    });

    testWidgets('the device font is labelled plainly and not drawn in itself', (
      tester,
    ) async {
      await _pump(tester);
      final name = find.descendant(
        of: _row(KansoFontFamily.system),
        matching: find.text('System \u2014 your device\u2019s font'),
      );
      expect(name, findsOneWidget);
      expect(
        tester.widget<Text>(name).style!.fontFamily,
        'Instrument Sans',
      );
    });

    testWidgets('choosing a font in one section changes only that one', (
      tester,
    ) async {
      final bloc = await _pump(tester);
      await tester.tap(_row(KansoFontFamily.sourceSerif4, index: 1));
      await tester.pump();
      expect(bloc.state.fonts.previewFont, KansoFontFamily.sourceSerif4);
      expect(bloc.state.fonts.editorFont, KansoFontFamily.instrumentSans);
      final samples = find.textContaining('Calm software says less');
      expect(
        tester.widget<Text>(samples.first).style!.fontFamily,
        'Instrument Sans',
      );
      expect(
        tester.widget<Text>(samples.last).style!.fontFamily,
        'Source Serif 4',
      );
      await tester.tap(_row(KansoFontFamily.ibmPlexMono, index: 1));
      await tester.pump();
      expect(bloc.state.fonts.codeFont, KansoFontFamily.ibmPlexMono);
      await tester.tap(_row(KansoFontFamily.literata));
      await tester.pump();
      expect(bloc.state.fonts.editorFont, KansoFontFamily.literata);
    });

    testWidgets('the check follows the choice', (tester) async {
      await _pump(tester);
      await tester.tap(_row(KansoFontFamily.inter));
      await tester.pump();
      final row = tester.widget<FontChoiceRow>(_row(KansoFontFamily.inter));
      expect(row.selected, isTrue);
      expect(
        find.descendant(
          of: _row(KansoFontFamily.inter),
          matching: find.byWidgetPredicate(
            (widget) => widget is KansoIcon && widget.icon == KansoIcons.check,
          ),
        ),
        findsOneWidget,
      );
    });

    testWidgets('the text size controls step down and up', (tester) async {
      final bloc = await _pump(tester);
      expect(bloc.state.textSize, 1.0);
      await tester.tap(find.text('+').first);
      await tester.pump();
      expect(bloc.state.textSize, 1.05);
      final sample = tester.widget<Text>(
        find.textContaining('Calm software says less').first,
      );
      expect(sample.style!.fontSize, closeTo(18.9, 1e-9));
      await tester.tap(find.text('\u2212').first);
      await tester.pump();
      expect(bloc.state.textSize, 1.0);
      expect(bloc.state.previewTextSize, 1.0);
    });

    testWidgets('dragging the editor track updates its scale independently', (
      tester,
    ) async {
      final bloc = await _pump(tester);
      final track = find.descendant(
        of: find.byType(ContinuousTextSizeControl).first,
        matching: find.byType(CustomPaint),
      );
      final rect = tester.getRect(track);
      await tester.dragFrom(
        Offset(rect.center.dx, rect.center.dy),
        Offset(rect.width * 0.25, 0),
      );
      await tester.pump();
      expect(bloc.state.textSize, 1.55);
      expect(bloc.state.previewTextSize, 1.0);
      expect(find.text('155%'), findsOneWidget);
    });

    testWidgets('the preview has its own accessible scale control', (
      tester,
    ) async {
      final bloc = await _pump(tester);
      expect(find.text('Preview text size'), findsOneWidget);
      expect(
        find.text('The preview’s text, in Split and Preview.'),
        findsOneWidget,
      );
      await tester.tap(find.text('+').last);
      await tester.pump();
      expect(bloc.state.previewTextSize, 1.05);
      expect(bloc.state.textSize, 1.0);
      expect(find.bySemanticsLabel('Increase text size'), findsNWidgets(2));
      expect(find.bySemanticsLabel('Text size'), findsNWidgets(2));
    });
  });
}
