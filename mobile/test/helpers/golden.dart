// Kanso — golden.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/configs/kanso_theme.dart';
import 'package:kanso/core/utils/kanso_scroll_behavior.dart';
import 'package:kanso/gen/l10n/app_localizations.dart';

/// The size of a phone in portrait, in logical pixels (device pixel ratio 3).
const Size phoneSize = Size(393, 852);

/// The size of a tablet in landscape (device pixel ratio 2).
const Size tabletLandscapeSize = Size(1180, 820);

/// The size of a tablet in portrait (device pixel ratio 2).
const Size tabletPortraitSize = Size(820, 1180);

/// Renders [child] inside the Kanso app shell at [size] in [brightness] and
/// compares it with `goldens/<name>.png`, once as iOS and once as Android.
///
/// Both renders are compared with the same file, so any difference between
/// the platforms fails the test. Text scale is 1 and animations are off.
/// The golden path is relative to the test file that calls this.
Future<void> expectGoldenOnBothPlatforms(
  WidgetTester tester,
  Widget child, {
  required String name,
  required Size size,
  required Brightness brightness,
}) async {
  final devicePixelRatio = size.shortestSide < 600 ? 3.0 : 2.0;
  tester.view
    ..devicePixelRatio = devicePixelRatio
    ..physicalSize = size * devicePixelRatio;
  tester.platformDispatcher
    ..textScaleFactorTestValue = 1
    ..accessibilityFeaturesTestValue = const FakeAccessibilityFeatures(
      disableAnimations: true,
      reduceMotion: true,
    );
  addTearDown(() {
    tester.view.reset();
    tester.platformDispatcher
      ..clearTextScaleFactorTestValue()
      ..clearAccessibilityFeaturesTestValue();
  });

  for (final platform in <TargetPlatform>[
    TargetPlatform.iOS,
    TargetPlatform.android,
  ]) {
    debugDefaultTargetPlatformOverride = platform;
    try {
      await tester.pumpWidget(
        RepaintBoundary(
          key: ValueKey<String>('golden-${platform.name}'),
          child: MaterialApp(
            debugShowCheckedModeBanner: false,
            // Built per platform: a ThemeData records the platform it was
            // made on, and the shared themes are made once.
            theme: KansoTheme.light.copyWith(platform: platform),
            darkTheme: KansoTheme.dark.copyWith(platform: platform),
            themeMode: brightness == Brightness.dark
                ? ThemeMode.dark
                : ThemeMode.light,
            scrollBehavior: const KansoScrollBehavior(),
            localizationsDelegates: AppLocalizations.localizationsDelegates,
            supportedLocales: AppLocalizations.supportedLocales,
            home: child,
          ),
        ),
      );
      await tester.pumpAndSettle();
      await expectLater(
        find.byKey(ValueKey<String>('golden-${platform.name}')),
        matchesGoldenFile('goldens/$name.png'),
      );
    } finally {
      debugDefaultTargetPlatformOverride = null;
    }
  }
}
