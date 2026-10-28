// Kanso — graph_canvas_test.dart
// Created 2026-10-28.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/graph/models/data/graph_viewport.dart';
import 'package:kanso/graph/views/widgets/graph_canvas.dart';

import '../../../helpers/golden.dart';

const GraphLayout _layout = GraphLayout(
  nodes: <GraphNodeRow>[
    GraphNodeRow(
      id: 'note-a',
      title: 'First note',
      notebookId: 'book-a',
      degree: 1,
      x: -30,
      y: 0,
      radius: 10,
      colorLight: 0xff7389a6,
      colorDark: 0xff607b9e,
      isCenter: false,
    ),
    GraphNodeRow(
      id: 'note-b',
      title: 'Open note',
      notebookId: 'book-a',
      degree: 2,
      x: 0,
      y: 0,
      radius: 14,
      colorLight: 0xff647d9c,
      colorDark: 0xff4e6d94,
      isCenter: true,
    ),
    GraphNodeRow(
      id: 'note-c',
      title: 'Third note',
      notebookId: 'book-b',
      degree: 1,
      x: 30,
      y: 10,
      radius: 10,
      colorLight: 0xff967d61,
      colorDark: 0xffa68c6d,
      isCenter: false,
    ),
  ],
  edges: <GraphEdgeRow>[
    GraphEdgeRow(from: 0, to: 1),
    GraphEdgeRow(from: 1, to: 2),
  ],
  linkCount: 2,
);

Widget _page(ValueChanged<GraphNodeRow> onSelected) => Scaffold(
  body: GraphCanvas(layout: _layout, onNodeSelected: onSelected),
);

void main() {
  for (final brightness in Brightness.values) {
    testWidgets('graph canvas, ${brightness.name}', (tester) async {
      await expectGoldenOnBothPlatforms(
        tester,
        _page((_) {}),
        name: 'phone_graph_canvas_${brightness.name}',
        size: phoneSize,
        brightness: brightness,
      );
      expect(find.byType(GraphCanvas), findsOneWidget);
    });
  }

  testWidgets('tapping a node selects its note', (tester) async {
    GraphNodeRow? selected;
    await expectGoldenOnBothPlatforms(
      tester,
      _page((node) => selected = node),
      name: 'phone_graph_canvas_tap',
      size: phoneSize,
      brightness: Brightness.light,
      interact: (tester) async {
        selected = null;
        final canvas = tester.getRect(
          find
              .descendant(
                of: find.byType(GraphCanvas),
                matching: find.byType(CustomPaint),
              )
              .first,
        );
        await tester.tapAt(canvas.center);
        expect(selected?.id, 'note-b');
      },
    );
  });

  testWidgets(
    'drag pans the graph without selecting, then transformed tap selects',
    (
      tester,
    ) async {
      GraphNodeRow? selected;
      await expectGoldenOnBothPlatforms(
        tester,
        _page((node) => selected = node),
        name: 'phone_graph_canvas_panned',
        size: phoneSize,
        brightness: Brightness.light,
        interact: (tester) async {
          selected = null;
          final canvas = tester.getRect(find.byType(GraphCanvas));
          final initialViewport = GraphViewport.fit(
            <Offset>[
              for (final node in _layout.nodes) Offset(node.x, node.y),
            ],
            canvas.size,
            margin: 48,
            maxZoom: GraphViewport.maxZoom,
          );
          final start =
              canvas.topLeft +
              initialViewport.toScreen(Offset.zero, canvas.size);
          const delta = Offset(72, 28);

          await tester.dragFrom(start, delta);
          await tester.pumpAndSettle();
          expect(selected, isNull);

          await tester.tapAt(start + delta);
          expect(selected?.id, 'note-b');
        },
      );
    },
  );

  testWidgets('pinch zoom does not select a node', (tester) async {
    GraphNodeRow? selected;
    await expectGoldenOnBothPlatforms(
      tester,
      _page((node) => selected = node),
      name: 'phone_graph_canvas_pinched',
      size: phoneSize,
      brightness: Brightness.light,
      interact: (tester) async {
        final canvas = tester.getRect(
          find
              .descendant(
                of: find.byType(GraphCanvas),
                matching: find.byType(CustomPaint),
              )
              .first,
        );
        final center = canvas.center;
        final first = await tester.startGesture(
          center - const Offset(22, 0),
          pointer: 1,
        );
        final second = await tester.startGesture(
          center + const Offset(22, 0),
          pointer: 2,
        );
        await first.moveTo(center - const Offset(44, 0));
        await second.moveTo(center + const Offset(44, 0));
        await first.up();
        await second.up();
        await tester.pumpAndSettle();

        expect(selected, isNull);
      },
    );
  });
}
