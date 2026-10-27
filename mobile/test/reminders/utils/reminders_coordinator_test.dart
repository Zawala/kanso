// Kanso — reminders_coordinator_test.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:clock/clock.dart';
import 'package:dartz/dartz.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/core/models/data/preferences.dart';
import 'package:kanso/core/models/data/reminder_permission.dart';
import 'package:kanso/core/models/data/reminder_response.dart';
import 'package:kanso/core/utils/reminder_channel.dart';
import 'package:kanso/reminders/utils/reminders_coordinator.dart';
import 'package:mocktail/mocktail.dart';

import '../../helpers/fixture_library_repository.dart';
import '../../helpers/fixture_reminder_notifier.dart';
import '../../helpers/mock_repositories.dart';
import '../../helpers/seeded_appearance_bloc.dart';

const _reminder = ReminderNote(
  key: 'note:a#3',
  noteId: 'note:a',
  line: 3,
  title: 'Call Sam',
  body: '09:00 · Plans',
  fireAt: LocalTime(
    year: 2026,
    month: 10,
    day: 27,
    hour: 9,
    minute: 0,
    second: 0,
  ),
);

void main() {
  late MockPreferencesRepository preferences;
  late MockRemindersRepository reminders;
  late FixtureReminderNotifier notifier;
  late FixtureLibraryRepository library;
  late RemindersCoordinator coordinator;
  var enabled = true;

  setUpAll(() {
    registerFallbackValue(ReminderAnswer.open);
  });

  setUp(() {
    enabled = true;
    preferences = MockPreferencesRepository();
    reminders = MockRemindersRepository();
    notifier = FixtureReminderNotifier()..current = ReminderPermission.granted;
    library = FixtureLibraryRepository();
    addTearDown(library.close);
    addTearDown(notifier.live.close);
    when(() => preferences.load()).thenAnswer(
      (_) async => Right<ApplicationError, Preferences>(
        Preferences(remindersEnabled: enabled),
      ),
    );
    when(
      () => reminders.plan(
        nowUtcMs: any(named: 'nowUtcMs'),
        offsetMs: any(named: 'offsetMs'),
        canNotify: any(named: 'canNotify'),
      ),
    ).thenAnswer(
      (_) async => const Right<ApplicationError, ReminderPlan>(
        ReminderPlan(
          showNow: <ReminderNote>[_reminder],
          schedule: <ReminderNote>[_reminder],
        ),
      ),
    );
    when(
      () => reminders.answer(
        key: any(named: 'key'),
        action: any(named: 'action'),
        nowUtcMs: any(named: 'nowUtcMs'),
      ),
    ).thenAnswer(
      (_) async => const Right<ApplicationError, TaskPlace?>(
        TaskPlace(noteId: 'note:a', line: 3),
      ),
    );
    coordinator = RemindersCoordinator(
      preferences,
      reminders,
      notifier,
      library,
    );
    addTearDown(coordinator.dispose);
  });

  Future<T> atTen<T>(Future<T> Function() body) => withClock(
    Clock.fixed(DateTime.utc(2026, 10, 27, 10)),
    body,
  );

  group('RemindersCoordinator', () {
    test('plans, shows what is due and schedules the rest', () async {
      await atTen(coordinator.refresh);
      expect(notifier.shown.single, <ReminderNote>[_reminder]);
      expect(notifier.scheduled.single, <ReminderNote>[_reminder]);
      expect(notifier.cancels, 0);
      final captured = verify(
        () => reminders.plan(
          nowUtcMs: captureAny(named: 'nowUtcMs'),
          offsetMs: any(named: 'offsetMs'),
          canNotify: captureAny(named: 'canNotify'),
        ),
      ).captured;
      expect(captured, <Object?>[
        DateTime.utc(2026, 10, 27, 10).millisecondsSinceEpoch,
        true,
      ]);
    });

    test('with reminders off it removes them all and plans nothing', () async {
      enabled = false;
      await coordinator.refresh();
      expect(notifier.cancels, 1);
      expect(notifier.shown, isEmpty);
      expect(notifier.scheduled, isEmpty);
      verifyNever(
        () => reminders.plan(
          nowUtcMs: any(named: 'nowUtcMs'),
          offsetMs: any(named: 'offsetMs'),
          canNotify: any(named: 'canNotify'),
        ),
      );
    });

    test('without the permission the core is told it cannot notify', () async {
      notifier.current = ReminderPermission.denied;
      await coordinator.refresh();
      verify(
        () => reminders.plan(
          nowUtcMs: any(named: 'nowUtcMs'),
          offsetMs: any(named: 'offsetMs'),
          canNotify: false,
        ),
      ).called(1);
    });

    test('a plan that fails leaves the system as it is', () async {
      when(
        () => reminders.plan(
          nowUtcMs: any(named: 'nowUtcMs'),
          offsetMs: any(named: 'offsetMs'),
          canNotify: any(named: 'canNotify'),
        ),
      ).thenAnswer(
        (_) async => const Left<ApplicationError, ReminderPlan>(
          ApplicationError.storage(message: 'Disk.'),
        ),
      );
      await coordinator.refresh();
      expect(notifier.shown, isEmpty);
      expect(notifier.scheduled, isEmpty);
      expect(notifier.cancels, 0);
    });

    test('answers waiting are applied once, then it plans', () async {
      notifier.waiting = <ReminderResponse>[
        const ReminderResponse(
          key: 'note:a#3',
          noteId: 'note:a',
          line: 3,
          action: ReminderAnswer.markDone,
        ),
        const ReminderResponse(
          key: 'note:a#4',
          noteId: 'note:a',
          line: 4,
          action: ReminderAnswer.dismiss,
        ),
      ];
      await coordinator.resumed();
      await coordinator.resumed();
      verify(
        () => reminders.answer(
          key: 'note:a#3',
          action: ReminderAnswer.markDone,
          nowUtcMs: any(named: 'nowUtcMs'),
        ),
      ).called(1);
      verify(
        () => reminders.answer(
          key: 'note:a#4',
          action: ReminderAnswer.dismiss,
          nowUtcMs: any(named: 'nowUtcMs'),
        ),
      ).called(1);
      verify(
        () => reminders.plan(
          nowUtcMs: any(named: 'nowUtcMs'),
          offsetMs: any(named: 'offsetMs'),
          canNotify: any(named: 'canNotify'),
        ),
      ).called(2);
    });

    test('a tap that opens a task is its place on opened', () async {
      final places = <TaskPlace>[];
      final subscription = coordinator.opened.listen(places.add);
      addTearDown(subscription.cancel);
      notifier.waiting = <ReminderResponse>[
        const ReminderResponse(
          key: 'note:a#3',
          noteId: 'note:a',
          line: 3,
          action: ReminderAnswer.open,
        ),
      ];
      await coordinator.resumed();
      await Future<void>.delayed(Duration.zero);
      expect(places, <TaskPlace>[const TaskPlace(noteId: 'note:a', line: 3)]);
    });

    test('a task the core no longer has opens its note at the line', () async {
      when(
        () => reminders.answer(
          key: any(named: 'key'),
          action: any(named: 'action'),
          nowUtcMs: any(named: 'nowUtcMs'),
        ),
      ).thenAnswer((_) async => const Right<ApplicationError, TaskPlace?>(null));
      final places = <TaskPlace>[];
      final subscription = coordinator.opened.listen(places.add);
      addTearDown(subscription.cancel);
      notifier.waiting = <ReminderResponse>[
        const ReminderResponse(
          key: 'k',
          noteId: 'note:b',
          line: 9,
          action: ReminderAnswer.open,
        ),
      ];
      await coordinator.resumed();
      await Future<void>.delayed(Duration.zero);
      expect(places.single, const TaskPlace(noteId: 'note:b', line: 9));
    });

    test('answers given while it runs are applied once each', () async {
      await coordinator.start();
      notifier.live.add(
        const ReminderResponse(
          key: 'note:a#3',
          noteId: 'note:a',
          line: 3,
          action: ReminderAnswer.markDone,
        ),
      );
      await Future<void>.delayed(const Duration(milliseconds: 50));
      verify(
        () => reminders.answer(
          key: 'note:a#3',
          action: ReminderAnswer.markDone,
          nowUtcMs: any(named: 'nowUtcMs'),
        ),
      ).called(1);
    });

    test('a change to the library plans again, once for a burst', () async {
      await coordinator.start();
      clearInteractions(reminders);
      library.changeStream
        ..add(<String>['a'])
        ..add(<String>['b'])
        ..add(<String>['c']);
      await Future<void>.delayed(
        remindersDebounce + const Duration(milliseconds: 200),
      );
      verify(
        () => reminders.plan(
          nowUtcMs: any(named: 'nowUtcMs'),
          offsetMs: any(named: 'offsetMs'),
          canNotify: any(named: 'canNotify'),
        ),
      ).called(1);
    });
  });

  group('the channel payload', () {
    test('carries only the key, note, line, words and fire time', () {
      expect(reminderToChannel(_reminder), <String, Object>{
        'key': 'note:a#3',
        'noteId': 'note:a',
        'line': 3,
        'title': 'Call Sam',
        'body': '09:00 · Plans',
        'year': 2026,
        'month': 10,
        'day': 27,
        'hour': 9,
        'minute': 0,
      });
    });

    test('answers have the names the native side writes', () {
      expect(reminderAnswerWire(ReminderAnswer.open), 'open');
      expect(reminderAnswerWire(ReminderAnswer.markDone), 'mark-done');
      expect(reminderAnswerWire(ReminderAnswer.dismiss), 'dismiss');
      for (final answer in ReminderAnswer.values) {
        expect(reminderAnswerOf(reminderAnswerWire(answer)), answer);
      }
      expect(reminderAnswerOf('later'), isNull);
    });

    test('a native response is read, and a malformed one is not', () {
      expect(
        reminderResponseOf(<String, Object>{
          'key': 'k',
          'noteId': 'n',
          'line': 2,
          'action': 'mark-done',
        }),
        const ReminderResponse(
          key: 'k',
          noteId: 'n',
          line: 2,
          action: ReminderAnswer.markDone,
        ),
      );
      expect(reminderResponseOf(<String, Object>{'key': 'k'}), isNull);
      expect(reminderResponseOf('nope'), isNull);
    });
  });
}
