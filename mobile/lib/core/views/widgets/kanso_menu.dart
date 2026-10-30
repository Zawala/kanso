// Kanso — kanso_menu.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:math' as math;

import 'package:flutter/material.dart';

import '../../configs/kanso_effects.dart';
import '../../configs/kanso_icons.dart';
import '../../configs/kanso_motion.dart';
import '../../configs/kanso_radii.dart';
import '../../configs/kanso_sizes.dart';
import '../../configs/kanso_spacing.dart';
import '../../utils/context_extensions.dart';
import 'kanso_icon.dart';
import 'kanso_pressable.dart';
import 'kanso_scrim.dart';

/// One row of a [showKansoMenu]: a [label], and what choosing it returns.
///
/// [checked] draws a check in the accent colour before the label, for a
/// menu that says which one is in use; [enabled] false draws it dimmed and
/// does nothing; [divider] draws a hairline above it.
class KansoMenuEntry<T> {
  const KansoMenuEntry(
    this.value,
    this.label, {
    this.enabled = true,
    this.checked = false,
    this.divider = false,
    this.icon,
  });

  final T value;
  final String label;
  final bool enabled;
  final bool checked;
  final bool divider;
  final KansoIcons? icon;
}

/// Shows [entries] in the app's own menu, its top left corner at [at], and
/// completes with what was chosen, or null.
///
/// It is the desktop's context menu brought to a phone: a small rounded
/// surface with a hairline, one row per choice, and a veil that closes it.
/// A menu taller than the screen scrolls rather than running off it.
Future<T?> showKansoMenu<T>(
  BuildContext context,
  Offset at,
  List<KansoMenuEntry<T>> entries,
) {
  return Navigator.of(context, rootNavigator: true).push<T>(
    KansoMenuRoute<T>(at: at, entries: entries),
  );
}

/// The route of [showKansoMenu].
class KansoMenuRoute<T> extends PopupRoute<T> {
  KansoMenuRoute({required this.at, required this.entries});

  final Offset at;
  final List<KansoMenuEntry<T>> entries;

  @override
  Duration get transitionDuration => KansoMotion.fast;

  @override
  Duration get reverseTransitionDuration => KansoMotion.fast;

  @override
  bool get barrierDismissible => true;

  // The veil is drawn by the page, in the app's own colour.
  @override
  Color? get barrierColor => null;

  @override
  String? get barrierLabel => null;

  @override
  Widget buildPage(
    BuildContext context,
    Animation<double> animation,
    Animation<double> secondaryAnimation,
  ) {
    return _KansoMenuView<T>(at: at, entries: entries);
  }

  @override
  Widget buildTransitions(
    BuildContext context,
    Animation<double> animation,
    Animation<double> secondaryAnimation,
    Widget child,
  ) {
    final curved = animation.drive(CurveTween(curve: KansoMotion.enter));
    return Stack(
      children: <Widget>[
        Positioned.fill(
          child: FadeTransition(
            opacity: curved,
            child: GestureDetector(
              behavior: HitTestBehavior.opaque,
              onTap: () => Navigator.of(context).maybePop(),
              child: const KansoScrim(),
            ),
          ),
        ),
        FadeTransition(opacity: curved, child: child),
      ],
    );
  }
}

/// The menu itself, placed inside the screen's padding.
class _KansoMenuView<T> extends StatelessWidget {
  const _KansoMenuView({required this.at, required this.entries});

  final Offset at;
  final List<KansoMenuEntry<T>> entries;

  /// The widest the menu grows; a longer label is cut short.
  static const double maxWidth = 260;

  /// The closest the menu comes to an edge of the screen.
  static const double margin = KansoSpacing.sm;

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
    final screen = MediaQuery.sizeOf(context);
    final padding = MediaQuery.viewPaddingOf(context);
    final left = at.dx.clamp(
      margin + padding.left,
      math.max<double>(
        margin,
        screen.width - maxWidth - margin - padding.right,
      ),
    );
    final top = at.dy.clamp(
      margin + padding.top,
      math.max<double>(margin, screen.height - margin - padding.bottom),
    );
    final available = math.max<double>(
      120,
      screen.height - top - margin - padding.bottom,
    );
    return Stack(
      children: <Widget>[
        Positioned(
          left: left,
          top: top,
          width: maxWidth,
          child: ConstrainedBox(
            constraints: BoxConstraints(maxHeight: available),
            child: DecoratedBox(
              decoration: BoxDecoration(
                color: colors.elevated,
                borderRadius: KansoRadii.mediumBorder,
                border: Border.all(color: colors.divider),
                boxShadow: <BoxShadow>[
                  BoxShadow(
                    color: Colors.black.withValues(alpha: 0.14),
                    blurRadius: 24,
                    offset: const Offset(0, 8),
                  ),
                ],
              ),
              child: ClipRRect(
                borderRadius: KansoRadii.mediumBorder,
                child: SingleChildScrollView(
                  child: Column(
                    mainAxisSize: MainAxisSize.min,
                    crossAxisAlignment: CrossAxisAlignment.stretch,
                    children: <Widget>[
                      for (var i = 0; i < entries.length; i++) ...<Widget>[
                        if (entries[i].divider && i > 0)
                          SizedBox(
                            height: 1,
                            child: ColoredBox(color: colors.divider),
                          ),
                        _KansoMenuRow<T>(entry: entries[i]),
                      ],
                    ],
                  ),
                ),
              ),
            ),
          ),
        ),
      ],
    );
  }
}

/// One row of the menu.
class _KansoMenuRow<T> extends StatelessWidget {
  const _KansoMenuRow({required this.entry});

  final KansoMenuEntry<T> entry;

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
    final glyph = entry.icon;
    return KansoPressable(
      onTap: entry.enabled
          ? () => Navigator.of(context).pop<T>(entry.value)
          : null,
      child: Opacity(
        opacity: entry.enabled ? 1 : KansoEffects.disabled,
        child: ConstrainedBox(
          constraints: const BoxConstraints(minHeight: KansoSizes.touchTarget),
          child: Padding(
            padding: const EdgeInsetsDirectional.symmetric(
              horizontal: KansoSpacing.md,
              vertical: KansoSpacing.xs,
            ),
            child: Row(
              children: <Widget>[
                SizedBox(
                  width: KansoSizes.icon,
                  child: entry.checked
                      ? KansoIcon(KansoIcons.check, color: colors.accent)
                      : glyph == null
                      ? null
                      : KansoIcon(glyph, color: colors.textSecondary),
                ),
                const SizedBox(width: KansoSpacing.xs),
                Expanded(
                  child: Text(
                    entry.label,
                    maxLines: 2,
                    overflow: TextOverflow.ellipsis,
                    style: context.typography.body.copyWith(
                      color: entry.enabled ? colors.text : colors.textMuted,
                    ),
                  ),
                ),
              ],
            ),
          ),
        ),
      ),
    );
  }
}
