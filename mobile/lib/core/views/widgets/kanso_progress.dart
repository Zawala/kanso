// Kanso — kanso_progress.dart
// Created 2026-10-29.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:math' as math;

import 'package:flutter/material.dart';

import '../../configs/kanso_motion.dart';
import '../../configs/kanso_radii.dart';
import '../../configs/kanso_sizes.dart';
import '../../utils/context_extensions.dart';

/// A determinate line of progress, drawn by Kanso.
///
/// [value] runs from 0 to 1. Without one the bar is indeterminate: a short
/// accent segment travels along the track instead of filling it, which
/// reads as "still working" without implying how much is left.
///
/// It is 3 points tall by default so it never pushes a layout around, and
/// it is excluded from semantics because the row it sits in already says
/// what is happening; pass [semanticLabel] when it needs to speak.
class KansoProgressBar extends StatelessWidget {
  const KansoProgressBar({
    this.value,
    this.height = 3,
    this.semanticLabel,
    super.key,
  });

  /// How much is done, 0 to 1; null for a bar that does not know.
  final double? value;
  final double height;

  /// What a screen reader says instead of the raw fraction.
  final String? semanticLabel;

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
    final label = semanticLabel;
    final fraction = value;
    final bar = ClipRRect(
      borderRadius: KansoRadii.pillBorder,
      child: SizedBox(
        height: height,
        width: double.infinity,
        child: ColoredBox(
          color: colors.fill,
          child: fraction == null
              ? _KansoIndeterminateBar(color: colors.accent)
              : FractionallySizedBox(
                  alignment: AlignmentDirectional.centerStart,
                  widthFactor: fraction.clamp(0.0, 1.0),
                  child: ColoredBox(color: colors.accent),
                ),
        ),
      ),
    );
    if (label == null) {
      return ExcludeSemantics(child: bar);
    }
    return Semantics(
      label: label,
      value: fraction == null ? null : '${(fraction * 100).round()}%',
      child: bar,
    );
  }
}

/// The travelling segment of an indeterminate [KansoProgressBar].
///
/// It draws nothing at all when animations are turned off, rather than
/// freezing a segment in the middle of the track where it would look like
/// real, stuck progress.
class _KansoIndeterminateBar extends StatefulWidget {
  const _KansoIndeterminateBar({required this.color});

  final Color color;

  @override
  State<_KansoIndeterminateBar> createState() => _KansoIndeterminateBarState();
}

class _KansoIndeterminateBarState extends State<_KansoIndeterminateBar>
    with SingleTickerProviderStateMixin {
  late final AnimationController controller;

  @override
  void initState() {
    super.initState();
    controller = AnimationController(vsync: this);
  }

  @override
  void didChangeDependencies() {
    super.didChangeDependencies();
    final duration = KansoMotion.resolve(context, _travel);
    if (duration == Duration.zero) {
      controller
        ..stop()
        ..value = 0;
      return;
    }
    if (controller.duration != duration) {
      controller.duration = duration;
    }
    if (!controller.isAnimating) {
      controller.repeat();
    }
  }

  @override
  void dispose() {
    controller.dispose();
    super.dispose();
  }

  static const Duration _travel = Duration(milliseconds: 1100);

  /// How much of the track the segment covers at its longest.
  static const double _segment = 0.38;

  @override
  Widget build(BuildContext context) {
    if (controller.duration == Duration.zero) {
      return const SizedBox.shrink();
    }
    return AnimatedBuilder(
      animation: controller,
      builder: (context, _) {
        // The segment slides in from before the start and out past the end,
        // clipped by the track's rounded box, so it never jumps at an edge.
        final start = controller.value * (1 + _segment) - _segment;
        return LayoutBuilder(
          builder: (context, constraints) {
            final width = constraints.maxWidth.isFinite
                ? constraints.maxWidth
                : 0.0;
            final left = start * width;
            final segment = _segment * width;
            return Stack(
              children: <Widget>[
                Positioned(
                  left: left,
                  width: segment,
                  top: 0,
                  bottom: 0,
                  child: ColoredBox(color: widget.color),
                ),
              ],
            );
          },
        );
      },
    );
  }
}

/// A small turning mark that says work is happening, drawn by Kanso.
///
/// It is a hairline ring with one accent quarter, turning at a steady
/// pace. When animations are off it shows a still ring so the space does
/// not collapse. [size] defaults to [KansoSizes.icon].
class KansoSpinner extends StatefulWidget {
  const KansoSpinner({
    this.size = KansoSizes.icon,
    this.semanticLabel,
    super.key,
  });

  final double size;
  final String? semanticLabel;

  @override
  State<KansoSpinner> createState() => _KansoSpinnerState();
}

class _KansoSpinnerState extends State<KansoSpinner>
    with SingleTickerProviderStateMixin {
  late final AnimationController controller;

  @override
  void initState() {
    super.initState();
    controller = AnimationController(vsync: this);
  }

  @override
  void didChangeDependencies() {
    super.didChangeDependencies();
    final duration = KansoMotion.resolve(context, _turn);
    if (duration == Duration.zero) {
      controller
        ..stop()
        ..value = 0;
      return;
    }
    if (controller.duration != duration) {
      controller.duration = duration;
    }
    if (!controller.isAnimating) {
      controller.repeat();
    }
  }

  @override
  void dispose() {
    controller.dispose();
    super.dispose();
  }

  static const Duration _turn = Duration(milliseconds: 900);

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
    final mark = SizedBox.square(
      dimension: widget.size,
      child: AnimatedBuilder(
        animation: controller,
        builder: (context, child) => Transform.rotate(
          angle: controller.value * 2 * math.pi,
          child: child,
        ),
        child: CustomPaint(
          painter: _KansoRingPainter(
            track: colors.divider,
            head: colors.accent,
            stroke: widget.size >= 16 ? 1.6 : 1.2,
          ),
        ),
      ),
    );
    final label = widget.semanticLabel;
    if (label == null) {
      return ExcludeSemantics(child: mark);
    }
    return Semantics(label: label, child: mark);
  }
}

/// A ring with a quiet track and an accent quarter.
class _KansoRingPainter extends CustomPainter {
  const _KansoRingPainter({
    required this.track,
    required this.head,
    required this.stroke,
    this.fraction = 0.25,
  });

  final Color track;
  final Color head;
  final double stroke;

  /// How much of the ring the head covers; a spinner draws a quarter.
  final double fraction;

  @override
  void paint(Canvas canvas, Size size) {
    final rect = Offset.zero & size;
    final center = rect.center;
    final radius = math.min(size.width, size.height) / 2 - stroke / 2;
    if (radius <= 0) {
      return;
    }
    final arc = Rect.fromCircle(center: center, radius: radius);
    final ring = Paint()
      ..style = PaintingStyle.stroke
      ..strokeWidth = stroke
      ..strokeCap = StrokeCap.round;
    canvas.drawCircle(center, radius, ring..color = track);
    if (fraction <= 0) {
      return;
    }
    ring.color = head;
    canvas.drawArc(
      arc,
      -math.pi / 2,
      2 * math.pi * fraction.clamp(0.0, 1.0),
      false,
      ring,
    );
  }

  @override
  bool shouldRepaint(_KansoRingPainter old) =>
      old.track != track ||
      old.head != head ||
      old.stroke != stroke ||
      old.fraction != fraction;
}

/// A small determinate ring: [value] of the way round, in [color].
///
/// It stands in for a spinner where how much is left is known, such as a
/// word goal in a note's footer. The ring is [size] across and drawn with a
/// [stroke] of the same colour against the divider colour.
class KansoProgressRing extends StatelessWidget {
  const KansoProgressRing({
    required this.value,
    this.size = 14,
    this.stroke = 1.75,
    this.color,
    this.semanticLabel,
    super.key,
  });

  /// How much is done, 0 to 1.
  final double value;
  final double size;
  final double stroke;

  /// The arc's colour; the accent colour when null.
  final Color? color;
  final String? semanticLabel;

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
    final ring = SizedBox.square(
      dimension: size,
      child: CustomPaint(
        painter: _KansoRingPainter(
          track: colors.divider,
          head: color ?? colors.accent,
          stroke: stroke,
          fraction: value.clamp(0.0, 1.0),
        ),
      ),
    );
    final label = semanticLabel;
    if (label == null) {
      return ExcludeSemantics(child: ring);
    }
    return Semantics(
      label: label,
      value: '${(value * 100).round()}%',
      child: ring,
    );
  }
}
