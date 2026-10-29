// Kanso — graph_canvas_test.dart
// Created 2026-10-28.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:math' as math;

import 'package:flutter/material.dart';
import 'package:flutter/semantics.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/configs/kanso_theme.dart';
import 'package:kanso/graph/models/data/graph_viewport.dart';
import 'package:kanso/graph/views/widgets/graph_canvas.dart';

import '../../../helpers/golden.dart';
import '../../../helpers/kanso_test_app.dart';

final GraphLayout _denseLayout = _makeDenseLayout();

GraphLayout _makeDenseLayout() {
  const clusterCount = 8;
  const perCluster = 9;
  final nodes = <GraphNodeRow>[];
  final edges = <GraphEdgeRow>[];
  final colors = <int>[
    0xff7389a6,
    0xff647d9c,
    0xff967d61,
    0xff6f8f7c,
    0xff8b758f,
    0xffa17d73,
    0xff6f90a0,
    0xff9b8a64,
  ];

  for (var cluster = 0; cluster < clusterCount; cluster++) {
    final clusterAngle = 2 * math.pi * cluster / clusterCount;
    final clusterCenter = Offset(
      math.cos(clusterAngle) * 360,
      math.sin(clusterAngle) * 420,
    );
    final first = nodes.length;
    for (var member = 0; member < perCluster; member++) {
      final angle = 2 * math.pi * member / perCluster;
      final radius = member == 0 ? 0.0 : 54.0;
      final noteNumber = cluster * perCluster + member;
      nodes.add(
        GraphNodeRow(
          id: 'dense-note-$noteNumber',
          title: 'Note ${noteNumber + 1}',
          notebookId: 'dense-book-$cluster',
          degree: member == 0 ? perCluster + 1 : 3,
          x: clusterCenter.dx + math.cos(angle) * radius,
          y: clusterCenter.dy + math.sin(angle) * radius,
          radius: member == 0 ? 12 : 6,
          colorLight: colors[cluster],
          colorDark: colors[cluster],
          isCenter: noteNumber == 0,
        ),
      );
    }
    // Each cluster is a hub-and-ring, with the hubs linked around the map.
    for (var member = 1; member < perCluster; member++) {
      edges
        ..add(GraphEdgeRow(from: first, to: first + member))
        ..add(
          GraphEdgeRow(
            from: first + member,
            to: first + (member == perCluster - 1 ? 1 : member + 1),
          ),
        );
    }
    if (cluster > 0) {
      final previous = (cluster - 1) * perCluster;
      edges.add(GraphEdgeRow(from: previous, to: first));
    }
  }
  // Close the hub ring and add a few long links so the dense case includes
  // the strands that make a graph look tangled on a small phone.
  edges.add(
    const GraphEdgeRow(from: (clusterCount - 1) * perCluster, to: 0),
  );
  for (var index = 0; index < 20; index++) {
    final from = (index * 7) % nodes.length;
    final to = (index * 13 + 11) % nodes.length;
    if (from != to) edges.add(GraphEdgeRow(from: from, to: to));
  }
  return GraphLayout(
    nodes: nodes,
    edges: edges,
    linkCount: edges.length,
  );
}

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

Future<void> _pumpInteractiveCanvas(
  WidgetTester tester,
  ValueChanged<GraphNodeRow> onSelected,
) async {
  tester.view
    ..devicePixelRatio = 3
    ..physicalSize = phoneSize * 3;
  addTearDown(tester.view.reset);
  await tester.pumpWidget(
    KansoTestApp(theme: KansoTheme.light, home: _page(onSelected)),
  );
  await tester.pumpAndSettle();
}

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

  testWidgets('dense graph, light', (tester) async {
    await expectGoldenOnBothPlatforms(
      tester,
      Scaffold(
        body: GraphCanvas(layout: _denseLayout, onNodeSelected: (_) {}),
      ),
      name: 'phone_graph_canvas_dense',
      size: phoneSize,
      brightness: Brightness.light,
    );
    expect(find.byType(GraphCanvas), findsOneWidget);
  });

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
        await tester.pump(const Duration(milliseconds: 350));
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
          await tester.pump(const Duration(milliseconds: 350));
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

  testWidgets('double tap zooms around the local tap point', (tester) async {
    GraphNodeRow? selected;
    await _pumpInteractiveCanvas(tester, (node) => selected = node);

    final canvas = tester.getRect(find.byType(GraphCanvas));
    final initialViewport = GraphViewport.fit(
      <Offset>[
        for (final node in _layout.nodes) Offset(node.x, node.y),
      ],
      canvas.size,
      margin: 48,
      maxZoom: GraphViewport.maxZoom,
    );
    final focalPoint =
        canvas.topLeft +
        initialViewport.toScreen(const Offset(-30, 0), canvas.size);
    await tester.tapAt(focalPoint);
    await tester.pump(const Duration(milliseconds: 40));
    await tester.tapAt(focalPoint);
    await tester.pump(const Duration(milliseconds: 400));
    await tester.pumpAndSettle();
    selected = null;
    await tester.tapAt(focalPoint);
    await tester.pump(const Duration(milliseconds: 450));

    expect(selected?.id, 'note-a');
  });

  testWidgets('nodes expose their titles as accessible actions', (
    tester,
  ) async {
    final semantics = tester.ensureSemantics();
    try {
      await _pumpInteractiveCanvas(tester, (_) {});
      final root = tester.getSemantics(find.byType(CustomPaint).first);
      final labels = <String>[];
      bool visit(SemanticsNode node) {
        final label = node.getSemanticsData().label;
        if (label.isNotEmpty) labels.add(label);
        node.visitChildren(visit);
        return true;
      }

      visit(root);
      expect(
        labels,
        containsAll(<String>[
          'First note',
          'Open note',
          'Third note',
        ]),
      );
      expect(
        root.getSemanticsData().label,
        isNot(equals('Open note')),
      );
    } finally {
      semantics.dispose();
    }
  });

  testWidgets('zoom controls expose accessible in, out, and fit actions', (
    tester,
  ) async {
    final semantics = tester.ensureSemantics();
    try {
      await _pumpInteractiveCanvas(tester, (_) {});

      expect(find.bySemanticsLabel('Zoom in'), findsOneWidget);
      expect(find.bySemanticsLabel('Zoom out'), findsOneWidget);
      expect(find.bySemanticsLabel('Fit graph to view'), findsOneWidget);

      await tester.tap(find.bySemanticsLabel('Zoom in'));
      await tester.pumpAndSettle();
      await tester.tap(find.bySemanticsLabel('Zoom out'));
      await tester.pumpAndSettle();
      await tester.tap(find.bySemanticsLabel('Fit graph to view'));
      await tester.pumpAndSettle();
    } finally {
      semantics.dispose();
    }
  });
}
