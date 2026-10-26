// Kanso — kanso_sheet_page.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../configs/kanso_motion.dart';
import '../configs/kanso_sizes.dart';
import '../views/widgets/kanso_scrim.dart';
import '../views/widgets/kanso_sheet.dart';

/// A page that shows its [child] in a [KansoSheet] over a veil, on top of
/// what is below it: a tablet's settings.
///
/// A tap on the veil, or the system's back, closes it.
class KansoSheetPage<T> extends Page<T> {
  const KansoSheetPage({
    required this.child,
    super.key,
    super.name,
    super.arguments,
    super.restorationId,
  });

  final Widget child;

  @override
  Route<T> createRoute(BuildContext context) {
    return KansoSheetRoute<T>(page: this);
  }
}

/// The route of a [KansoSheetPage].
class KansoSheetRoute<T> extends PopupRoute<T> {
  KansoSheetRoute({required KansoSheetPage<T> page}) : super(settings: page);

  KansoSheetPage<T> get page => settings as KansoSheetPage<T>;

  @override
  Duration get transitionDuration => KansoMotion.content;

  @override
  Duration get reverseTransitionDuration => KansoMotion.short;

  @override
  bool get barrierDismissible => true;

  // The veil is drawn with the sheet, in the app's own colour.
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
    return LayoutBuilder(
      builder: (context, constraints) {
        // The sheet floats a touch target clear of the top and bottom.
        final height = constraints.maxHeight - 2 * KansoSizes.touchTarget;
        return Center(
          child: KansoSheet(
            width: constraints.maxWidth < KansoSizes.sheetWidth
                ? constraints.maxWidth
                : KansoSizes.sheetWidth,
            height: height > 0 ? height : constraints.maxHeight,
            child: page.child,
          ),
        );
      },
    );
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
        FadeTransition(
          opacity: curved,
          child: ScaleTransition(
            scale: Tween<double>(begin: 0.98, end: 1).animate(curved),
            child: child,
          ),
        ),
      ],
    );
  }
}
