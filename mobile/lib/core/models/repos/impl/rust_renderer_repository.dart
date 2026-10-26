// Kanso — rust_renderer_repository.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:typed_data';

import 'package:dartz/dartz.dart';
import 'package:injectable/injectable.dart';

import '../../../bridge/api/renderer.dart';
import '../../../bridge/api/types.dart';
import '../../../utils/kanso_error_mapping.dart';
import '../../data/application_error.dart';
import '../abs/renderer_repository.dart';

/// [RendererRepository] over the Rust core's renderer.
@LazySingleton(as: RendererRepository)
class RustRendererRepository implements RendererRepository {
  RustRendererRepository(this._renderer);

  final KansoRenderer _renderer;

  @override
  Future<Either<ApplicationError, String>> previewHtml(
    String id,
    PaletteDto palette,
  ) => attempt(() => _renderer.previewHtml(id: id, palette: palette));

  @override
  Future<Either<ApplicationError, List<SpanRow>>> highlight(
    String text, {
    required int start,
    required int end,
  }) => attempt(
    () => KansoRenderer.highlight(text: text, start: start, end: end),
  );

  @override
  Future<Either<ApplicationError, ThemeTokens>> themeTokens(
    String themeJson,
  ) => attempt(() => KansoRenderer.themeTokens(themeJson: themeJson));

  @override
  Future<Either<ApplicationError, SvgRow>> mathSvg(
    String source, {
    required bool display,
    required PaletteDto palette,
  }) => attempt(
    () => KansoRenderer.mathSvg(
      source: source,
      display: display,
      palette: palette,
    ),
  );

  @override
  Future<Either<ApplicationError, SvgRow>> diagramSvg(
    DiagramKind kind,
    String source,
    PaletteDto palette,
  ) => attempt(
    () => KansoRenderer.diagramSvg(
      kind: kind,
      source: source,
      palette: palette,
    ),
  );

  @override
  Future<Either<ApplicationError, bool>> setFonts(List<Uint8List> fonts) =>
      attempt(() => KansoRenderer.setFonts(fonts: fonts));
}
