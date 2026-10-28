// Kanso — pump.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/configs/kanso_theme.dart';
import 'package:kanso/gen/l10n/app_localizations.dart';

/// Pumps [child] in the Kanso theme with Kanso's strings, centred.
Future<void> pumpKanso(
  WidgetTester tester,
  Widget child, {
  Brightness brightness = Brightness.light,
  bool settle = true,
}) async {
  await tester.pumpWidget(
    MaterialApp(
      theme: KansoTheme.light,
      darkTheme: KansoTheme.dark,
      themeMode: brightness == Brightness.dark
          ? ThemeMode.dark
          : ThemeMode.light,
      localizationsDelegates: AppLocalizations.localizationsDelegates,
      supportedLocales: AppLocalizations.supportedLocales,
      home: Scaffold(body: Center(child: child)),
    ),
  );
  if (settle) {
    await tester.pumpAndSettle();
  }
}
