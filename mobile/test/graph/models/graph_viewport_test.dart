// Kanso — graph_viewport_test.dart
// Created 2026-10-29.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/graph/models/data/graph_viewport.dart';
import 'package:kanso/graph/views/widgets/graph_canvas.dart';

void main() {
  const size = Size(400, 300);

  test('fit centers all positions inside the viewport margin', () {
    final viewport = GraphViewport.fit(
      const <Offset>[Offset(-100, -50), Offset(100, 50)],
      size,
      margin: 40,
      maxZoom: 2,
    );

    expect(viewport.center, Offset.zero);
    expect(viewport.zoom, 1.6);
    expect(
      viewport.toScreen(const Offset(-100, -50), size),
      const Offset(40, 70),
    );
    expect(
      viewport.toScreen(const Offset(100, 50), size),
      const Offset(360, 230),
    );
  });

  test('zooming around an anchor keeps its graph point fixed', () {
    const viewport = GraphViewport(center: Offset(12, -8), zoom: 1.2);
    const anchor = Offset(90, 210);
    final graphPoint = viewport.toLayout(anchor, size);
    final zoomed = viewport.zoomAt(1.75, anchor, size);

    expect(zoomed.toScreen(graphPoint, size).dx, closeTo(anchor.dx, 1e-9));
    expect(zoomed.toScreen(graphPoint, size).dy, closeTo(anchor.dy, 1e-9));
    expect(zoomed.zoom, 2.1);
  });

  test('pan moves the visible graph opposite the finger delta', () {
    const viewport = GraphViewport(center: Offset(4, 7), zoom: 2);
    final panned = viewport.panned(const Offset(30, -10));

    expect(panned.center, const Offset(-11, 12));
    expect(panned.zoom, 2);
  });

  test('zoom clamps to the requested minimum and maximum', () {
    const viewport = GraphViewport();

    expect(
      viewport.zoomAt(0.001, Offset.zero, size).zoom,
      GraphViewport.minZoom,
    );
    expect(viewport.zoomAt(100, Offset.zero, size).zoom, GraphViewport.maxZoom);
  });

  test(
    'hit testing applies the viewport transform and chooses nearest node',
    () {
      const viewport = GraphViewport(center: Offset(10, 5), zoom: 2);
      const positions = <Offset>[Offset(12, 5), Offset(13, 5)];
      final point = viewport.toScreen(const Offset(13, 5), size);

      expect(
        graphNodeAt(positions, const <double>[4, 4], viewport, size, point),
        1,
      );
      expect(
        graphNodeAt(
          positions,
          const <double>[4, 4],
          viewport,
          size,
          const Offset(2, 2),
        ),
        isNull,
      );
    },
  );
}
