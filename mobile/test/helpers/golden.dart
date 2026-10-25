// Kanso — golden.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:flutter/rendering.dart';
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

/// The safe area of an iPhone with a Dynamic Island, in logical pixels.
const EdgeInsets phoneSafeArea = EdgeInsets.only(top: 59, bottom: 34);

/// The safe area of an iPad, in logical pixels.
const EdgeInsets tabletSafeArea = EdgeInsets.only(top: 24, bottom: 20);

/// Renders [child] inside the Kanso app shell at [size] in [brightness], once
/// as iOS and once as Android.
///
/// The two renders must be identical: their pixels are compared in memory,
/// with no tolerance, and the test fails with the number of pixels that
/// differ. The iOS render is then compared with `goldens/<name>.png` (see
/// `KansoGoldenComparator` for the tolerance against the file). Text scale
/// is 1 and animations are off. The golden path is relative to the test
/// file that calls this.
///
/// [safeArea] simulates the device's status bar and home indicator for both
/// platforms; it defaults to an iPhone's on a phone and an iPad's on a
/// tablet.
Future<void> expectGoldenOnBothPlatforms(
  WidgetTester tester,
  Widget child, {
  required String name,
  required Size size,
  required Brightness brightness,
  EdgeInsets? safeArea,
}) async {
  final devicePixelRatio = size.shortestSide < 600 ? 3.0 : 2.0;
  final insets =
      safeArea ?? (size.shortestSide < 600 ? phoneSafeArea : tabletSafeArea);
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

  final renders = <TargetPlatform, Uint8List>{};
  for (final platform in <TargetPlatform>[
    TargetPlatform.iOS,
    TargetPlatform.android,
  ]) {
    debugDefaultTargetPlatformOverride = platform;
    try {
      final key = ValueKey<String>('golden-${platform.name}');
      await tester.pumpWidget(
        RepaintBoundary(
          key: key,
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
            builder: (context, app) {
              return MediaQuery(
                data: MediaQuery.of(
                  context,
                ).copyWith(padding: insets, viewPadding: insets),
                child: app!,
              );
            },
            home: child,
          ),
        ),
      );
      await tester.pumpAndSettle();
      final boundary = tester.renderObject<RenderRepaintBoundary>(
        find.byKey(key),
      );
      renders[platform] = (await tester.runAsync(() async {
        final image = await boundary.toImage(pixelRatio: devicePixelRatio);
        final data = await image.toByteData();
        image.dispose();
        return data!.buffer.asUint8List();
      }))!;
      if (platform == TargetPlatform.iOS) {
        await expectLater(
          find.byKey(key),
          matchesGoldenFile('goldens/$name.png'),
        );
      }
    } finally {
      debugDefaultTargetPlatformOverride = null;
    }
  }

  final differing = _differingPixels(
    renders[TargetPlatform.iOS]!,
    renders[TargetPlatform.android]!,
  );
  expect(
    differing,
    0,
    reason: '$differing pixels differ between the iOS and Android renders',
  );
}

int _differingPixels(Uint8List a, Uint8List b) {
  if (a.length != b.length) {
    return a.length > b.length ? a.length ~/ 4 : b.length ~/ 4;
  }
  var count = 0;
  for (var i = 0; i < a.length; i += 4) {
    if (a[i] != b[i] ||
        a[i + 1] != b[i + 1] ||
        a[i + 2] != b[i + 2] ||
        a[i + 3] != b[i + 3]) {
      count++;
    }
  }
  return count;
}
