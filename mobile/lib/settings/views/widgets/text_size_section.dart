// Kanso — text_size_section.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_grouped_section.dart';
import 'continuous_text_size_control.dart';

/// The text size control and a paragraph sample in the selected note font.
///
/// It serves the editor's text size and the preview's: [title] and [footer]
/// say which, and default to the editor's.
class TextSizeSection extends StatelessWidget {
  const TextSizeSection({
    required this.selected,
    required this.onChanged,
    required this.sampleStyle,
    this.title,
    this.footer,
    super.key,
  });

  final String? title;
  final String? footer;
  final TextStyle sampleStyle;

  final double selected;
  final ValueChanged<double> onChanged;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    return KansoGroupedSection(
      title: title ?? l10n.fontsTextSizeSection,
      footer: footer ?? l10n.fontsTextSizeFooter,
      children: <Widget>[
        Padding(
          padding: const EdgeInsets.all(KansoSpacing.sm),
          child: ContinuousTextSizeControl(
            value: selected,
            onChanged: onChanged,
          ),
        ),
        Padding(
          padding: const EdgeInsets.fromLTRB(
            KansoSpacing.md,
            KansoSpacing.xs,
            KansoSpacing.md,
            KansoSpacing.md,
          ),
          child: Text(
            l10n.fontsTextSizeSample,
            style: sampleStyle,
          ),
        ),
      ],
    );
  }
}
