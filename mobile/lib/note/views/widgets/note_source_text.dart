// Kanso — note_source_text.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/bridge/api/types.dart';
import '../../../core/utils/context_extensions.dart';
import '../../models/data/source_line.dart';

/// A note's Markdown as the editor will show it, read-only: its text in the
/// editor font, each run in the colour, slant and weight the theme gives
/// it. It is a sliver, laid out line by line as it scrolls into view, so
/// the size of the note costs nothing until it is read.
class NoteSourceText extends StatelessWidget {
  const NoteSourceText({required this.text, required this.lines, super.key});

  /// The note's text.
  final String text;

  /// The text cut into lines, each with its styled runs.
  final List<SourceLine> lines;

  @override
  Widget build(BuildContext context) {
    final syntax = context.syntaxColors;
    final base = context.typography.editorBody.copyWith(color: syntax.text);
    final blankHeight =
        MediaQuery.textScalerOf(context).scale(base.fontSize ?? 15) *
        (base.height ?? 1.4);
    return SliverList.builder(
      itemCount: lines.length,
      itemBuilder: (context, at) {
        final line = lines[at];
        if (line.end == line.start) {
          return ExcludeSemantics(child: SizedBox(height: blankHeight));
        }
        final spans = <InlineSpan>[];
        var cursor = line.start;
        for (final run in line.runs) {
          if (run.range.start > cursor) {
            spans.add(
              TextSpan(text: text.substring(cursor, run.range.start)),
            );
          }
          spans.add(
            TextSpan(
              text: text.substring(run.range.start, run.range.end),
              style: styleOf(run),
            ),
          );
          cursor = run.range.end;
        }
        if (cursor < line.end) {
          spans.add(TextSpan(text: text.substring(cursor, line.end)));
        }
        return Text.rich(TextSpan(style: base, children: spans));
      },
    );
  }

  /// The style a run asks for: only what it names, over the plain text.
  static TextStyle styleOf(StyledRun run) {
    final weight = run.weight;
    return TextStyle(
      color: run.color == null ? null : Color(0xFF000000 | run.color!),
      fontStyle: run.italic ?? false ? FontStyle.italic : null,
      fontWeight: weight == null
          ? null
          : FontWeight.values[((weight / 100).round() - 1).clamp(0, 8)],
    );
  }
}
