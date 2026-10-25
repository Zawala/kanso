// Kanso — kanso_golden_comparator.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/foundation.dart';
import 'package:flutter_test/flutter_test.dart';

/// Compares a render with its golden file, accepting a small difference.
///
/// Goldens are generated on one Mac and compared on another (CI), and text
/// is rasterised slightly differently from machine to machine: about 0.06%
/// of the pixels drift, along the edges of glyphs. A change a person would
/// see moves far more than [tolerance] and still fails, with the usual
/// failure images. The iOS and Android renders are compared with each other
/// separately, with no tolerance (see `expectGoldenOnBothPlatforms`).
class KansoGoldenComparator extends LocalFileComparator {
  KansoGoldenComparator(super.testFile);

  /// The share of pixels that may differ from the golden file: 0.5%.
  static const double tolerance = 0.005;

  @override
  Future<bool> compare(Uint8List imageBytes, Uri golden) async {
    final result = await GoldenFileComparator.compareLists(
      imageBytes,
      await getGoldenBytes(golden),
    );
    if (result.passed || result.diffPercent <= tolerance) {
      return true;
    }
    final error = await generateFailureOutput(result, golden, basedir);
    throw FlutterError(error);
  }
}
