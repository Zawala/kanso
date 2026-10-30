// Kanso — bundled_font_repository_test.dart
// Created 2026-10-30.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/models/data/kanso_font_family.dart';
import 'package:kanso/core/models/repos/impl/asset_bundled_font_repository.dart';

class CountingBundle extends CachingAssetBundle {
  int loads = 0;

  @override
  Future<ByteData> load(String key) {
    loads++;
    return rootBundle.load(key);
  }
}

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  group('AssetBundledFontRepository', () {
    test('every bundled family has its files, and each is shipped', () async {
      final fonts = AssetBundledFontRepository();
      for (final font in KansoFontFamily.values) {
        final family = font.family;
        if (family == null) {
          continue;
        }
        final faces = await fonts.faces(family);
        expect(faces, isNotEmpty, reason: family);
        for (final face in faces) {
          // A TrueType file starts 00 01 00 00.
          expect(face.bytes.take(4), <int>[0, 1, 0, 0], reason: family);
        }
      }
    });

    test('a family the app does not ship has no faces', () async {
      expect(await AssetBundledFontRepository().faces('Lora'), isEmpty);
    });

    test('reads a family once', () async {
      final bundle = CountingBundle();
      final fonts = AssetBundledFontRepository.from(bundle);
      await fonts.faces('Inter');
      await fonts.faces('Inter');
      expect(bundle.loads, 1);
    });
  });
}
