// Kanso — perf_driver.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

// This is a command-line tool; its output is printing.
// ignore_for_file: avoid_print

import 'dart:convert';
import 'dart:io';

import 'package:integration_test/integration_test_driver.dart';

/// Where the report of the scrolling check is written, from `mobile/`.
const String reportPath = 'build/scroll_performance.json';

/// The driver of `integration_test/scroll_performance_test.dart` for
/// `flutter drive`: it prints the test's frame report and writes it to
/// [reportPath]. The test itself decides whether the numbers pass.
///
/// ```sh
/// flutter drive --profile \
///   --driver=test_driver/perf_driver.dart \
///   --target=integration_test/scroll_performance_test.dart \
///   -d <device>
/// ```
Future<void> main() {
  return integrationDriver(
    // A failing run's numbers are the ones to read.
    writeResponseOnFailure: true,
    responseDataCallback: (data) async {
      final report = data?['frame_report'];
      if (report is! Map<String, dynamic>) {
        print('The test did not report any frames.');
        return;
      }
      File(reportPath)
        ..createSync(recursive: true)
        ..writeAsStringSync(const JsonEncoder.withIndent('  ').convert(report));
      String ms(Object? value) => (value! as num).toStringAsFixed(2);
      print('Scrolling the note list');
      print('  frames           ${report['frames']}');
      print('  refresh rate     ${report['refreshRate']} Hz');
      print('  budget           ${ms(report['budgetMs'])} ms');
      print(
        '  build  p50/p90/p99  ${ms(report['buildP50'])} / '
        '${ms(report['buildP90'])} / ${ms(report['buildP99'])} ms',
      );
      print(
        '  raster p50/p90/p99  ${ms(report['rasterP50'])} / '
        '${ms(report['rasterP90'])} / ${ms(report['rasterP99'])} ms',
      );
      print(
        '  over the budget  ${report['framesOverBudget']} frames '
        '(${((report['overBudgetShare']! as num) * 1000).round() / 10}%)',
      );
      print('  written to       $reportPath');
    },
  );
}
