// Kanso — code_block.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/configs/kanso_radii.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../models/data/mock_code_token.dart';
import '../../models/data/mock_token_kind.dart';
import '../../utils/token_colors.dart';

/// A fenced code block in the code font, coloured with the theme's syntax
/// colours. [fence] adds the ``` lines of the Markdown source.
class CodeBlock extends StatelessWidget {
  const CodeBlock({
    required this.lines,
    this.language = '',
    this.fence = false,
    super.key,
  });

  final List<List<MockCodeToken>> lines;
  final String language;
  final bool fence;

  @override
  Widget build(BuildContext context) {
    final syntax = context.syntaxColors;
    final mono = context.typography.mono;
    final marker = mono.copyWith(color: syntax.blockMarker);
    return Container(
      width: double.infinity,
      padding: const EdgeInsets.all(KansoSpacing.sm),
      decoration: BoxDecoration(
        color: syntax.codeBackground,
        borderRadius: KansoRadii.mediumBorder,
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: <Widget>[
          if (fence) Text('```$language', style: marker),
          for (final line in lines)
            Text.rich(
              TextSpan(
                style: mono,
                children: <InlineSpan>[
                  for (final token in line)
                    TextSpan(
                      text: token.text,
                      style: TextStyle(
                        color: token.kind.colorIn(syntax),
                        fontStyle: token.kind == MockTokenKind.comment
                            ? FontStyle.italic
                            : null,
                      ),
                    ),
                ],
              ),
              softWrap: false,
              overflow: TextOverflow.clip,
            ),
          if (fence) Text('```', style: marker),
        ],
      ),
    );
  }
}
