// Kanso — preview_figure.dart
// Created 2026-10-30.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/widgets.dart';
import 'package:flutter_svg/flutter_svg.dart';

import '../../../../core/bridge/api/types.dart';
import '../../../../core/utils/context_extensions.dart';
import 'preview_style.dart';

/// Math, a diagram or a drawing set on its own: its SVG at its size,
/// scaled with the system's text, made smaller to fit the note's width.
/// Display math is centred.
class PreviewFigure extends StatelessWidget {
  const PreviewFigure({required this.kind, required this.svg, super.key});

  final PreviewFigureKind kind;
  final PreviewSvg svg;

  @override
  Widget build(BuildContext context) {
    // Drawn at the note's text size, so scaled with the system's text as the
    // text around it is.
    final size = PreviewScope.of(context).style.size;
    final scale = size > 0
        ? MediaQuery.textScalerOf(context).scale(size) / size
        : 1.0;
    return Align(
      alignment: kind == PreviewFigureKind.math
          ? Alignment.center
          : Alignment.centerLeft,
      child: FittedBox(
        fit: BoxFit.scaleDown,
        child: SizedBox(
          width: svg.width * scale,
          height: svg.height * scale,
          child: SvgPicture.string(
            svg.markup,
            width: svg.width * scale,
            height: svg.height * scale,
          ),
        ),
      ),
    );
  }
}

/// A drawing with nothing in it yet: a quiet placeholder that says so.
class PreviewEmptyDrawing extends StatelessWidget {
  const PreviewEmptyDrawing({super.key});

  @override
  Widget build(BuildContext context) {
    final style = PreviewScope.of(context).style;
    return Align(
      alignment: Alignment.centerLeft,
      child: Container(
        constraints: const BoxConstraints(maxWidth: 380),
        height: 120,
        alignment: Alignment.center,
        decoration: BoxDecoration(
          color: style.surface,
          border: Border.all(color: style.border),
          borderRadius: BorderRadius.circular(8),
        ),
        child: Text(
          context.l10n.previewEmptyDrawing,
          style: style.sized(0.8667).copyWith(color: style.muted),
        ),
      ),
    );
  }
}
