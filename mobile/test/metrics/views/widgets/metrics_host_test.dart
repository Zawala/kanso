// Kanso — metrics_host_test.dart
// Created 2026-10-31.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/ai_settings/state/ai_status/ai_status_bloc.dart';
import 'package:kanso/metrics/views/widgets/metrics_host.dart';

import '../../../helpers/fake_usage_metrics.dart';
import '../../../helpers/fixture_ai_repository.dart';
import '../../../helpers/kanso_test_app.dart';

void main() {
  Future<AiStatusBloc> pumpHost(
    WidgetTester tester,
    FakeUsageMetrics metrics, {
    Duration firstSendDelay = const Duration(seconds: 20),
  }) async {
    final ai = AiStatusBloc(FixtureAiRepository());
    addTearDown(ai.close);
    await tester.pumpWidget(
      KansoTestApp(
        home: BlocProvider<AiStatusBloc>.value(
          value: ai,
          child: MetricsHost(
            metrics: metrics,
            firstSendDelay: firstSendDelay,
            child: const Scaffold(body: Text('the app')),
          ),
        ),
      ),
    );
    // The first read starts after the first frame.
    await tester.pump();
    await tester.pump();
    return ai;
  }

  testWidgets('shows the notice and sends only after OK', (tester) async {
    final metrics = FakeUsageMetrics();
    await pumpHost(tester, metrics);

    expect(find.text('Kanso counts, it doesn’t watch'), findsOneWidget);
    expect(metrics.sendDueCalls, 0);

    await tester.tap(find.text('OK'));
    await tester.pump();
    await tester.pump();

    expect(metrics.markSeenCalls, 1);
    expect(metrics.sendDueCalls, 1);
    expect(metrics.sends.single.osVersion, isNotEmpty);

    // Let the host dispose its hourly timer.
    await tester.pumpWidget(const SizedBox.shrink());
  });

  testWidgets('a notice already seen sends after the settle delay', (
    tester,
  ) async {
    final metrics = FakeUsageMetrics(seen: true);
    await pumpHost(
      tester,
      metrics,
      firstSendDelay: const Duration(milliseconds: 20),
    );

    expect(find.text('Kanso counts, it doesn’t watch'), findsNothing);
    expect(metrics.sendDueCalls, 0);

    await tester.pump(const Duration(milliseconds: 21));
    await tester.pump();

    expect(metrics.sendDueCalls, 1);
    expect(metrics.markSeenCalls, 0);

    await tester.pumpWidget(const SizedBox.shrink());
  });

  testWidgets('a failed notice read shows nothing and sends nothing', (
    tester,
  ) async {
    final metrics = FakeUsageMetrics(noticeSeenThrows: true);
    await pumpHost(tester, metrics);

    expect(find.text('Kanso counts, it doesn’t watch'), findsNothing);
    expect(metrics.sendDueCalls, 0);

    await tester.pumpWidget(const SizedBox.shrink());
  });
}
