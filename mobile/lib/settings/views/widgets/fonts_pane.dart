// Kanso — fonts_pane.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../core/configs/kanso_spacing.dart';
import '../../../core/configs/kanso_typography.dart';
import '../../../core/models/data/kanso_font_family.dart';
import '../../../core/models/data/kanso_font_slot.dart';
import '../../../core/models/data/kanso_fonts.dart';
import '../../../core/models/data/kanso_text_size.dart';
import '../../../core/state/appearance/appearance_bloc.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_grouped_section.dart';
import '../../../core/views/widgets/kanso_scroll_host.dart';
import '../../../core/views/widgets/kanso_top_bar.dart';
import '../../../shell/views/widgets/shell_back_button.dart';
import 'font_choice_row.dart';
import 'google_font_section.dart';
import 'text_size_section.dart';

/// Settings › Fonts: the interface font, the editor font, the preview font
/// and the code font, each a list in which every family, bundled or
/// downloaded from Google Fonts, is shown in itself with the one in use
/// checked; then the field that downloads a Google Font, and the text
/// sizes.
///
/// The choices are the [AppearanceBloc]'s, so choosing here restyles every
/// note in the app at once. [startAtEnd] shows the page scrolled to its
/// foot.
class FontsPane extends StatelessWidget {
  const FontsPane({this.startAtEnd = false, super.key});

  final bool startAtEnd;

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
    final l10n = context.l10n;
    final bottom = MediaQuery.paddingOf(context).bottom;
    final bloc = context.read<AppearanceBloc>();
    return ColoredBox(
      color: colors.app,
      child: Column(
        children: <Widget>[
          KansoTopBar(
            backgroundColor: colors.app,
            bordered: false,
            leading: const ShellBackButton(),
            title: l10n.fontsTitle,
          ),
          Expanded(
            child: KansoScrollHost(
              startAtEnd: startAtEnd,
              builder: (context, controller) {
                return BlocBuilder<AppearanceBloc, AppearanceState>(
                  builder: (context, state) {
                    return ListView(
                      controller: controller,
                      padding: EdgeInsets.fromLTRB(
                        KansoSpacing.md,
                        KansoSpacing.xs,
                        KansoSpacing.md,
                        bottom + KansoSpacing.lg,
                      ),
                      children: <Widget>[
                        KansoGroupedSection(
                          title: l10n.fontsInterfaceSection,
                          footer: l10n.fontsInterfaceFooter,
                          children: fontRows(
                            context,
                            state.fonts,
                            KansoFontSlot.interface,
                          ),
                        ),
                        const SizedBox(height: KansoSpacing.lg),
                        KansoGroupedSection(
                          title: l10n.fontsEditorSection,
                          footer: l10n.fontsEditorFooter,
                          children: fontRows(
                            context,
                            state.fonts,
                            KansoFontSlot.editor,
                          ),
                        ),
                        const SizedBox(height: KansoSpacing.lg),
                        KansoGroupedSection(
                          title: l10n.fontsPreviewSection,
                          footer: l10n.fontsPreviewFooter,
                          children: fontRows(
                            context,
                            state.fonts,
                            KansoFontSlot.preview,
                          ),
                        ),
                        const SizedBox(height: KansoSpacing.lg),
                        KansoGroupedSection(
                          title: l10n.fontsCodeSection,
                          footer: l10n.fontsCodeFooter,
                          children: fontRows(
                            context,
                            state.fonts,
                            KansoFontSlot.code,
                          ),
                        ),
                        const SizedBox(height: KansoSpacing.lg),
                        const GoogleFontSection(),
                        const SizedBox(height: KansoSpacing.lg),
                        TextSizeSection(
                          title: l10n.fontsInterfaceTextSizeSection,
                          footer: l10n.fontsInterfaceTextSizeFooter,
                          sample: l10n.fontsInterfaceTextSizeSample,
                          selected: state.interfaceTextSize,
                          minimum: minimumInterfaceTextScale,
                          maximum: maximumInterfaceTextScale,
                          step: interfaceTextScaleStep,
                          semanticLabel: l10n.interfaceTextSizeControlLabel,
                          decreaseLabel: l10n.interfaceTextSizeDecrease,
                          increaseLabel: l10n.interfaceTextSizeIncrease,
                          sampleStyle: context.typography.body.copyWith(
                            fontSize: 16 * state.interfaceTextSize,
                          ),
                          onChanged: (size) => bloc.add(
                            AppearanceEvent.interfaceTextSizeChanged(size),
                          ),
                        ),
                        const SizedBox(height: KansoSpacing.lg),
                        TextSizeSection(
                          selected: state.textSize,
                          sampleStyle: context.typography.editorBody.copyWith(
                            fontSize: 18 * state.textSize,
                            height: KansoTypography.leading(
                              state.fonts.editorFont,
                            ),
                            fontFamily:
                                state.fonts.editorGoogleFont ??
                                state.fonts.editorFont.family ??
                                KansoTypography.systemFamily,
                          ),
                          onChanged: (size) =>
                              bloc.add(AppearanceEvent.textSizeChanged(size)),
                        ),
                        const SizedBox(height: KansoSpacing.lg),
                        TextSizeSection(
                          title: l10n.fontsPreviewTextSizeSection,
                          footer: l10n.fontsPreviewTextSizeFooter,
                          selected: state.previewTextSize,
                          sampleStyle: context.typography.previewBody.copyWith(
                            fontSize: 18 * state.previewTextSize,
                            height: KansoTypography.leading(
                              state.fonts.previewFont,
                            ),
                            fontFamily:
                                state.fonts.previewGoogleFont ??
                                state.fonts.previewFont.family ??
                                KansoTypography.systemFamily,
                          ),
                          onChanged: (size) => bloc.add(
                            AppearanceEvent.previewTextSizeChanged(size),
                          ),
                        ),
                      ],
                    );
                  },
                );
              },
            ),
          ),
        ],
      ),
    );
  }

  /// The rows of the fonts that can be chosen for [slot]: the bundled
  /// families offered there, then every downloaded Google Font, then the
  /// device's own where it is offered, with a check on the one in use.
  static List<Widget> fontRows(
    BuildContext context,
    KansoFonts fonts,
    KansoFontSlot slot,
  ) {
    final l10n = context.l10n;
    final bloc = context.read<AppearanceBloc>();
    final (choices, chosen, sample) = switch (slot) {
      KansoFontSlot.interface => (
        KansoFontFamily.interfaceChoices,
        fonts.interfaceFont,
        l10n.fontsSampleText,
      ),
      KansoFontSlot.editor => (
        KansoFontFamily.editorChoices,
        fonts.editorFont,
        l10n.fontsSampleText,
      ),
      KansoFontSlot.preview => (
        KansoFontFamily.previewChoices,
        fonts.previewFont,
        l10n.fontsSampleText,
      ),
      KansoFontSlot.code => (
        KansoFontFamily.codeChoices,
        fonts.codeFont,
        l10n.fontsSampleCode,
      ),
    };
    final google = fonts.googleFontFor(slot);
    AppearanceEvent pick(KansoFontFamily font) => switch (slot) {
      KansoFontSlot.interface => AppearanceEvent.interfaceFontChanged(font),
      KansoFontSlot.editor => AppearanceEvent.editorFontChanged(font),
      KansoFontSlot.preview => AppearanceEvent.previewFontChanged(font),
      KansoFontSlot.code => AppearanceEvent.codeFontChanged(font),
    };
    Widget bundled(KansoFontFamily font) => FontChoiceRow(
      font: font,
      sample: sample,
      selected: google == null && font == chosen,
      onTap: () => bloc.add(pick(font)),
    );
    return <Widget>[
      for (final font in choices)
        if (font != KansoFontFamily.system) bundled(font),
      for (final family in fonts.downloadedGoogleFonts)
        FontChoiceRow.downloaded(
          family: family,
          sample: sample,
          selected: google == family,
          onTap: () => bloc.add(AppearanceEvent.googleFontChosen(slot, family)),
        ),
      if (choices.contains(KansoFontFamily.system))
        bundled(KansoFontFamily.system),
    ];
  }
}
