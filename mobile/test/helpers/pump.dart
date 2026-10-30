// Kanso — pump.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import 'kanso_test_app.dart';

/// Pumps [child] in the Kanso theme with Kanso's strings, centred.
Future<void> pumpKanso(
  WidgetTester tester,
  Widget child, {
  Brightness brightness = Brightness.light,
  bool settle = true,
}) async {
  await tester.pumpWidget(
    KansoTestApp(
      brightness: brightness,
      home: Center(child: child),
    ),
  );
  if (settle) {
    await tester.pumpAndSettle();
  }
}
