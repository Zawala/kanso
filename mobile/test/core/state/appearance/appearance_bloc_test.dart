// Kanso — appearance_bloc_test.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:bloc_test/bloc_test.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/models/data/appearance_mode.dart';
import 'package:kanso/core/models/data/kanso_font_family.dart';
import 'package:kanso/core/models/data/kanso_fonts.dart';
import 'package:kanso/core/models/data/kanso_text_size.dart';
import 'package:kanso/core/state/appearance/appearance_bloc.dart';

void main() {
  group('AppearanceBloc', () {
    test('starts following the system', () {
      expect(AppearanceBloc().state.mode, AppearanceMode.system);
    });

    blocTest<AppearanceBloc, AppearanceState>(
      'started announces the default appearance',
      build: AppearanceBloc.new,
      act: (bloc) => bloc.add(const AppearanceEvent.started()),
      expect: () => <AppearanceState>[const AppearanceState()],
    );

    blocTest<AppearanceBloc, AppearanceState>(
      'changed emits the chosen mode',
      build: AppearanceBloc.new,
      act: (bloc) => bloc
        ..add(const AppearanceEvent.changed(AppearanceMode.dark))
        ..add(const AppearanceEvent.changed(AppearanceMode.light)),
      expect: () => <AppearanceState>[
        const AppearanceState(mode: AppearanceMode.dark),
        const AppearanceState(mode: AppearanceMode.light),
      ],
    );

    blocTest<AppearanceBloc, AppearanceState>(
      'choosing the same mode twice emits once',
      build: AppearanceBloc.new,
      act: (bloc) => bloc
        ..add(const AppearanceEvent.changed(AppearanceMode.dark))
        ..add(const AppearanceEvent.changed(AppearanceMode.dark)),
      expect: () => <AppearanceState>[
        const AppearanceState(mode: AppearanceMode.dark),
      ],
    );

    blocTest<AppearanceBloc, AppearanceState>(
      'started after a change goes back to the system',
      build: AppearanceBloc.new,
      seed: () => const AppearanceState(mode: AppearanceMode.dark),
      act: (bloc) => bloc.add(const AppearanceEvent.started()),
      expect: () => <AppearanceState>[const AppearanceState()],
    );

    test("starts with Kanso's fonts at the designed text size", () {
      final state = AppearanceBloc().state;
      expect(state.fonts, const KansoFonts());
      expect(state.textSize, KansoTextSize.standard);
    });

    blocTest<AppearanceBloc, AppearanceState>(
      'editorFontChanged changes only the editor font',
      build: AppearanceBloc.new,
      act: (bloc) => bloc.add(
        const AppearanceEvent.editorFontChanged(KansoFontFamily.literata),
      ),
      expect: () => <AppearanceState>[
        const AppearanceState(
          fonts: KansoFonts(editorFont: KansoFontFamily.literata),
        ),
      ],
    );

    blocTest<AppearanceBloc, AppearanceState>(
      "editorFontChanged accepts the device's font",
      build: AppearanceBloc.new,
      act: (bloc) => bloc.add(
        const AppearanceEvent.editorFontChanged(KansoFontFamily.system),
      ),
      expect: () => <AppearanceState>[
        const AppearanceState(
          fonts: KansoFonts(editorFont: KansoFontFamily.system),
        ),
      ],
    );

    blocTest<AppearanceBloc, AppearanceState>(
      'previewFontChanged changes only the preview font',
      build: AppearanceBloc.new,
      act: (bloc) => bloc.add(
        const AppearanceEvent.previewFontChanged(KansoFontFamily.sourceSerif4),
      ),
      expect: () => <AppearanceState>[
        const AppearanceState(
          fonts: KansoFonts(previewFont: KansoFontFamily.sourceSerif4),
        ),
      ],
    );

    blocTest<AppearanceBloc, AppearanceState>(
      'codeFontChanged changes only the code font',
      build: AppearanceBloc.new,
      act: (bloc) => bloc.add(
        const AppearanceEvent.codeFontChanged(KansoFontFamily.ibmPlexMono),
      ),
      expect: () => <AppearanceState>[
        const AppearanceState(
          fonts: KansoFonts(codeFont: KansoFontFamily.ibmPlexMono),
        ),
      ],
    );

    blocTest<AppearanceBloc, AppearanceState>(
      'codeFontChanged ignores a family that is not monospaced',
      build: AppearanceBloc.new,
      act: (bloc) => bloc.add(
        const AppearanceEvent.codeFontChanged(KansoFontFamily.inter),
      ),
      expect: () => <AppearanceState>[],
    );

    blocTest<AppearanceBloc, AppearanceState>(
      'textSizeChanged emits each size chosen',
      build: AppearanceBloc.new,
      act: (bloc) => bloc
        ..add(const AppearanceEvent.textSizeChanged(KansoTextSize.large))
        ..add(const AppearanceEvent.textSizeChanged(KansoTextSize.small)),
      expect: () => <AppearanceState>[
        const AppearanceState(textSize: KansoTextSize.large),
        const AppearanceState(textSize: KansoTextSize.small),
      ],
    );

    blocTest<AppearanceBloc, AppearanceState>(
      'font and size choices keep each other and the mode',
      build: AppearanceBloc.new,
      act: (bloc) => bloc
        ..add(const AppearanceEvent.changed(AppearanceMode.dark))
        ..add(const AppearanceEvent.textSizeChanged(KansoTextSize.larger))
        ..add(const AppearanceEvent.editorFontChanged(KansoFontFamily.inter))
        ..add(const AppearanceEvent.previewFontChanged(KansoFontFamily.inter)),
      verify: (bloc) {
        expect(
          bloc.state,
          const AppearanceState(
            mode: AppearanceMode.dark,
            textSize: KansoTextSize.larger,
            fonts: KansoFonts(
              editorFont: KansoFontFamily.inter,
              previewFont: KansoFontFamily.inter,
            ),
          ),
        );
      },
    );

    blocTest<AppearanceBloc, AppearanceState>(
      'choosing the same font twice emits once',
      build: AppearanceBloc.new,
      act: (bloc) => bloc
        ..add(const AppearanceEvent.previewFontChanged(KansoFontFamily.inter))
        ..add(const AppearanceEvent.previewFontChanged(KansoFontFamily.inter)),
      expect: () => <AppearanceState>[
        const AppearanceState(
          fonts: KansoFonts(previewFont: KansoFontFamily.inter),
        ),
      ],
    );

    blocTest<AppearanceBloc, AppearanceState>(
      'started goes back to the default fonts and size',
      build: AppearanceBloc.new,
      seed: () => const AppearanceState(
        fonts: KansoFonts(editorFont: KansoFontFamily.literata),
        textSize: KansoTextSize.large,
      ),
      act: (bloc) => bloc.add(const AppearanceEvent.started()),
      expect: () => <AppearanceState>[const AppearanceState()],
    );
  });
}
