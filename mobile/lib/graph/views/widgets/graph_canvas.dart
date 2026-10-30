// Kanso — graph_canvas.dart
// Created 2026-10-29.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:math' as math;

import 'package:flutter/material.dart';
import 'package:flutter/semantics.dart';

import '../../../core/bridge/api/types.dart';
import '../../../core/configs/kanso_icons.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_icon_button.dart';
import '../../models/data/graph_viewport.dart';

/// A calm, pannable map of linked notes.
class GraphCanvas extends StatefulWidget {
  const GraphCanvas({
    required this.layout,
    required this.onNodeSelected,
    super.key,
  });

  final GraphLayout layout;
  final ValueChanged<GraphNodeRow> onNodeSelected;

  @override
  State<GraphCanvas> createState() => _GraphCanvasState();
}

class _GraphCanvasState extends State<GraphCanvas> {
  final ValueNotifier<GraphViewport> _viewport = ValueNotifier<GraphViewport>(
    const GraphViewport(),
  );
  bool _needsFit = true;
  Size? _lastSize;
  double _lastScale = 1;
  Offset? _doubleTapPosition;

  @override
  void didUpdateWidget(GraphCanvas oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (oldWidget.layout != widget.layout) _needsFit = true;
  }

  @override
  void dispose() {
    _viewport.dispose();
    super.dispose();
  }

  List<Offset> get _positions => <Offset>[
    for (final node in widget.layout.nodes) Offset(node.x, node.y),
  ];

  void _fit(Size size) {
    _viewport.value = GraphViewport.fit(
      _positions,
      size,
      margin: 48,
      maxZoom: GraphViewport.maxZoom,
    );
    _needsFit = false;
  }

  Rect get _layoutBounds => graphExtent(_positions) ?? Rect.zero;

  void _zoom(double factor, Offset focalPoint, Size size) {
    _viewport.value = _viewport.value
        .zoomAt(factor, focalPoint, size)
        .keptOver(_layoutBounds);
  }

  void _onScaleStart(ScaleStartDetails details) {
    _lastScale = 1;
  }

  void _onScaleUpdate(ScaleUpdateDetails details, Size size) {
    final factor = details.scale / _lastScale;
    _lastScale = details.scale;
    _viewport.value = _viewport.value.updatedForGesture(
      pointerCount: details.pointerCount,
      scaleFactor: factor,
      focalPoint: details.localFocalPoint,
      focalDelta: details.focalPointDelta,
      size: size,
      bounds: _layoutBounds,
    );
  }

  void _selectAt(Offset point, Size size, GraphViewport viewport) {
    final positions = _positions;
    final index = graphNodeAt(
      positions,
      <double>[for (final node in widget.layout.nodes) node.radius],
      viewport,
      size,
      point,
    );
    if (index != null) widget.onNodeSelected(widget.layout.nodes[index]);
  }

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
    final textStyle = context.typography.secondary;
    final dark = context.isDark;

    return LayoutBuilder(
      builder: (context, constraints) {
        final size = Size(constraints.maxWidth, constraints.maxHeight);
        if (_lastSize != size) {
          _lastSize = size;
          _needsFit = true;
        }
        if (_needsFit && size.width.isFinite && size.height.isFinite) {
          _fit(size);
        }
        return ClipRRect(
          borderRadius: BorderRadius.circular(18),
          child: DecoratedBox(
            decoration: BoxDecoration(
              color: colors.editor,
              border: Border.all(color: colors.divider.withValues(alpha: 0.72)),
              borderRadius: BorderRadius.circular(18),
            ),
            child: Stack(
              fit: StackFit.expand,
              children: <Widget>[
                ValueListenableBuilder<GraphViewport>(
                  valueListenable: _viewport,
                  builder: (context, viewport, _) => GestureDetector(
                    behavior: HitTestBehavior.opaque,
                    onScaleStart: _onScaleStart,
                    onScaleUpdate: (details) => _onScaleUpdate(details, size),
                    onDoubleTapDown: (details) {
                      _doubleTapPosition = details.localPosition;
                    },
                    onDoubleTap: () {
                      final focalPoint = _doubleTapPosition;
                      if (focalPoint != null) _zoom(1.5, focalPoint, size);
                    },
                    onTapUp: (details) => _selectAt(
                      details.localPosition,
                      size,
                      viewport,
                    ),
                    child: CustomPaint(
                      painter: _GraphPainter(
                        layout: widget.layout,
                        viewport: viewport,
                        textColor: colors.text,
                        mutedColor: colors.textSecondary,
                        edgeColor: colors.textMuted,
                        surfaceColor: colors.editor,
                        accentColor: colors.accent,
                        textStyle: textStyle,
                        textDirection: Directionality.of(context),
                        untitled: context.l10n.noteUntitled,
                        dark: dark,
                        onNodeSelected: widget.onNodeSelected,
                      ),
                      size: size,
                    ),
                  ),
                ),
                Positioned(
                  top: 10,
                  right: 10,
                  child: DecoratedBox(
                    decoration: BoxDecoration(
                      color: colors.elevated.withValues(alpha: 0.94),
                      borderRadius: BorderRadius.circular(14),
                    ),
                    child: Column(
                      mainAxisSize: MainAxisSize.min,
                      children: <Widget>[
                        KansoIconButton(
                          icon: KansoIcons.plus,
                          semanticLabel: context.l10n.graphZoomIn,
                          onPressed: () =>
                              _zoom(1.25, size.center(Offset.zero), size),
                        ),
                        KansoIconButton(
                          icon: KansoIcons.minus,
                          semanticLabel: context.l10n.graphZoomOut,
                          onPressed: () =>
                              _zoom(0.8, size.center(Offset.zero), size),
                        ),
                        KansoIconButton(
                          icon: KansoIcons.target,
                          semanticLabel: context.l10n.graphFit,
                          onPressed: () => _fit(size),
                        ),
                      ],
                    ),
                  ),
                ),
              ],
            ),
          ),
        );
      },
    );
  }
}

/// Returns the closest node under a screen point after applying [viewport].
int? graphNodeAt(
  List<Offset> positions,
  List<double> radii,
  GraphViewport viewport,
  Size size,
  Offset point, {
  double slop = 12,
}) {
  int? nearest;
  var nearestDistance = double.infinity;
  for (var i = 0; i < positions.length; i++) {
    final center = viewport.toScreen(positions[i], size);
    final distance = (center - point).distance;
    final reach = screenRadius(radii[i], viewport.zoom) + slop;
    if (distance <= reach && distance < nearestDistance) {
      nearest = i;
      nearestDistance = distance;
    }
  }
  return nearest;
}

class _GraphPainter extends CustomPainter {
  const _GraphPainter({
    required this.layout,
    required this.viewport,
    required this.textColor,
    required this.mutedColor,
    required this.edgeColor,
    required this.surfaceColor,
    required this.accentColor,
    required this.textStyle,
    required this.textDirection,
    required this.untitled,
    required this.dark,
    required this.onNodeSelected,
  });

  final GraphLayout layout;
  final GraphViewport viewport;
  final Color textColor;
  final Color mutedColor;
  final Color edgeColor;
  final Color surfaceColor;
  final Color accentColor;
  final TextStyle textStyle;
  final TextDirection textDirection;
  final String untitled;
  final bool dark;
  final ValueChanged<GraphNodeRow> onNodeSelected;

  @override
  SemanticsBuilderCallback get semanticsBuilder {
    return (size) {
      final semantics = <CustomPainterSemantics>[];
      for (final node in layout.nodes) {
        final center = viewport.toScreen(Offset(node.x, node.y), size);
        final radius = math.max<double>(
          screenRadius(node.radius, viewport.zoom) + 12,
          22,
        );
        final rect = Rect.fromCircle(center: center, radius: radius);
        if (!rect.overlaps(Offset.zero & size)) continue;
        final title = node.title.trim().isEmpty ? untitled : node.title.trim();
        semantics.add(
          CustomPainterSemantics(
            key: ValueKey<String>(node.id),
            rect: rect,
            properties: SemanticsProperties(
              label: title,
              button: true,
              textDirection: textDirection,
              onTap: () => onNodeSelected(node),
            ),
          ),
        );
      }
      return semantics;
    };
  }

  @override
  void paint(Canvas canvas, Size size) {
    if (layout.nodes.isEmpty) return;
    final positions = <Offset>[
      for (final node in layout.nodes) Offset(node.x, node.y),
    ];
    final visible = viewport.visible(size, 24);
    final points = <Offset>[
      for (final position in positions) viewport.toScreen(position, size),
    ];

    final edgePaint = Paint()
      ..color = edgeColor.withValues(alpha: dark ? 0.09 : 0.12)
      ..strokeWidth = viewport.zoom < 0.35 ? 0.8 : 1
      ..strokeCap = StrokeCap.round;
    for (final edge in layout.edges) {
      if (edge.from < 0 ||
          edge.from >= positions.length ||
          edge.to < 0 ||
          edge.to >= positions.length ||
          edge.from == edge.to ||
          !segmentVisible(positions[edge.from], positions[edge.to], visible)) {
        continue;
      }
      canvas.drawLine(points[edge.from], points[edge.to], edgePaint);
    }

    final labelCandidates = <_LabelCandidate>[];
    for (var i = 0; i < layout.nodes.length; i++) {
      final node = layout.nodes[i];
      final radius = screenRadius(node.radius, viewport.zoom);
      final center = points[i];
      if (center.dx < -radius ||
          center.dx > size.width + radius ||
          center.dy < -radius ||
          center.dy > size.height + radius) {
        continue;
      }
      final nodeColor = Color(dark ? node.colorDark : node.colorLight);
      if (node.isCenter) {
        canvas.drawCircle(
          center,
          radius + 6,
          Paint()..color = accentColor.withValues(alpha: 0.12),
        );
      }
      canvas
        ..drawCircle(
          center,
          radius + 1.4,
          Paint()..color = surfaceColor,
        )
        ..drawCircle(center, radius, Paint()..color = nodeColor);
      if (node.isCenter) {
        canvas.drawCircle(
          center,
          radius + 1.5,
          Paint()
            ..color = accentColor
            ..style = PaintingStyle.stroke
            ..strokeWidth = 1.5,
        );
      }
      labelCandidates.add(
        _LabelCandidate(index: i, center: center, radius: radius),
      );
    }

    labelCandidates.sort((a, b) {
      final aNode = layout.nodes[a.index];
      final bNode = layout.nodes[b.index];
      if (aNode.isCenter != bNode.isCenter) return aNode.isCenter ? -1 : 1;
      return bNode.degree.compareTo(aNode.degree);
    });
    final maxLabels = viewport.zoom >= 0.9
        ? math.min(36, labelCandidates.length)
        : viewport.zoom >= 0.55
        ? math.min(12, labelCandidates.length)
        : 0;
    final labelRects = <Rect>[];
    var labelsDrawn = 0;
    for (final candidate in labelCandidates) {
      final node = layout.nodes[candidate.index];
      final isPriority = node.isCenter;
      if (labelsDrawn >= maxLabels && !isPriority) continue;
      final title = node.title.trim().isEmpty ? untitled : node.title.trim();
      final textPainter = TextPainter(
        text: TextSpan(
          text: title,
          style: textStyle.copyWith(
            color: isPriority ? textColor : mutedColor,
            fontSize: 11,
            fontWeight: isPriority ? FontWeight.w600 : FontWeight.w400,
          ),
        ),
        maxLines: 1,
        ellipsis: '…',
        textDirection: textDirection,
      )..layout(maxWidth: 150);
      final center = candidate.center;
      final gap = candidate.radius + 5;
      final options = <Rect>[
        Rect.fromCenter(
          center: Offset(center.dx, center.dy + gap + textPainter.height / 2),
          width: textPainter.width + 8,
          height: textPainter.height + 4,
        ),
        Rect.fromCenter(
          center: Offset(center.dx, center.dy - gap - textPainter.height / 2),
          width: textPainter.width + 8,
          height: textPainter.height + 4,
        ),
        Rect.fromCenter(
          center: Offset(center.dx + gap + textPainter.width / 2, center.dy),
          width: textPainter.width + 8,
          height: textPainter.height + 4,
        ),
        Rect.fromCenter(
          center: Offset(center.dx - gap - textPainter.width / 2, center.dy),
          width: textPainter.width + 8,
          height: textPainter.height + 4,
        ),
      ];
      Rect? placement;
      for (final option in options) {
        final clipped = option.intersect(Offset.zero & size);
        if (clipped == option &&
            labelRects.every((placed) => !placed.overlaps(option))) {
          placement = option;
          break;
        }
      }
      if (placement == null) continue;
      labelRects.add(placement);
      labelsDrawn++;
      textPainter.paint(
        canvas,
        Offset(
          placement.center.dx - textPainter.width / 2,
          placement.center.dy - textPainter.height / 2,
        ),
      );
    }
  }

  @override
  bool shouldRepaint(_GraphPainter oldDelegate) =>
      oldDelegate.layout != layout ||
      oldDelegate.viewport != viewport ||
      oldDelegate.textColor != textColor ||
      oldDelegate.mutedColor != mutedColor ||
      oldDelegate.edgeColor != edgeColor ||
      oldDelegate.surfaceColor != surfaceColor ||
      oldDelegate.accentColor != accentColor ||
      oldDelegate.textStyle != textStyle ||
      oldDelegate.textDirection != textDirection ||
      oldDelegate.untitled != untitled ||
      oldDelegate.dark != dark ||
      oldDelegate.onNodeSelected != onNodeSelected;
}

class _LabelCandidate {
  const _LabelCandidate({
    required this.index,
    required this.center,
    required this.radius,
  });

  final int index;
  final Offset center;
  final double radius;
}
