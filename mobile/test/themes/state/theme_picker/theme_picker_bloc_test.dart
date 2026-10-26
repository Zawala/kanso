// Kanso — theme_picker_bloc_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:bloc_test/bloc_test.dart';
import 'package:dartz/dartz.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/core/models/data/load_status.dart';
import 'package:kanso/themes/state/theme_picker/theme_picker_bloc.dart';
import 'package:kanso/themes/utils/theme_preview_from_builtin.dart';
import 'package:mocktail/mocktail.dart';

import '../../../helpers/seeded_appearance_bloc.dart';
import '../../../helpers/theme_tokens.dart';

BuiltinTheme _theme(String id, {bool dark = false}) {
  return BuiltinTheme(
    id: id,
    name: id.toUpperCase(),
    author: 'Someone',
    license: 'MIT',
    dark: dark,
    tokens: tokensFor(fakePalette(id, dark: dark)),
  );
}

void main() {
  late MockRendererRepository renderer;
  final themes = <BuiltinTheme>[
    _theme('kanso-light'),
    _theme('nord', dark: true),
  ];

  setUp(() {
    renderer = MockRendererRepository();
    when(renderer.builtinThemes).thenAnswer(
      (_) async => Right<ApplicationError, List<BuiltinTheme>>(themes),
    );
  });

  ThemePickerBloc build() => ThemePickerBloc(renderer);

  group('ThemePickerBloc', () {
    test('starts loading, with no themes', () {
      final state = build().state;
      expect(state.status, LoadStatus.loading);
      expect(state.themes, isEmpty);
    });

    blocTest<ThemePickerBloc, ThemePickerState>(
      'started offers the built-in themes in order, previewed from their '
      'tokens',
      build: build,
      act: (bloc) => bloc.add(const ThemePickerEvent.started()),
      expect: () => <ThemePickerState>[
        const ThemePickerState(),
        ThemePickerState(
          status: LoadStatus.ready,
          themes: themes.map(themePreviewFromBuiltin).toList(),
        ),
      ],
      verify: (bloc) {
        expect(bloc.state.themes.map((theme) => theme.id), <String>[
          'kanso-light',
          'nord',
        ]);
        expect(bloc.state.themes.last.dark, isTrue);
      },
    );

    blocTest<ThemePickerBloc, ThemePickerState>(
      'started fails when the themes cannot be read, and reads them again '
      'when asked to try again',
      setUp: () {
        var calls = 0;
        when(renderer.builtinThemes).thenAnswer((_) async {
          calls++;
          return calls == 1
              ? const Left<ApplicationError, List<BuiltinTheme>>(
                  ApplicationError.internal(message: 'No themes.'),
                )
              : Right<ApplicationError, List<BuiltinTheme>>(themes);
        });
      },
      build: build,
      act: (bloc) async {
        bloc.add(const ThemePickerEvent.started());
        await Future<void>.delayed(Duration.zero);
        bloc.add(const ThemePickerEvent.started());
      },
      expect: () => <ThemePickerState>[
        const ThemePickerState(),
        const ThemePickerState(status: LoadStatus.failed),
        const ThemePickerState(),
        ThemePickerState(
          status: LoadStatus.ready,
          themes: themes.map(themePreviewFromBuiltin).toList(),
        ),
      ],
    );
  });
}
