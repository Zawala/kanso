// Kanso — google_font_section.dart
// Created 2026-10-29.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../core/configs/kanso_spacing.dart';
import '../../../core/state/appearance/appearance_bloc.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_button.dart';
import '../../../core/views/widgets/kanso_grouped_section.dart';
import '../../../core/views/widgets/kanso_text_field.dart';

/// Lets someone download a Google Font and use it for note writing and reading.
class GoogleFontSection extends StatefulWidget {
  const GoogleFontSection({super.key});

  @override
  State<GoogleFontSection> createState() => GoogleFontSectionState();
}

/// Owns the family name field used by [GoogleFontSection].
class GoogleFontSectionState extends State<GoogleFontSection> {
  late final TextEditingController controller;

  @override
  void initState() {
    super.initState();
    controller = TextEditingController();
  }

  @override
  void dispose() {
    controller.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    return BlocBuilder<AppearanceBloc, AppearanceState>(
      buildWhen: (before, after) =>
          before.fonts != after.fonts ||
          before.googleFontLoading != after.googleFontLoading ||
          before.googleFontError != after.googleFontError ||
          before.googleFontOffline != after.googleFontOffline,
      builder: (context, state) {
        final active =
            state.fonts.editorGoogleFont ?? state.fonts.previewGoogleFont;
        return KansoGroupedSection(
          title: l10n.googleFontSection,
          footer: active == null
              ? l10n.googleFontFooter
              : l10n.googleFontActive(active),
          children: <Widget>[
            Padding(
              padding: const EdgeInsets.fromLTRB(
                KansoSpacing.md,
                KansoSpacing.sm,
                KansoSpacing.md,
                KansoSpacing.xs,
              ),
              child: KansoTextField(
                controller: controller,
                hint: l10n.googleFontHint,
                semanticLabel: l10n.googleFontHint,
                textInputAction: TextInputAction.done,
                onSubmitted: (_) => request(
                  context,
                  loading: state.googleFontLoading,
                ),
              ),
            ),
            Padding(
              padding: const EdgeInsets.fromLTRB(
                KansoSpacing.md,
                0,
                KansoSpacing.md,
                KansoSpacing.sm,
              ),
              child: KansoButton(
                label: state.googleFontLoading
                    ? l10n.googleFontLoading
                    : l10n.googleFontDownload,
                onPressed: state.googleFontLoading
                    ? null
                    : () => request(context),
              ),
            ),
            if (state.googleFontLoading)
              const Padding(
                padding: EdgeInsets.fromLTRB(
                  KansoSpacing.md,
                  0,
                  KansoSpacing.md,
                  KansoSpacing.md,
                ),
                child: SizedBox.square(
                  dimension: 16,
                  child: CircularProgressIndicator(strokeWidth: 2),
                ),
              ),
            if (state.googleFontError != null)
              Padding(
                padding: const EdgeInsets.fromLTRB(
                  KansoSpacing.md,
                  0,
                  KansoSpacing.md,
                  KansoSpacing.md,
                ),
                child: Text(
                  state.googleFontOffline
                      ? l10n.googleFontOffline
                      : l10n.googleFontFailed,
                  style: context.typography.secondary.copyWith(
                    color: context.colors.danger,
                  ),
                ),
              ),
          ],
        );
      },
    );
  }

  void request(BuildContext context, {bool loading = false}) {
    final family = controller.text.trim();
    if (loading || family.isEmpty) {
      return;
    }
    context.read<AppearanceBloc>().add(
      AppearanceEvent.googleFontPicked(family),
    );
  }
}
