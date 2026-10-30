// Kanso — live_preview_drawer.dart
// Created 2026-10-30.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:clock/clock.dart';
import 'package:dartz/dartz.dart';

import '../../../core/bridge/api/types.dart';
import '../../../core/models/data/application_error.dart';
import '../../../core/models/repos/abs/renderer_repository.dart';
import '../../../core/models/repos/abs/themes_repository.dart';
import '../../../note/state/preview/preview_bloc.dart' show PreviewThemes;
import 'live_layout_bridge.dart';
import 'live_rows.dart';

/// Draws the preview's blocks of the editor's text of a note, in the themes
/// and the text size in force, as `PreviewBloc` draws a saved note's: what
/// the Live editor draws its widgets from.
class LivePreviewDrawer {
  LivePreviewDrawer({
    required this.renderer,
    required this.themes,
    required this.noteId,
    required this.inForce,
    required this.size,
  });

  final RendererRepository renderer;
  final ThemesRepository themes;
  final String noteId;

  /// The themes in force, and the preview's text size in points.
  PreviewThemes inForce;
  double size;

  (PreviewThemes, ThemeTokens)? _fitted;

  Future<Either<ApplicationError, ThemeTokens>> _tokens() async {
    final wanted = inForce;
    if (_fitted case (final known, final tokens) when known == wanted) {
      return Right<ApplicationError, ThemeTokens>(tokens);
    }
    final fitted = await themes.fit(wanted.ui, wanted.syntax, wanted.preview);
    if (fitted case Right(value: final tokens)) {
      _fitted = (wanted, tokens);
      return fitted;
    }
    final own = wanted.dark ? 'kanso-dark' : 'kanso-light';
    return themes.fit(own, own, own);
  }

  /// The preview's top-level blocks of [text].
  Future<Either<ApplicationError, List<LivePreviewBlock>>> draw(
    String text,
  ) async {
    final tokens = await _tokens();
    return tokens.fold(
      (error) async => Left<ApplicationError, List<LivePreviewBlock>>(error),
      (tokens) async {
        final blocks = await renderer.previewBlocksForText(
          noteId,
          text,
          tokens.palette,
          PreviewOptions(
            size: size,
            utcOffsetSecs: clock.now().timeZoneOffset.inSeconds,
            code: tokens.syntax,
          ),
        );
        return blocks.map((blocks) => livePreviewBlocksOf(blocks, text));
      },
    );
  }
}
