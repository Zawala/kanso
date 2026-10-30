// Kanso — preview_code.dart
// Created 2026-10-30.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../../core/bridge/api/types.dart';
import '../../../../core/configs/kanso_icons.dart';
import '../../../../core/utils/context_extensions.dart';
import '../../../../core/views/widgets/kanso_icon.dart';
import 'preview_style.dart';

/// A code block: its runs in the theme's code colours, on the surface, with
/// its language and a copy button (`kanso-copy:<copy>`). Long lines scroll
/// sideways.
class PreviewCode extends StatelessWidget {
  const PreviewCode({
    required this.runs,
    required this.copy,
    this.language,
    super.key,
  });

  final String? language;
  final List<PreviewCodeRun> runs;
  final int copy;

  @override
  Widget build(BuildContext context) {
    final style = PreviewScope.of(context).style;
    return PreviewCodeBox(
      language: language,
      copy: copy,
      child: Text.rich(
        TextSpan(
          style: style.code,
          children: <InlineSpan>[
            for (final run in runs)
              TextSpan(
                text: run.text,
                style: TextStyle(
                  color: run.color == null ? null : previewColor(run.color!),
                  fontWeight: run.bold ? FontWeight.w700 : null,
                  fontStyle: run.italic ? FontStyle.italic : null,
                ),
              ),
          ],
        ),
        softWrap: false,
      ),
    );
  }
}

/// A source that couldn't be drawn: its code, with a badge saying why (the
/// details on a long press) and, for a fence, a copy button.
class PreviewFailed extends StatelessWidget {
  const PreviewFailed({
    required this.badge,
    required this.detail,
    required this.code,
    this.language,
    this.copy,
    super.key,
  });

  final String badge;
  final String detail;
  final String? language;
  final String code;
  final int? copy;

  @override
  Widget build(BuildContext context) {
    final style = PreviewScope.of(context).style;
    return Stack(
      fit: StackFit.passthrough,
      children: <Widget>[
        PreviewCodeBox(
          language: language,
          copy: copy,
          child: Text(code, style: style.code, softWrap: false),
        ),
        Positioned(
          top: 7,
          left: 10,
          child: PreviewBadge(badge, detail: detail),
        ),
      ],
    );
  }
}

/// A query that can't be read: its text, with a badge, and why.
class PreviewQueryInvalid extends StatelessWidget {
  const PreviewQueryInvalid({
    required this.code,
    required this.message,
    super.key,
  });

  final String code;
  final String message;

  @override
  Widget build(BuildContext context) {
    final style = PreviewScope.of(context).style;
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      mainAxisSize: MainAxisSize.min,
      children: <Widget>[
        Stack(
          fit: StackFit.passthrough,
          children: <Widget>[
            PreviewCodeBox(
              child: Text(code, style: style.code, softWrap: false),
            ),
            Positioned(
              top: 7,
              left: 10,
              child: PreviewBadge(context.l10n.previewQueryBadge),
            ),
          ],
        ),
        SizedBox(height: 0.4 * style.size),
        Text(
          message,
          style: style.sized(0.8).copyWith(color: style.muted),
        ),
      ],
    );
  }
}

/// The page's block of code: the surface with a quiet edge and rounded
/// corners, room above for the [language] label and the copy button.
class PreviewCodeBox extends StatelessWidget {
  const PreviewCodeBox({
    required this.child,
    this.language,
    this.copy,
    super.key,
  });

  final Widget child;
  final String? language;

  /// The code's `kanso-copy:` number; no copy button when null.
  final int? copy;

  @override
  Widget build(BuildContext context) {
    final scope = PreviewScope.of(context);
    final style = scope.style;
    final copy = this.copy;
    final language = this.language;
    return Container(
      decoration: BoxDecoration(
        color: style.surface,
        border: Border.all(color: style.quiet),
        borderRadius: BorderRadius.circular(8),
      ),
      child: Stack(
        children: <Widget>[
          Padding(
            padding: const EdgeInsets.fromLTRB(0, 34, 0, 14),
            child: SingleChildScrollView(
              scrollDirection: Axis.horizontal,
              padding: const EdgeInsets.symmetric(horizontal: 16),
              child: child,
            ),
          ),
          if (language != null && language.isNotEmpty)
            Positioned(
              top: 7,
              right: copy == null ? 12 : 44,
              child: Text(language, style: style.label),
            ),
          if (copy != null)
            Positioned(
              top: 0,
              right: 0,
              child: _CopyButton(onTap: () => scope.onLink('kanso-copy:$copy')),
            ),
        ],
      ),
    );
  }
}

/// A small label over a block: why a source wasn't drawn, or "Query".
class PreviewBadge extends StatelessWidget {
  const PreviewBadge(this.text, {this.detail, super.key});

  final String text;

  /// More about it, shown on a long press.
  final String? detail;

  @override
  Widget build(BuildContext context) {
    final style = PreviewScope.of(context).style;
    final badge = Container(
      padding: const EdgeInsets.symmetric(horizontal: 7, vertical: 1),
      decoration: BoxDecoration(
        color: style.tableHead,
        border: Border.all(color: style.border),
        borderRadius: BorderRadius.circular(5),
      ),
      child: Text(text, style: style.sized(0.72).copyWith(color: style.muted)),
    );
    final detail = this.detail;
    return detail == null || detail.isEmpty
        ? badge
        : Tooltip(message: detail, child: badge);
  }
}

class _CopyButton extends StatelessWidget {
  const _CopyButton({required this.onTap});

  final VoidCallback onTap;

  @override
  Widget build(BuildContext context) {
    final style = PreviewScope.of(context).style;
    return Semantics(
      container: true,
      button: true,
      label: context.l10n.previewCopyCode,
      onTap: onTap,
      excludeSemantics: true,
      child: GestureDetector(
        behavior: HitTestBehavior.opaque,
        onTap: onTap,
        child: SizedBox.square(
          dimension: 44,
          child: Center(
            child: KansoIcon(KansoIcons.copy, size: 16, color: style.muted),
          ),
        ),
      ),
    );
  }
}
