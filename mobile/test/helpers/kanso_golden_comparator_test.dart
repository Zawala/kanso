// Kanso — kanso_golden_comparator_test.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:typed_data';
import 'dart:ui' as ui;

import 'package:flutter_test/flutter_test.dart';

import 'kanso_golden_comparator.dart';

/// A 10x10 PNG filled with [fill], with [changed] pixels set to [mark].
Future<Uint8List> png(
  int fill, {
  int changed = 0,
  int mark = 0xFF000000,
}) async {
  const size = 10;
  final pixels = Uint32List(size * size)
    ..fillRange(0, size * size, fill)
    ..fillRange(0, changed, mark);
  final buffer = await ui.ImmutableBuffer.fromUint8List(
    pixels.buffer.asUint8List(),
  );
  final descriptor = ui.ImageDescriptor.raw(
    buffer,
    width: size,
    height: size,
    pixelFormat: ui.PixelFormat.rgba8888,
  );
  final codec = await descriptor.instantiateCodec();
  final image = (await codec.getNextFrame()).image;
  final data = await image.toByteData(format: ui.ImageByteFormat.png);
  return data!.buffer.asUint8List();
}

void main() {
  group('KansoGoldenComparator.changedPixelShare', () {
    test('identical images change nothing', () async {
      final a = await png(0xFF808080);
      expect(await KansoGoldenComparator.changedPixelShare(a, a), 0);
    });

    test('anti-aliasing-sized drift on every pixel is ignored', () async {
      final a = await png(0xFF808080);
      final b = await png(0xFF909090);
      expect(await KansoGoldenComparator.changedPixelShare(a, b), 0);
    });

    test('a visible change counts every pixel it touches', () async {
      final a = await png(0xFFFFFFFF);
      final b = await png(0xFFFFFFFF, changed: 3);
      expect(await KansoGoldenComparator.changedPixelShare(a, b), 0.03);
    });

    test('images of different sizes never match', () async {
      final a = await png(0xFFFFFFFF);
      final codec = await ui.instantiateImageCodec(a, targetWidth: 5);
      final image = (await codec.getNextFrame()).image;
      final data = await image.toByteData(format: ui.ImageByteFormat.png);
      final b = data!.buffer.asUint8List();
      expect(await KansoGoldenComparator.changedPixelShare(a, b), isNull);
    });
  });
}
