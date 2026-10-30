// Kanso — kanso_page_route_test.dart.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/cupertino.dart';
import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:go_router/go_router.dart';
import 'package:kanso/core/views/widgets/kanso_page_route.dart';
import '../../../helpers/pump.dart';

void main() {
  Future<void> openPage(
    WidgetTester tester, {
    Size size = const Size(390, 844),
  }) async {
    tester.view
      ..devicePixelRatio = 1
      ..physicalSize = size;
    await pumpKanso(
      tester,
      Builder(
        builder: (context) => TextButton(
          onPressed: () => Navigator.of(context).push<void>(
            KansoPageRoute<void>(builder: (_) => const Text('Destination')),
          ),
          child: const Text('Open'),
        ),
      ),
    );
    await tester.tap(find.text('Open'));
    await tester.pumpAndSettle();
  }

  testWidgets('Android phone routes use owned fade and slide transitions', (
    tester,
  ) async {
    await openPage(tester);

    expect(find.byType(FadeTransition), findsWidgets);
    expect(find.byType(SlideTransition), findsOneWidget);
  });

  testWidgets('Android tablet routes use an owned fade transition', (
    tester,
  ) async {
    await openPage(tester, size: const Size(1024, 768));

    expect(find.byType(FadeTransition), findsWidgets);
    expect(find.byType(SlideTransition), findsNothing);
  });

  testWidgets('reduced motion removes Android route animation', (tester) async {
    tester.platformDispatcher.accessibilityFeaturesTestValue =
        const FakeAccessibilityFeatures(disableAnimations: true);
    addTearDown(tester.platformDispatcher.clearAccessibilityFeaturesTestValue);
    await openPage(tester);

    expect(find.text('Destination'), findsOneWidget);
    expect(find.byType(FadeTransition), findsNothing);
    expect(find.byType(SlideTransition), findsNothing);
  });

  test('GoRouter pages are native on iPhone and owned elsewhere', () {
    const child = SizedBox.shrink();
    const key = ValueKey<String>('page');
    try {
      debugDefaultTargetPlatformOverride = TargetPlatform.iOS;
      expect(
        kansoPage<void>(key: key, child: child),
        isA<CupertinoPage<void>>(),
      );

      debugDefaultTargetPlatformOverride = TargetPlatform.android;
      expect(
        kansoPage<void>(key: key, child: child),
        isA<CustomTransitionPage<void>>(),
      );
    } finally {
      debugDefaultTargetPlatformOverride = null;
    }
  });

  testWidgets('iPhone edge swipe returns to the previous route', (
    tester,
  ) async {
    debugDefaultTargetPlatformOverride = TargetPlatform.iOS;
    try {
      await openPage(tester);

      await tester.dragFrom(const Offset(1, 400), const Offset(300, 0));
      await tester.pumpAndSettle();

      expect(find.text('Open'), findsOneWidget);
      expect(find.text('Destination'), findsNothing);
    } finally {
      debugDefaultTargetPlatformOverride = null;
    }
  });
}
