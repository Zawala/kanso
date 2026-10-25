// Kanso — kanso_mark.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:math' as math;

import 'package:flutter/material.dart';

import '../../configs/kanso_brand.dart';
import '../../configs/kanso_sizes.dart';
import '../../utils/context_extensions.dart';

/// Kanso's mark: an ink ensō, thick where the brush lands and thinning
/// toward the gap, with a vermilion seal in the gap. The ink follows the
/// theme's text colour.
class KansoMark extends StatelessWidget {
  const KansoMark({this.size = KansoSizes.mark, super.key});

  final double size;

  @override
  Widget build(BuildContext context) {
    return CustomPaint(
      size: Size.square(size),
      painter: KansoMarkPainter(ink: context.colors.text),
    );
  }
}

/// Draws the [KansoMark] on a square canvas.
class KansoMarkPainter extends CustomPainter {
  const KansoMarkPainter({required this.ink});

  final Color ink;

  // The geometry of the app icon, on its 1024 grid, around the ring's
  // centre: a radius of 228, a start of 58 degrees, a sweep of 318.
  static const double ringRadius = 228;
  static const double ringStartDegrees = 58;
  static const double ringSweepDegrees = 318;
  static const int ringSteps = 240;
  static const double extent = 520;

  @override
  void paint(Canvas canvas, Size size) {
    final scale = size.width / extent;
    final centre = Offset(size.width / 2, size.height / 2);
    final paint = Paint()
      ..color = ink
      ..strokeCap = StrokeCap.round
      ..style = PaintingStyle.stroke;
    Offset at(double degrees) {
      final angle = degrees * math.pi / 180;
      return centre +
          Offset(math.cos(angle), -math.sin(angle)) * ringRadius * scale;
    }

    for (var i = 0; i < ringSteps; i++) {
      final t0 = i / ringSteps;
      final t1 = (i + 1) / ringSteps;
      paint.strokeWidth =
          (30 + 34 * math.sin(math.pi * math.pow(t0, 0.8))) * scale;
      canvas.drawLine(
        at(ringStartDegrees + ringSweepDegrees * t0),
        at(ringStartDegrees + ringSweepDegrees * t1),
        paint,
      );
    }
    canvas.drawCircle(
      centre + const Offset(195, -185) * scale,
      24 * scale,
      Paint()..color = KansoBrand.seal,
    );
  }

  @override
  bool shouldRepaint(KansoMarkPainter oldDelegate) => oldDelegate.ink != ink;
}
