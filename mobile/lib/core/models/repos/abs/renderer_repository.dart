// Kanso — renderer_repository.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:typed_data';

import 'package:dartz/dartz.dart';

import '../../../bridge/api/types.dart';
import '../../data/application_error.dart';

/// Drawing done by the Rust core: previews, source highlighting, themes,
/// math and diagrams.
abstract interface class RendererRepository {
  /// The note [id] as a standalone HTML page in [palette]'s colours; locked
  /// while the note is locked and closed.
  Future<Either<ApplicationError, String>> previewHtml(
    String id,
    PaletteDto palette,
  );

  /// The runs of [text] between [start] and [end] (UTF-16 units) that the
  /// Markdown highlighter styles.
  Future<Either<ApplicationError, List<SpanRow>>> highlight(
    String text, {
    required int start,
    required int end,
  });

  /// A theme file's JSON, fitted for drawing.
  Future<Either<ApplicationError, ThemeTokens>> themeTokens(String themeJson);

  Future<Either<ApplicationError, SvgRow>> mathSvg(
    String source, {
    required bool display,
    required PaletteDto palette,
  });

  Future<Either<ApplicationError, SvgRow>> diagramSvg(
    DiagramKind kind,
    String source,
    PaletteDto palette,
  );

  /// Hands over the app's fonts so diagrams are set in them; whether this
  /// call took effect (only the first before a drawing does).
  Future<Either<ApplicationError, bool>> setFonts(List<Uint8List> fonts);
}
