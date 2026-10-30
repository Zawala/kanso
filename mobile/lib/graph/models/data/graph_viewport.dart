// Kanso — graph_viewport.dart
// Created 2026-10-29.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:math' as math;
import 'dart:ui';

import 'package:freezed_annotation/freezed_annotation.dart';

part 'graph_viewport.freezed.dart';

/// How the graph's layout maps onto the screen: the layout point in the
/// middle of the view, and how many points one layout unit takes.
///
/// The same transform the desktop graph draws through
/// (`kanso_engine::discovery::graph::Viewport`), ported as it is so the two
/// apps zoom, pan and fit alike.
@freezed
abstract class GraphViewport with _$GraphViewport {
  const factory GraphViewport({
    @Default(Offset.zero) Offset center,
    @Default(1.0) double zoom,
  }) = _GraphViewport;

  const GraphViewport._();

  /// The view that shows all of [positions] in a view [size] big, with
  /// [margin] points around them, no closer in than [maxZoom].
  factory GraphViewport.fit(
    List<Offset> positions,
    Size size, {
    required double margin,
    required double maxZoom,
  }) {
    final bounds = graphExtent(positions);
    if (bounds == null) return const GraphViewport();
    final width = math.max(bounds.width, 1);
    final height = math.max(bounds.height, 1);
    final roomX = math.max(size.width - 2 * margin, 1);
    final roomY = math.max(size.height - 2 * margin, 1);
    final zoom = math
        .min(roomX / width, roomY / height)
        .clamp(minZoom, math.min(maxZoom, GraphViewport.maxZoom))
        .toDouble();
    return GraphViewport(center: bounds.center, zoom: zoom);
  }

  /// The closest the view ever zooms out.
  static const double minZoom = 0.05;

  /// The closest the view ever zooms in.
  static const double maxZoom = 6;

  /// Where layout point [p] is in a view [size] big, from its top left.
  Offset toScreen(Offset p, Size size) => Offset(
    (p.dx - center.dx) * zoom + size.width / 2,
    (p.dy - center.dy) * zoom + size.height / 2,
  );

  /// The layout point at [s] in a view [size] big.
  Offset toLayout(Offset s, Size size) => Offset(
    (s.dx - size.width / 2) / zoom + center.dx,
    (s.dy - size.height / 2) / zoom + center.dy,
  );

  /// Zooms by [factor] (kept within [min] and [max]), keeping the layout
  /// point under [at] where it is.
  GraphViewport zoomAt(
    double factor,
    Offset at,
    Size size, {
    double min = minZoom,
    double max = maxZoom,
  }) {
    final anchor = toLayout(at, size);
    final next = GraphViewport(
      center: center,
      zoom: (zoom * factor).clamp(min, max),
    );
    final moved = next.toLayout(at, size);
    return next.copyWith(center: center + (anchor - moved));
  }

  /// Moves the view by [delta] points, as dragging the graph does.
  GraphViewport panned(Offset delta) =>
      GraphViewport(center: center - delta / zoom, zoom: zoom);

  /// Applies either a one-pointer pan or a multi-pointer anchored zoom.
  /// Pinching deliberately ignores [focalDelta], so the gesture cannot pan
  /// and zoom the graph at the same time.
  GraphViewport updatedForGesture({
    required int pointerCount,
    required double scaleFactor,
    required Offset focalPoint,
    required Offset focalDelta,
    required Size size,
    required Rect bounds,
  }) {
    final updated = pointerCount > 1
        ? zoomAt(scaleFactor, focalPoint, size)
        : panned(focalDelta);
    return updated.keptOver(bounds);
  }

  /// The layout rectangle a view [size] big shows, grown by [margin]
  /// points on every side.
  Rect visible(Size size, double margin) => Rect.fromPoints(
    toLayout(Offset(-margin, -margin), size),
    toLayout(Offset(size.width + margin, size.height + margin), size),
  );

  /// This view with its middle kept inside [bounds], so the graph can never
  /// be dragged out of sight.
  GraphViewport keptOver(Rect bounds) => copyWith(
    center: Offset(
      center.dx.clamp(bounds.left, bounds.right),
      center.dy.clamp(bounds.top, bounds.bottom),
    ),
  );

  /// The view [t] of the way from this one to [to]: the middle moves in a
  /// line and the zoom evenly in scale, so a zoom feels steady.
  GraphViewport lerpTo(GraphViewport to, double t) => GraphViewport(
    center: Offset.lerp(center, to.center, t)!,
    zoom: math.exp(
      math.log(zoom) + (math.log(to.zoom) - math.log(zoom)) * t,
    ),
  );
}

/// The rectangle around [positions]; null for none.
Rect? graphExtent(List<Offset> positions) {
  if (positions.isEmpty) return null;
  var left = positions.first.dx;
  var right = left;
  var top = positions.first.dy;
  var bottom = top;
  for (final p in positions) {
    left = math.min(left, p.dx);
    right = math.max(right, p.dx);
    top = math.min(top, p.dy);
    bottom = math.max(bottom, p.dy);
  }
  return Rect.fromLTRB(left, top, right, bottom);
}

/// Whether a segment from [a] to [b] may cross [area]: its own bounding box
/// meets it.
bool segmentVisible(Offset a, Offset b, Rect area) =>
    math.max(a.dx, b.dx) >= area.left &&
    math.min(a.dx, b.dx) <= area.right &&
    math.max(a.dy, b.dy) >= area.top &&
    math.min(a.dy, b.dy) <= area.bottom;

/// A node's radius on screen at [zoom], from its [radius] in layout units:
/// it shrinks as the view zooms out, but never below a dot nor above a
/// button, so zooming in spreads the notes apart instead of swelling them.
double screenRadius(double radius, double zoom) =>
    (radius * math.sqrt(zoom)).clamp(1.6, 16).toDouble();
