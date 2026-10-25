// Kanso — kanso_golden_comparator.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:ui' as ui;

import 'package:flutter/foundation.dart';
import 'package:flutter_test/flutter_test.dart';

/// Compares a render with its golden file, ignoring anti-aliasing.
///
/// Goldens are generated on one Mac and compared on another (CI), and text
/// is rasterised slightly differently from machine to machine: up to about
/// 1% of the pixels drift along the edges of glyphs, nearly all by a few
/// levels of a channel. So a pixel only counts as changed when a channel
/// moves by more than [channelTolerance], and at most [pixelTolerance] of
/// the pixels may change. A change a person would see (layout, a colour,
/// a missing glyph) moves far more and still fails, with the usual failure
/// images. The iOS and Android renders are compared with each other
/// separately, with no tolerance (see `expectGoldenOnBothPlatforms`).
class KansoGoldenComparator extends LocalFileComparator {
  KansoGoldenComparator(super.testFile);

  /// How far, out of 255, a channel may move before its pixel counts as
  /// changed. Measured drift between Macs stays under 80 and mostly under 16.
  static const int channelTolerance = 32;

  /// The share of pixels that may change: 0.3%. Text in a variable font
  /// (the serif preview) drifts most, about 0.13% between Macs by this rule;
  /// static fonts about 0.05%.
  static const double pixelTolerance = 0.003;

  @override
  Future<bool> compare(Uint8List imageBytes, Uri golden) async {
    final goldenBytes = Uint8List.fromList(await getGoldenBytes(golden));
    final result = await GoldenFileComparator.compareLists(
      imageBytes,
      goldenBytes,
    );
    if (result.passed) {
      return true;
    }
    final changed = await changedPixelShare(imageBytes, goldenBytes);
    if (changed != null && changed <= pixelTolerance) {
      return true;
    }
    final error = await generateFailureOutput(result, golden, basedir);
    throw FlutterError(error);
  }

  /// The share of pixels where a channel differs by more than
  /// [channelTolerance], or null when the images differ in size.
  static Future<double?> changedPixelShare(Uint8List a, Uint8List b) async {
    final first = await _rgba(a);
    final second = await _rgba(b);
    if (first.width != second.width || first.height != second.height) {
      return null;
    }
    var changed = 0;
    for (var i = 0; i < first.bytes.length; i += 4) {
      for (var channel = 0; channel < 4; channel++) {
        final delta = first.bytes[i + channel] - second.bytes[i + channel];
        if (delta.abs() > channelTolerance) {
          changed++;
          break;
        }
      }
    }
    return changed / (first.width * first.height);
  }

  static Future<({int width, int height, Uint8List bytes})> _rgba(
    Uint8List png,
  ) async {
    final codec = await ui.instantiateImageCodec(png);
    final frame = await codec.getNextFrame();
    final image = frame.image;
    final data = await image.toByteData();
    final decoded = (
      width: image.width,
      height: image.height,
      bytes: data!.buffer.asUint8List(),
    );
    image.dispose();
    codec.dispose();
    return decoded;
  }
}
