// Kanso — kanso_viewport_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/configs/kanso_layout.dart';
import 'package:kanso/core/models/data/layout_class.dart';

import 'kanso_viewport.dart';

void main() {
  group('KansoViewport', () {
    for (final viewport in KansoViewport.values) {
      test('${viewport.name} has room for ${viewport.layout.name}', () {
        expect(KansoLayout.classFor(viewport.size), viewport.layout);
      });
    }

    test('the matrix has every layout class, on a tablet too', () {
      final layouts = <LayoutClass>{
        for (final viewport in KansoViewport.values) viewport.layout,
      };
      expect(layouts, LayoutClass.values.toSet());
      final tabletSingles = KansoViewport.values.where(
        (viewport) => viewport.tablet && viewport.layout == LayoutClass.single,
      );
      expect(tabletSingles, isNotEmpty);
    });

    test('a tablet window is a tablet by the display, not the window', () {
      // The notifier asks the display; a Split View half is narrow but
      // sits on a 820 x 1180 or larger screen.
      for (final viewport in KansoViewport.values) {
        if (viewport.tablet) {
          expect(
            viewport.size.height,
            greaterThanOrEqualTo(KansoLayout.tabletShortestSide),
            reason: '${viewport.name} is a window of a tablet',
          );
        }
      }
    });

    test('the safe area fits in the window', () {
      for (final viewport in KansoViewport.values) {
        expect(
          viewport.safeArea.vertical,
          lessThan(viewport.size.height / 4),
          reason: viewport.name,
        );
      }
    });

    test('physical sizes are whole pixels', () {
      for (final viewport in KansoViewport.values) {
        expect(viewport.physicalSize.width % 1, 0, reason: viewport.name);
        expect(viewport.physicalSize.height % 1, 0, reason: viewport.name);
      }
    });
  });
}
