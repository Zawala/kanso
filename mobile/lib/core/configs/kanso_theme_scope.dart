// Kanso — kanso_theme_scope.dart
// Created 2026-10-29.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/widgets.dart';

import 'kanso_colors.dart';
import 'kanso_palette.dart';
import 'kanso_syntax_colors.dart';
import 'kanso_typography.dart';

/// The tokens a Kanso page draws with: its colours, its Markdown and code
/// colours, its type scale, and whether it is a dark theme.
///
/// It is what the app hands down instead of a framework theme: a page reads
/// `context.colors` and `context.typography` and never a default that some
/// widget library chose.
@immutable
class KansoThemeData {
  const KansoThemeData({
    required this.brightness,
    required this.colors,
    required this.syntax,
    required this.typography,
  });

  final Brightness brightness;
  final KansoColors colors;
  final KansoSyntaxColors syntax;
  final KansoTypography typography;

  bool get isDark => brightness == Brightness.dark;

  /// The theme [t] of the way from this one to [other], for a theme that
  /// is changing: the colours blend and the type scale snaps at the end.
  KansoThemeData lerp(KansoThemeData other, double t) {
    if (t <= 0) {
      return this;
    }
    if (t >= 1) {
      return other;
    }
    return KansoThemeData(
      brightness: other.brightness,
      colors: colors.lerp(other.colors, t),
      syntax: syntax.lerp(other.syntax, t),
      typography: other.typography,
    );
  }
}

/// Hands a [KansoThemeData] down to everything under it.
///
/// The `colors`, `typography` and `syntaxColors` extensions on a
/// `BuildContext` read it; nothing in the app reads a framework theme.
class KansoThemeScope extends InheritedWidget {
  const KansoThemeScope({required this.theme, required super.child, super.key});

  final KansoThemeData theme;

  /// The theme in force above [context].
  ///
  /// A widget drawn outside an app, in a test or a preview, is given Kanso
  /// Light rather than failing: a page must not crash because it is alone.
  static KansoThemeData of(BuildContext context) {
    final scope = context.dependOnInheritedWidgetOfExactType<KansoThemeScope>();
    if (scope != null) {
      return scope.theme;
    }
    final inherited = context.getInheritedWidgetOfExactType<KansoThemeScope>();
    return inherited?.theme ?? fallback;
  }

  /// The theme a widget gets when no scope is above it.
  static final KansoThemeData fallback = KansoThemeData(
    brightness: Brightness.light,
    colors: KansoPalette.kansoLight.colors,
    syntax: KansoPalette.kansoLight.syntax,
    typography: KansoTypography.fromColors(KansoPalette.kansoLight.colors),
  );

  @override
  bool updateShouldNotify(KansoThemeScope oldWidget) =>
      oldWidget.theme != theme;
}

/// Gives its subtree [theme], and moves to a new one by blending the
/// colours rather than swapping them in a frame.
///
/// The blend is the app's own: it lerps the colour tokens, and hands the
/// new type scale down at once because text cannot be half a size. With
/// animations turned off it changes at once.
class KansoThemeTransition extends StatefulWidget {
  const KansoThemeTransition({
    required this.theme,
    required this.child,
    this.duration = const Duration(milliseconds: 220),
    super.key,
  });

  final KansoThemeData theme;
  final Duration duration;
  final Widget child;

  @override
  State<KansoThemeTransition> createState() => _KansoThemeTransitionState();
}

class _KansoThemeTransitionState extends State<KansoThemeTransition>
    with SingleTickerProviderStateMixin {
  late final AnimationController controller;
  late KansoThemeData from;
  late KansoThemeData to;

  @override
  void initState() {
    super.initState();
    from = widget.theme;
    to = widget.theme;
    controller = AnimationController(vsync: this, value: 1);
  }

  @override
  void didUpdateWidget(KansoThemeTransition oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (oldWidget.theme == widget.theme) {
      return;
    }
    // Start again from wherever the last blend had got to, so changing the
    // theme twice in quick succession does not snap back.
    from = _current;
    to = widget.theme;
    controller
      ..duration = widget.duration
      ..forward(from: 0);
  }

  KansoThemeData get _current => from.lerp(to, controller.value);

  @override
  void dispose() {
    controller.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    if (MediaQuery.disableAnimationsOf(context)) {
      return KansoThemeScope(theme: widget.theme, child: widget.child);
    }
    return AnimatedBuilder(
      animation: controller,
      builder: (context, _) => KansoThemeScope(
        theme: controller.isAnimating ? _current : widget.theme,
        child: widget.child,
      ),
    );
  }
}
