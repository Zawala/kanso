// Kanso — editor_goldens_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/types.dart' as core;
import 'package:kanso/core/configs/kanso_palette.dart';
import 'package:kanso/core/utils/context_extensions.dart';
import 'package:kanso/editor/state/editor/editor_bloc.dart';
import 'package:kanso/editor/views/controllers/kanso_markdown_controller.dart';
import 'package:kanso/editor/views/widgets/kanso_editable_text.dart';
import 'package:kanso/editor/views/widgets/markdown_editor.dart';
import 'package:kanso/editor/views/widgets/properties_row.dart';
import 'package:kanso/editor/views/widgets/property_row.dart';

import '../../../helpers/editor_fixture.dart';
import '../../../helpers/fake_editor_document.dart';
import '../../../helpers/golden.dart';

/// The height the keyboard is drawn at on a phone.
const double _keyboard = 336;

/// [page] with a block where the keyboard would be, as the mocks have.
Widget _withKeyboard(Widget page) {
  return Stack(
    children: <Widget>[
      page,
      Positioned(
        left: 0,
        right: 0,
        bottom: 0,
        height: _keyboard,
        child: Builder(
          builder: (context) => ColoredBox(color: context.colors.selected),
        ),
      ),
    ],
  );
}

/// Puts the keyboard up: the page makes room for it.
Future<void> _keyboardUp(WidgetTester tester) async {
  tester.view.viewInsets = const FakeViewPadding(bottom: _keyboard * 3);
  await tester.pump();
  await tester.pump();
}

EditorBloc _bloc(WidgetTester tester) =>
    tester.element(find.byType(KansoEditableText)).read<EditorBloc>();

/// The note with [last] at the end of the text.
String _body(String last) =>
    '$goldenFront## Open questions\n'
    '- How long a device may stay offline\n'
    '- What a conflict looks like to a person\n'
    '\n'
    '$last';

/// A spell checker that marks every `teh` as a misspelling of `the`.
class TehSpellCheckService implements SpellCheckService {
  @override
  Future<List<SuggestionSpan>?> fetchSpellCheckSuggestions(
    Locale locale,
    String text,
  ) async => <SuggestionSpan>[
    for (final match in RegExp('teh').allMatches(text))
      SuggestionSpan(
        TextRange(start: match.start, end: match.end),
        const <String>['the', 'tech', 'ten'],
      ),
  ];
}

void main() {
  setUp(() => EditableText.debugDeterministicCursor = true);
  tearDown(() => EditableText.debugDeterministicCursor = false);

  for (final brightness in Brightness.values) {
    final name = brightness.name;

    group('on a phone, $name', () {
      testWidgets('a note being edited', (tester) async {
        final fixture = await EditorFixture.open(
          tester,
          brightness: brightness,
        );
        await expectGoldenOnBothPlatforms(
          tester,
          fixture.phone,
          name: 'phone_edit_$name',
          size: phoneSize,
          brightness: brightness,
          interact: fixture.settle,
        );
      });

      testWidgets('with the keyboard up, and the bar over it', (tester) async {
        final fixture = await EditorFixture.open(
          tester,
          brightness: brightness,
        );
        await expectGoldenOnBothPlatforms(
          tester,
          _withKeyboard(fixture.phone),
          name: 'phone_keyboard_$name',
          size: phoneSize,
          brightness: brightness,
          interact: (tester) async {
            await fixture.settle(tester);
            await _keyboardUp(tester);
          },
        );
      });

      testWidgets('a link to a note being typed', (tester) async {
        final fixture = await EditorFixture.open(
          tester,
          brightness: brightness,
          body: _body('See also [[Sy'),
        );
        await expectGoldenOnBothPlatforms(
          tester,
          _withKeyboard(fixture.phone),
          name: 'phone_wiki_completion_$name',
          size: phoneSize,
          brightness: brightness,
          interact: (tester) async {
            await fixture.settle(tester);
            await _keyboardUp(tester);
            await fixture.settle(tester);
          },
        );
      });

      testWidgets('a tag being typed', (tester) async {
        final fixture = await EditorFixture.open(
          tester,
          brightness: brightness,
          body: _body('Filed under #to'),
        );
        await expectGoldenOnBothPlatforms(
          tester,
          _withKeyboard(fixture.phone),
          name: 'phone_tag_completion_$name',
          size: phoneSize,
          brightness: brightness,
          interact: (tester) async {
            await fixture.settle(tester);
            await _keyboardUp(tester);
            await fixture.settle(tester);
          },
        );
      });

      testWidgets('the properties open', (tester) async {
        final fixture = await EditorFixture.open(
          tester,
          brightness: brightness,
        );
        await expectGoldenOnBothPlatforms(
          tester,
          fixture.phone,
          name: 'phone_properties_open_$name',
          size: phoneSize,
          brightness: brightness,
          interact: (tester) async {
            await fixture.settle(tester);
            await tester.tap(find.byType(PropertiesRow));
            await fixture.settle(tester);
          },
        );
      });

      testWidgets('a property being edited', (tester) async {
        final fixture = await EditorFixture.open(
          tester,
          brightness: brightness,
        );
        await expectGoldenOnBothPlatforms(
          tester,
          _withKeyboard(fixture.phone),
          name: 'phone_properties_editing_$name',
          size: phoneSize,
          brightness: brightness,
          interact: (tester) async {
            await fixture.settle(tester);
            await tester.tap(find.byType(PropertiesRow));
            await fixture.settle(tester);
            await tester.tap(
              find.descendant(
                of: find.byType(PropertyRow).first,
                matching: find.byType(EditableText),
              ),
            );
            await _keyboardUp(tester);
            await fixture.settle(tester);
          },
        );
      });

      testWidgets('a change made elsewhere', (tester) async {
        final fixture = await EditorFixture.open(
          tester,
          brightness: brightness,
        );
        await expectGoldenOnBothPlatforms(
          tester,
          fixture.phone,
          name: 'phone_conflict_changed_$name',
          size: phoneSize,
          brightness: brightness,
          interact: (tester) async {
            await fixture.settle(tester);
            _bloc(tester).add(
              const EditorEvent.signalReceived(
                core.EditorEvent.incoming(deleted: false),
              ),
            );
            await fixture.settle(tester);
          },
        );
      });

      testWidgets('a note deleted elsewhere', (tester) async {
        final fixture = await EditorFixture.open(
          tester,
          brightness: brightness,
        );
        await expectGoldenOnBothPlatforms(
          tester,
          fixture.phone,
          name: 'phone_conflict_deleted_$name',
          size: phoneSize,
          brightness: brightness,
          interact: (tester) async {
            await fixture.settle(tester);
            _bloc(tester).add(
              const EditorEvent.signalReceived(
                core.EditorEvent.incoming(deleted: true),
              ),
            );
            await fixture.settle(tester);
          },
        );
      });

      testWidgets('a save that failed', (tester) async {
        final fixture = await EditorFixture.open(
          tester,
          brightness: brightness,
        );
        await expectGoldenOnBothPlatforms(
          tester,
          fixture.phone,
          name: 'phone_save_failed_$name',
          size: phoneSize,
          brightness: brightness,
          interact: (tester) async {
            await fixture.settle(tester);
            _bloc(tester).add(
              const EditorEvent.signalReceived(
                core.EditorEvent.failed(message: 'disk'),
              ),
            );
            await fixture.settle(tester);
          },
        );
      });

      testWidgets('text selected, with its handles and menu', (tester) async {
        final fixture = await EditorFixture.open(
          tester,
          brightness: brightness,
        );
        await expectGoldenOnBothPlatforms(
          tester,
          _withKeyboard(fixture.phone),
          name: 'phone_selection_menu_$name',
          size: phoneSize,
          brightness: brightness,
          interact: (tester) async {
            await fixture.settle(tester);
            await _keyboardUp(tester);
            final state = tester.state<EditableTextState>(
              find.byType(KansoEditableText),
            );
            final text = state.textEditingValue.text;
            final start = text.indexOf('conflict');
            state.userUpdateTextEditingValue(
              state.textEditingValue.copyWith(
                selection: TextSelection(
                  baseOffset: start,
                  extentOffset: start + 'conflict'.length,
                ),
              ),
              SelectionChangedCause.longPress,
            );
            await tester.pump();
            state.selectionOverlay
              ?..handlesVisible = true
              ..showHandles();
            state.showToolbar();
            await tester.pump(const Duration(milliseconds: 400));
            await tester.pump(const Duration(milliseconds: 400));
          },
        );
      });

      testWidgets('a misspelled word and its menu', (tester) async {
        final document = FakeEditorDocument(text: 'Notes on the sync design.');
        final controller = KansoMarkdownController(
          document: document,
          snapshot: document.current,
          themeId: 'kanso-$name',
        );
        final focus = FocusNode();
        addTearDown(() {
          focus.dispose();
          controller.dispose();
        });
        await expectGoldenOnBothPlatforms(
          tester,
          Scaffold(
            body: SafeArea(
              child: Padding(
                padding: const EdgeInsets.all(16),
                child: MarkdownEditor(
                  controller: controller,
                  focusNode: focus,
                  spellCheckService: TehSpellCheckService(),
                ),
              ),
            ),
          ),
          name: 'phone_spell_menu_$name',
          size: phoneSize,
          brightness: brightness,
          interact: (tester) async {
            // Each platform starts from the same text and no focus.
            focus.unfocus();
            controller.value = const TextEditingValue(
              text: 'Notes on the sync design.',
            );
            await tester.pump();
            focus.requestFocus();
            await tester.pump();
            tester.testTextInput.updateEditingValue(
              const TextEditingValue(
                text: 'Notes on teh sync design, and `teh` in code.',
                selection: TextSelection.collapsed(offset: 10),
              ),
            );
            await tester.pump(const Duration(milliseconds: 50));
            await tester.pump(const Duration(milliseconds: 50));
            tester
                .state<EditableTextState>(find.byType(KansoEditableText))
                .showSpellCheckSuggestionsToolbar();
            await tester.pump(const Duration(milliseconds: 400));
          },
        );
      });

      testWidgets('the info of the note', (tester) async {
        final fixture = await EditorFixture.open(
          tester,
          brightness: brightness,
        );
        await expectGoldenOnBothPlatforms(
          tester,
          fixture.phone,
          name: 'phone_info_sheet_$name',
          size: phoneSize,
          brightness: brightness,
          interact: (tester) async {
            await fixture.settle(tester);
            await tester.tap(find.bySemanticsLabel('Note info'));
            await tester.pump(const Duration(milliseconds: 600));
          },
        );
      });
    });

    group('on a tablet, $name', () {
      Future<void> tabletGolden(
        WidgetTester tester,
        String file,
        Future<void> Function(WidgetTester, EditorFixture) interact, {
        String? body,
      }) async {
        final palette = brightness == Brightness.dark
            ? KansoPalette.kansoDark
            : KansoPalette.kansoLight;
        final fixture = await EditorFixture.open(
          tester,
          brightness: brightness,
          body: body ?? '$goldenFront$goldenText',
        );
        await expectGoldenOnBothPlatforms(
          tester,
          fixture.tablet(palette.colors.sidebar, palette.colors.list),
          name: '${file}_$name',
          size: tabletLandscapeSize,
          brightness: brightness,
          interact: (tester) => interact(tester, fixture),
        );
      }

      testWidgets('a note being edited, with the bar', (tester) async {
        await tabletGolden(
          tester,
          'tablet_edit',
          (tester, fixture) => fixture.settle(tester),
        );
      });

      testWidgets('a link to a note being typed', (tester) async {
        await tabletGolden(
          tester,
          'tablet_wiki_completion',
          (tester, fixture) => fixture.settle(tester),
          body: _body('See also [[Sy'),
        );
      });

      testWidgets('the properties open', (tester) async {
        await tabletGolden(tester, 'tablet_properties_open', (
          tester,
          fixture,
        ) async {
          await fixture.settle(tester);
          await tester.tap(find.byType(PropertiesRow));
          await fixture.settle(tester);
        });
      });
    });
  }
}
