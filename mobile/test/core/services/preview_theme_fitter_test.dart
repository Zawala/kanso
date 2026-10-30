// Kanso — preview_theme_fitter_test.dart
// Created 2026-10-30.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/configs/kanso_palette.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/core/services/preview_theme_fitter.dart';
import 'package:mocktail/mocktail.dart';

import '../../helpers/seeded_appearance_bloc.dart' show MockThemesRepository;
import '../../helpers/theme_tokens.dart';

const PreviewThemes _picked = (
  ui: 'nord',
  syntax: 'gallery-dracula',
  preview: 'gallery-paper',
  dark: true,
);

const _failure = ApplicationError.storage(message: 'Disk error.');

void main() {
  final fitted = tokensFor(fakePalette('gallery-paper'));
  final dark = tokensFor(KansoPalette.kansoDark);
  final light = tokensFor(KansoPalette.kansoLight);

  late MockThemesRepository themes;
  late PreviewThemeFitter fitter;

  setUp(() {
    themes = MockThemesRepository();
    fitter = PreviewThemeFitter(themes);
    when(
      () => themes.fit('nord', 'gallery-dracula', 'gallery-paper'),
    ).thenAnswer((_) async => Right<ApplicationError, ThemeTokens>(fitted));
    when(
      () => themes.fit('kanso-dark', 'kanso-dark', 'kanso-dark'),
    ).thenAnswer((_) async => Right<ApplicationError, ThemeTokens>(dark));
    when(
      () => themes.fit('kanso-light', 'kanso-light', 'kanso-light'),
    ).thenAnswer((_) async => Right<ApplicationError, ThemeTokens>(light));
  });

  void unreadable() =>
      when(
        () => themes.fit('nord', 'gallery-dracula', 'gallery-paper'),
      ).thenAnswer(
        (_) async => const Left<ApplicationError, ThemeTokens>(_failure),
      );

  test('the three themes are fitted together', () async {
    expect(
      await fitter.tokens(_picked),
      Right<ApplicationError, ThemeTokens>(fitted),
    );
    verify(
      () => themes.fit('nord', 'gallery-dracula', 'gallery-paper'),
    ).called(1);
  });

  test('the same themes again are not fitted again', () async {
    await fitter.tokens(_picked);
    expect(
      await fitter.tokens(_picked),
      Right<ApplicationError, ThemeTokens>(fitted),
    );
    verify(
      () => themes.fit('nord', 'gallery-dracula', 'gallery-paper'),
    ).called(1);
  });

  test('other themes are fitted, and only the last fit is kept', () async {
    const PreviewThemes other = (
      ui: 'kanso-light',
      syntax: 'kanso-light',
      preview: 'kanso-light',
      dark: false,
    );
    await fitter.tokens(_picked);
    expect(
      await fitter.tokens(other),
      Right<ApplicationError, ThemeTokens>(light),
    );
    await fitter.tokens(_picked);
    verify(
      () => themes.fit('nord', 'gallery-dracula', 'gallery-paper'),
    ).called(2);
  });

  test("themes that can't be read fall back to Kanso's own dark", () async {
    unreadable();
    expect(
      await fitter.tokens(_picked),
      Right<ApplicationError, ThemeTokens>(dark),
    );
  });

  test("the light slot falls back to Kanso's own light", () async {
    unreadable();
    expect(
      await fitter.tokens((
        ui: 'nord',
        syntax: 'gallery-dracula',
        preview: 'gallery-paper',
        dark: false,
      )),
      Right<ApplicationError, ThemeTokens>(light),
    );
  });

  test('the fallback is not kept: the themes are tried again', () async {
    unreadable();
    await fitter.tokens(_picked);
    when(
      () => themes.fit('nord', 'gallery-dracula', 'gallery-paper'),
    ).thenAnswer((_) async => Right<ApplicationError, ThemeTokens>(fitted));
    expect(
      await fitter.tokens(_picked),
      Right<ApplicationError, ThemeTokens>(fitted),
    );
    verify(
      () => themes.fit('nord', 'gallery-dracula', 'gallery-paper'),
    ).called(2);
  });

  test("Kanso's own that can't be read either is the error", () async {
    when(() => themes.fit(any(), any(), any())).thenAnswer(
      (_) async => const Left<ApplicationError, ThemeTokens>(_failure),
    );
    expect(
      await fitter.tokens(_picked),
      const Left<ApplicationError, ThemeTokens>(_failure),
    );
  });
}
