// Kanso — resize_viewport.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';

import 'kanso_viewport.dart';

/// Gives the window the size, density and safe area of [viewport], as a
/// rotation or a change of a Split View does, and lets the app lay itself out
/// again.
Future<void> resizeToViewport(
  WidgetTester tester,
  KansoViewport viewport,
) async {
  final ratio = viewport.devicePixelRatio;
  final safeArea = viewport.safeArea;
  final padding = FakeViewPadding(
    left: safeArea.left * ratio,
    top: safeArea.top * ratio,
    right: safeArea.right * ratio,
    bottom: safeArea.bottom * ratio,
  );
  tester.view
    ..devicePixelRatio = ratio
    ..physicalSize = viewport.physicalSize
    ..padding = padding
    ..viewPadding = padding;
  await tester.pumpAndSettle();
}
