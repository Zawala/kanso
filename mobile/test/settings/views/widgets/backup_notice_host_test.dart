// Kanso — backup_notice_host_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/settings/state/backup_notice/backup_notice_bloc.dart';
import 'package:kanso/settings/views/widgets/backup_notice_card.dart';
import 'package:kanso/settings/views/widgets/backup_notice_host.dart';

import '../../../helpers/close_bloc_later.dart';
import '../../../helpers/pump.dart';
import '../../../helpers/seeded_appearance_bloc.dart';

Future<BackupNoticeBloc> _pump(
  WidgetTester tester, {
  required ValueNotifier<int> taps,
}) async {
  final bloc = BackupNoticeBloc(ForgetfulPreferencesRepository())
    ..add(const BackupNoticeEvent.started());
  closeBlocLater(bloc);
  await pumpKanso(
    tester,
    BlocProvider<BackupNoticeBloc>.value(
      value: bloc,
      child: BackupNoticeHost(
        child: SizedBox.expand(
          child: GestureDetector(
            behavior: HitTestBehavior.opaque,
            onTap: () => taps.value++,
            child: const Center(child: Text('Behind')),
          ),
        ),
      ),
    ),
  );
  await tester.pump();
  return bloc;
}

void main() {
  group('BackupNoticeHost', () {
    testWidgets('says once that the notes are not in device backups', (
      tester,
    ) async {
      final taps = ValueNotifier<int>(0);
      addTearDown(taps.dispose);
      await _pump(tester, taps: taps);
      expect(find.byType(BackupNoticeCard), findsOneWidget);
      expect(find.text('Notes stay on this phone'), findsOneWidget);
      expect(find.textContaining('device backups'), findsOneWidget);
      expect(find.text('OK'), findsOneWidget);
    });

    testWidgets('covers what is behind it until OK, then goes', (tester) async {
      final taps = ValueNotifier<int>(0);
      addTearDown(taps.dispose);
      final bloc = await _pump(tester, taps: taps);
      await tester.tapAt(const Offset(10, 10));
      expect(taps.value, 0);

      await tester.tap(find.text('OK'));
      await tester.pump();
      await tester.pump();
      expect(find.byType(BackupNoticeCard), findsNothing);
      expect(bloc.state.visible, isFalse);
      await tester.tapAt(const Offset(10, 10));
      expect(taps.value, 1);
    });
  });
}
