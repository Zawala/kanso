// Kanso — store_screenshot_driver.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

// This is a command-line tool; its output is printing.
// ignore_for_file: avoid_print

import 'dart:io';

import 'package:integration_test/integration_test_driver_extended.dart';

/// The driver of `integration_test/store_screenshots_test.dart` for
/// `flutter drive`: it writes each picture the test takes to
/// `$KANSO_SCREENSHOT_DIR/<name>.png`. `scripts/mobile-store-screenshots.sh`
/// sets the folder and runs it.
Future<void> main() {
  final folder = Platform.environment['KANSO_SCREENSHOT_DIR'];
  if (folder == null || folder.isEmpty) {
    stderr.writeln('Set KANSO_SCREENSHOT_DIR to the folder for the pictures.');
    exit(2);
  }
  return integrationDriver(
    onScreenshot: (name, image, [args]) async {
      final file = File('$folder/$name.png')
        ..createSync(recursive: true)
        ..writeAsBytesSync(image);
      print('  ${file.path}');
      return true;
    },
  );
}
