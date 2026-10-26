// Kanso — kanso_layout_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:ui';

import 'package:flutter_test/flutter_test.dart';
import 'package:glados/glados.dart' show Glados2, Glados3, IntAnys, any;
import 'package:kanso/core/configs/kanso_layout.dart';
import 'package:kanso/core/configs/kanso_sizes.dart';
import 'package:kanso/core/models/data/layout_class.dart';

import '../../helpers/property.dart';

void main() {
  group('KansoLayout', () {
    test('has the breakpoints of the pane widths', () {
      expect(KansoLayout.doubleMinWidth, 740);
      expect(KansoLayout.tripleMinWidth, 1020);
      expect(KansoLayout.noteMinWidth, 400);
    });

    test('lays out the devices it was designed for', () {
      const cases = <String, (Size, LayoutClass)>{
        'iPhone 15': (Size(393, 852), LayoutClass.single),
        'iPhone 15 landscape': (Size(852, 393), LayoutClass.single),
        'iPhone Pro Max landscape': (Size(932, 430), LayoutClass.single),
        'iPad mini portrait': (Size(744, 1133), LayoutClass.double),
        'iPad 11 portrait': (Size(834, 1194), LayoutClass.double),
        'iPad 11 landscape': (Size(1194, 834), LayoutClass.triple),
        'iPad 13 portrait': (Size(1024, 1366), LayoutClass.triple),
        'iPad 13 landscape': (Size(1366, 1024), LayoutClass.triple),
        'half Split View': (Size(590, 1024), LayoutClass.single),
        'two thirds Split View': (Size(694, 1024), LayoutClass.single),
        'tablet design, landscape': (Size(1180, 820), LayoutClass.triple),
        'tablet design, portrait': (Size(820, 1180), LayoutClass.double),
        'Android tablet portrait': (Size(800, 1280), LayoutClass.double),
        'foldable open': (Size(673, 841), LayoutClass.single),
      };
      for (final entry in cases.entries) {
        expect(
          KansoLayout.classFor(entry.value.$1),
          entry.value.$2,
          reason: entry.key,
        );
      }
    });

    Glados2<int, int>(
      any.intInRange(100, 1700),
      any.intInRange(100, 1700),
      kansoExploreConfig(),
    ).test('a phone is always one pane', (a, b) {
      final size = Size(a.toDouble(), b.toDouble());
      if (size.shortestSide < KansoLayout.tabletShortestSide) {
        expect(KansoLayout.classFor(size), LayoutClass.single);
      }
    });

    Glados3<int, int, int>(
      any.intInRange(100, 1700),
      any.intInRange(100, 1700),
      any.intInRange(100, 1700),
      kansoExploreConfig(),
    ).test('a wider window never has fewer panes', (a, b, height) {
      final narrow = a < b ? a : b;
      final wide = a < b ? b : a;
      final fewer = KansoLayout.classFor(
        Size(narrow.toDouble(), height.toDouble()),
      );
      final more = KansoLayout.classFor(
        Size(wide.toDouble(), height.toDouble()),
      );
      expect(more.index, greaterThanOrEqualTo(fewer.index));
    });

    Glados2<int, int>(
      any.intInRange(100, 1700),
      any.intInRange(100, 1700),
      kansoExploreConfig(),
    ).test('the panes shown fit, and the note pane is at least 400', (w, h) {
      final size = Size(w.toDouble(), h.toDouble());
      final layout = KansoLayout.classFor(size);
      final note = KansoLayout.noteWidth(size.width, layout);
      expect(note, greaterThan(0));
      switch (layout) {
        case LayoutClass.single:
          expect(note, size.width);
        case LayoutClass.double:
          expect(note, greaterThanOrEqualTo(KansoLayout.noteMinWidth));
          expect(note + KansoSizes.listPane, size.width);
        case LayoutClass.triple:
          expect(note, greaterThanOrEqualTo(KansoLayout.noteMinWidth));
          expect(
            note + KansoSizes.listPane + KansoSizes.libraryPane,
            size.width,
          );
      }
    });

    Glados2<int, int>(
      any.intInRange(100, 1700),
      any.intInRange(100, 1700),
      kansoExploreConfig(),
    ).test('a tablet window has as many panes as fit', (w, h) {
      final size = Size(w.toDouble(), h.toDouble());
      if (size.shortestSide < KansoLayout.tabletShortestSide) {
        return;
      }
      final layout = KansoLayout.classFor(size);
      expect(
        layout == LayoutClass.triple,
        size.width >= KansoLayout.tripleMinWidth,
      );
      expect(
        layout != LayoutClass.single,
        size.width >= KansoLayout.doubleMinWidth,
      );
    });
  });
}
