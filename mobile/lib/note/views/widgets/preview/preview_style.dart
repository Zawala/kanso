// Kanso — preview_style.dart
// Created 2026-10-30.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/widgets.dart';

import '../../../../core/bridge/api/types.dart';
import '../../../../core/configs/kanso_typography.dart';

/// `0xRRGGBB`, as the core writes a colour, opaque.
Color previewColor(int rgb) => Color(0xFF000000 | (rgb & 0xFFFFFF));

/// [top] laid over [under] at [alpha], as the page mixes its quiet colours.
Color previewOver(Color top, Color under, double alpha) =>
    Color.lerp(under, top, alpha)!;

/// The look of a note's preview: the fitted preview palette's colours and
/// the note's fonts, sized and spaced as the core's page is
/// (`kanso-render`'s `stylesheet()` and `typography.rs`).
@immutable
class PreviewStyle {
  PreviewStyle({
    required this.palette,
    required KansoTypography typography,
    Color? text,
  }) : text = text ?? previewColor(palette.text),
       _body = typography.previewBody,
       _mono = typography.mono;

  PreviewStyle._copy(PreviewStyle of, this.text)
    : palette = of.palette,
      _body = of._body,
      _mono = of._mono;

  /// The colours the note is drawn in.
  final PaletteDto palette;

  /// The colour of running text here: the page's, a quote's, a ticked
  /// task's.
  final Color text;

  final TextStyle _body;
  final TextStyle _mono;

  /// The same look with running text in [color].
  PreviewStyle withText(Color color) => PreviewStyle._copy(this, color);

  /// The body text size, in points.
  double get size => _body.fontSize ?? 18;

  /// How much the note's text is scaled from the page's 18 points.
  double get scale => size / 18;

  Color get background => previewColor(palette.background);
  Color get muted => previewColor(palette.muted);
  Color get accent => previewColor(palette.accent);
  Color get border => previewColor(palette.border);
  Color get surface => previewColor(palette.surface);
  Color get quote => previewColor(palette.quote);
  Color get taskDone => previewColor(palette.taskDone);
  Color get headingColor => previewColor(palette.heading ?? palette.text);
  Color get codeText => previewColor(palette.codeText ?? palette.text);
  Color get tableHead =>
      previewColor(palette.tableHeaderBackground ?? palette.surface);

  /// A code block's edge: the border over the surface.
  Color get quiet => previewOver(border, surface, 0.6);

  /// A quote's bar: the quote colour over the page.
  Color get quoteBar => previewOver(quote, background, 0.45);

  /// The underline of a link: the accent over the page.
  Color get linkUnderline => previewOver(accent, background, 0.4);

  /// The room below a paragraph, a list, a table or a code block.
  double get gap => 0.9 * size;

  /// Running text.
  TextStyle get body => _body.copyWith(color: text);

  /// Running text [ratio] times as large.
  TextStyle sized(double ratio) => body.copyWith(fontSize: size * ratio);

  /// A heading's size relative to the body, and the room above it relative
  /// to the heading itself.
  static (double scale, double gap) headingMetrics(int level) =>
      switch (level) {
        1 => (1.8, 1.1),
        2 => (4 / 3, 1.6),
        3 => (17 / 15, 1.4),
        4 => (1.0, 1.4),
        5 => (14 / 15, 1.4),
        _ => (13 / 15, 1.4),
      };

  /// The heading of [level].
  TextStyle heading(int level) {
    final (ratio, _) = headingMetrics(level);
    return _body.copyWith(
      fontSize: size * ratio,
      fontWeight: level == 1 ? FontWeight.w700 : FontWeight.w600,
      height: level == 1 ? 1.25 : 1.35,
      color: level == 6 ? taskDone : headingColor,
    );
  }

  /// A link's text.
  TextStyle get link => TextStyle(
    color: accent,
    decoration: TextDecoration.underline,
    decorationColor: linkUnderline,
  );

  /// Code in a line of text: the code font at the text's size.
  TextStyle get inlineCode => TextStyle(
    fontFamily: _mono.fontFamily,
    fontFamilyFallback: _mono.fontFamilyFallback,
    backgroundColor: previewColor(palette.inlineCodeBackground),
    color: palette.inlineCodeText == null
        ? null
        : previewColor(palette.inlineCodeText!),
  );

  /// A code block's text: the code font at 13 points, scaled with the
  /// note's text.
  TextStyle get code => _mono.copyWith(
    fontSize: 13 * scale,
    height: 1.6,
    fontWeight: FontWeight.w400,
    color: codeText,
  );

  /// A label set small in the note's font: a code block's language.
  TextStyle get label => _body.copyWith(
    fontSize: 12 * scale,
    height: 16 / 12,
    color: muted,
    fontWeight: FontWeight.w400,
  );
}

/// What the blocks of a preview share: their look, what a tap on an address
/// does, and the key each heading is found by.
class PreviewScope extends InheritedWidget {
  const PreviewScope({
    required this.style,
    required this.onLink,
    required this.headingKey,
    required super.child,
    this.depth = 0,
    super.key,
  });

  final PreviewStyle style;

  /// Hands the address tapped to the preview's BLoC.
  final ValueChanged<String> onLink;

  /// The key of the heading whose id is given, for scrolling to it.
  final GlobalKey Function(String id) headingKey;

  /// How deep in lists the blocks are, for the bullets.
  final int depth;

  static PreviewScope of(BuildContext context) =>
      context.dependOnInheritedWidgetOfExactType<PreviewScope>()!;

  /// The blocks under [child] in [style], or [depth] lists deep.
  static Widget nested(
    BuildContext context, {
    required Widget child,
    PreviewStyle? style,
    int? depth,
  }) {
    final scope = of(context);
    return PreviewScope(
      style: style ?? scope.style,
      onLink: scope.onLink,
      headingKey: scope.headingKey,
      depth: depth ?? scope.depth,
      child: child,
    );
  }

  @override
  bool updateShouldNotify(PreviewScope oldWidget) =>
      style != oldWidget.style ||
      onLink != oldWidget.onLink ||
      depth != oldWidget.depth;
}
