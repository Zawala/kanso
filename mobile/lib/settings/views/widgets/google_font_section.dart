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
import '../../../core/views/widgets/kanso_text_input.dart';

/// Lets someone download a Google Font. It is used for writing and reading
/// at once, and joins every font list so it can be chosen anywhere.
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
    return BlocConsumer<AppearanceBloc, AppearanceState>(
      listenWhen: (before, after) =>
          before.googleFontLoading &&
          !after.googleFontLoading &&
          after.googleFontError == null,
      // The font is downloaded: the field is ready for the next one.
      listener: (context, state) => controller.clear(),
      buildWhen: (before, after) =>
          before.googleFontLoading != after.googleFontLoading ||
          before.googleFontError != after.googleFontError ||
          before.googleFontOffline != after.googleFontOffline,
      builder: (context, state) {
        final loading = state.googleFontLoading;
        return KansoGroupedSection(
          title: l10n.googleFontSection,
          footer: l10n.googleFontFooter,
          children: <Widget>[
            Padding(
              padding: const EdgeInsets.all(KansoSpacing.sm),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.stretch,
                children: <Widget>[
                  Row(
                    children: <Widget>[
                      Expanded(
                        child: KansoTextInput(
                          controller: controller,
                          hint: l10n.googleFontHint,
                          semanticLabel: l10n.googleFontHint,
                          autocorrect: false,
                          enabled: !loading,
                          textCapitalization: TextCapitalization.words,
                          onSubmitted: (_) =>
                              request(context, loading: loading),
                        ),
                      ),
                      const SizedBox(width: KansoSpacing.xs),
                      KansoButton(
                        label: loading
                            ? l10n.googleFontLoading
                            : l10n.googleFontDownload,
                        expanded: false,
                        onPressed: loading ? null : () => request(context),
                      ),
                    ],
                  ),
                  if (!loading && state.googleFontError != null) ...<Widget>[
                    const SizedBox(height: KansoSpacing.xs),
                    Text(
                      state.googleFontOffline
                          ? l10n.googleFontOffline
                          : l10n.googleFontFailed,
                      style: context.typography.secondary.copyWith(
                        color: context.colors.danger,
                      ),
                    ),
                  ],
                ],
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
