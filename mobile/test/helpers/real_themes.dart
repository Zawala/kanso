// Kanso — real_themes.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';
import 'package:flutter_test/flutter_test.dart' show Fake, setUpAll;
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/configs/kanso_palette.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/core/models/repos/abs/renderer_repository.dart';
import 'package:kanso/core/models/repos/impl/rust_renderer_repository.dart';
import 'package:kanso/core/utils/palette_from_tokens.dart';

import 'core_library.dart';

/// The built-in themes as the Rust core gives them, in the desktop's order.
/// Valid after [loadRealThemes] has run in a `setUpAll`.
late List<BuiltinTheme> realThemes;

/// The built-in themes by their id.
Map<String, BuiltinTheme> get realThemesById => <String, BuiltinTheme>{
  for (final theme in realThemes) theme.id: theme,
};

/// The palette of the real theme [id], from its tokens.
KansoPalette realPalette(String id) {
  final theme = realThemesById[id]!;
  return paletteFromTokens(theme.id, theme.tokens);
}

/// Opens the Rust core and reads the built-in themes, once for the whole
/// test file. Call it in `main`, not inside a test: the core's calls need
/// the real clock a `testWidgets` body does not have.
void loadRealThemes() {
  setUpAll(() async {
    await initCoreForTests();
    final library = await openTestLibrary();
    final themes = await RustRendererRepository(
      library.renderer(),
    ).builtinThemes();
    realThemes = themes.getOrElse(
      () => throw StateError('The built-in themes did not load.'),
    );
  });
}

/// A renderer that answers the theme calls from [realThemes], so a bloc
/// built on it picks themes without a library open in the test.
class RealThemesRenderer extends Fake implements RendererRepository {
  @override
  Future<Either<ApplicationError, List<BuiltinTheme>>> builtinThemes() async =>
      Right<ApplicationError, List<BuiltinTheme>>(realThemes);

  @override
  Future<Either<ApplicationError, ThemeTokens>> builtinThemeTokens(
    String id,
  ) async {
    final theme = realThemesById[id];
    return theme == null
        ? const Left<ApplicationError, ThemeTokens>(
            ApplicationError.notFound(message: 'No such theme.'),
          )
        : Right<ApplicationError, ThemeTokens>(theme.tokens);
  }
}
