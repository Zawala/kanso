// Kanso — flutter_test_config.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';
import 'dart:io';

import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';

/// Loads the bundled fonts from their files before any test runs, so text
/// in widget tests and goldens is drawn in Kanso's fonts rather than the
/// test harness's placeholder font.
Future<void> testExecutable(FutureOr<void> Function() testMain) async {
  TestWidgetsFlutterBinding.ensureInitialized();
  await _loadFamily('Instrument Sans', <String>[
    'InstrumentSans-Regular.ttf',
    'InstrumentSans-Medium.ttf',
    'InstrumentSans-SemiBold.ttf',
    'InstrumentSans-Bold.ttf',
    'InstrumentSans-Italic.ttf',
  ]);
  await _loadFamily('JetBrains Mono', <String>[
    'JetBrainsMono-Regular.ttf',
    'JetBrainsMono-Bold.ttf',
  ]);
  await testMain();
}

Future<void> _loadFamily(String family, List<String> files) async {
  final loader = FontLoader(family);
  for (final file in files) {
    final bytes = await File('assets/fonts/$file').readAsBytes();
    loader.addFont(Future<ByteData>.value(ByteData.sublistView(bytes)));
  }
  await loader.load();
}
