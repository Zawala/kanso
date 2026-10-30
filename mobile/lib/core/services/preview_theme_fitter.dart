// Kanso — preview_theme_fitter.dart
// Created 2026-10-30.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';
import 'package:injectable/injectable.dart';

import '../bridge/api/types.dart' show ThemeTokens;
import '../models/data/application_error.dart';
import '../models/data/preview_themes.dart';
import '../models/repos/abs/themes_repository.dart';

export '../models/data/preview_themes.dart';

/// The tokens a note is drawn in, in its preview and in its exports: the
/// interface, syntax and preview themes fitted together, as the app's
/// colours are, or Kanso's own light or dark theme when they can't be read,
/// as the app falls back.
///
/// It keeps the tokens last fitted, and the themes they were fitted for, so
/// that a note drawn again doesn't fit the same themes again. Kanso's own
/// tokens, used when the themes can't be read, are never kept, so the next
/// draw tries the themes again.
@injectable
class PreviewThemeFitter {
  PreviewThemeFitter(this._themes);

  final ThemesRepository _themes;

  (PreviewThemes, ThemeTokens)? _fitted;

  /// The tokens of [themes], fitted together; Kanso's own light or dark
  /// theme, as [themes] says, when they can't be read.
  Future<Either<ApplicationError, ThemeTokens>> tokens(
    PreviewThemes themes,
  ) async {
    if (_fitted case (final known, final tokens) when known == themes) {
      return Right<ApplicationError, ThemeTokens>(tokens);
    }
    final fitted = await _themes.fit(
      themes.ui,
      themes.syntax,
      themes.preview,
    );
    if (fitted case Right(value: final tokens)) {
      _fitted = (themes, tokens);
      return fitted;
    }
    final own = themes.dark ? 'kanso-dark' : 'kanso-light';
    return _themes.fit(own, own, own);
  }
}
