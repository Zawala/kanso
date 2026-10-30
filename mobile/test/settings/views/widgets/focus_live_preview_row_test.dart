// Kanso — focus_live_preview_row_test.dart
// Created 2026-10-30.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/models/data/preferences.dart';
import 'package:kanso/core/views/widgets/kanso_switch.dart';
import 'package:kanso/settings/views/widgets/focus_live_preview_row.dart';
import 'package:kanso/shell/state/focus_mode/focus_mode_bloc.dart';

import '../../../helpers/close_bloc_later.dart';
import '../../../helpers/pump.dart';
import '../../../helpers/seeded_appearance_bloc.dart';

void main() {
  group('FocusLivePreviewRow', () {
    testWidgets('is off by default and turns Live Preview on', (tester) async {
      final preferences = SeededPreferencesRepository(const Preferences());
      final focus = FocusModeBloc(preferences)
        ..add(const FocusModeEvent.started());
      closeBlocLater(focus);
      await pumpKanso(
        tester,
        BlocProvider<FocusModeBloc>.value(
          value: focus,
          child: const FocusLivePreviewRow(),
        ),
      );
      expect(find.text('Live Preview in focus mode'), findsOneWidget);
      expect(
        find.text(
          'Focus mode shows the note in Live, and leaving it goes back to '
          'the mode you were in.',
        ),
        findsOneWidget,
      );
      KansoSwitch toggle() =>
          tester.widget<KansoSwitch>(find.byType(KansoSwitch));
      expect(toggle().value, isFalse);

      await tester.tap(find.byType(KansoSwitch));
      await tester.pumpAndSettle();
      expect(focus.state.livePreview, isTrue);
      expect(toggle().value, isTrue);
      expect(preferences.current.livePreviewInFocus, isTrue);
      expect(focus.state.active, isFalse);
    });
  });
}
