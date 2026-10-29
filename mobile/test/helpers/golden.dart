// Kanso — golden.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:flutter/rendering.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/configs/kanso_palette.dart';
import 'package:kanso/core/configs/kanso_theme.dart';
import 'package:kanso/core/state/appearance/appearance_bloc.dart';

import 'kanso_test_app.dart';
import 'kanso_viewport.dart';
import 'seeded_appearance_bloc.dart';

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
/// is 1, animations are off and shadows are blurred as on a device. The
/// golden path is relative to the test file that calls this.
///
/// [viewport] is the window of a device in the matrix (see [KansoViewport])
/// and takes the place of [size], its density and its safe area.
///
/// [safeArea] simulates the device's status bar and home indicator for both
/// platforms; it defaults to an iPhone's on a phone and an iPad's on a
/// tablet. [appearance] sets the fonts and text size the themes are built
/// with and the state of the `AppearanceBloc` the page can read; it must not
/// use the device's own font (`KansoFontFamily.system`), which differs per
/// platform by design.
///
/// [palette] draws the page in a theme's colours instead of Kanso Light and
/// Dark (which [brightness] chooses between); [brightness] is then only
/// what the platform is told. The palette must come from the theme tokens,
/// not be made up.
///
/// [interact] runs on the settled page, once per platform, before the
/// picture is taken: a swipe, a long press, a tap that opens a sheet.
Future<void> expectGoldenOnBothPlatforms(
  WidgetTester tester,
  Widget child, {
  required String name,
  required Brightness brightness,
  Size? size,
  KansoViewport? viewport,
  EdgeInsets? safeArea,
  AppearanceState appearance = const AppearanceState(),
  KansoPalette? palette,
  Future<void> Function(WidgetTester tester)? interact,
}) async {
  assert(
    (size == null) != (viewport == null),
    'Give either a size or a viewport.',
  );
  final window = viewport?.size ?? size!;
  final devicePixelRatio =
      viewport?.devicePixelRatio ?? (window.shortestSide < 600 ? 3.0 : 2.0);
  final insets =
      safeArea ??
      viewport?.safeArea ??
      (window.shortestSide < 600 ? phoneSafeArea : tabletSafeArea);
  tester.view
    ..devicePixelRatio = devicePixelRatio
    ..physicalSize = window * devicePixelRatio;
  tester.platformDispatcher
    ..textScaleFactorTestValue = 1
    ..accessibilityFeaturesTestValue = const FakeAccessibilityFeatures(
      disableAnimations: true,
      reduceMotion: true,
    );

  final bloc = SeededAppearanceBloc(appearance);
  // Leave the final render mounted for tests that assert against it after
  // this helper returns. Dispose it here, before restoring globals, so the
  // platform override in KansoTestApp unwinds while the widget tree is alive.
  addTearDown(() async {
    debugDisableShadows = true;
    debugDefaultTargetPlatformOverride = null;
    await bloc.close();
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
    // The test binding draws shadows as hard-edged slabs; the pictures
    // should show them as a device does, blurred. The binding checks the
    // flag when the test ends, so it is put back after each render.
    debugDisableShadows = false;
    try {
      final key = ValueKey<String>('golden-${platform.name}');
      await tester.pumpWidget(
        RepaintBoundary(
          key: key,
          child: KansoTestApp(
            theme: KansoTheme.build(
              palette ??
                  (brightness == Brightness.dark
                      ? KansoPalette.kansoDark
                      : KansoPalette.kansoLight),
              fonts: appearance.fonts,
              textSize: appearance.textSize,
              previewTextSize: appearance.previewTextSize,
              interfaceTextSize: appearance.interfaceTextSize,
            ),
            platform: platform,
            builder: (context, app) => MediaQuery(
              data: MediaQuery.of(
                context,
              ).copyWith(padding: insets, viewPadding: insets),
              child: app,
            ),
            home: BlocProvider<AppearanceBloc>.value(
              value: bloc,
              child: child,
            ),
          ),
        ),
      );
      await tester.pumpAndSettle();
      if (interact != null) {
        await interact(tester);
        await tester.pumpAndSettle();
      }
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
        // Clear the iOS tree before mounting Android so the platform override
        // is restored. If the golden mismatches, leave disposal to the test's
        // own teardown; pumping from a failed async matcher deadlocks.
        await tester.pumpWidget(const SizedBox.shrink());
      }
    } finally {
      debugDisableShadows = true;
    }
  }

  final differing = differingPixels(
    renders[TargetPlatform.iOS]!,
    renders[TargetPlatform.android]!,
  );
  expect(
    differing,
    0,
    reason: '$differing pixels differ between the iOS and Android renders',
  );
}

/// How many pixels differ between two RGBA renders of the same size.
int differingPixels(Uint8List a, Uint8List b) {
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
