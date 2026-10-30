// Kanso — kanso_page_route.dart.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/cupertino.dart'
    show CupertinoPage, CupertinoRouteTransitionMixin;
import 'package:flutter/foundation.dart';
import 'package:flutter/widgets.dart';
import 'package:go_router/go_router.dart';

import '../../configs/kanso_layout.dart';
import '../../configs/kanso_motion.dart';

/// A route page for GoRouter: native on iPhone, owned elsewhere.
///
/// GoRouter's nested shell navigators need a page implemented by GoRouter for
/// imperative pushes inside a shell. A custom [Page] works for ordinary stacks
/// but silently prevents a child route from being added under the settings
/// shell. Use [CupertinoPage] on iPhone and GoRouter's
/// [CustomTransitionPage] elsewhere, keeping Kanso's transitions in the app.
Page<T> kansoPage<T>({
  required LocalKey key,
  required Widget child,
  bool fullscreenDialog = false,
}) {
  if (defaultTargetPlatform == TargetPlatform.iOS) {
    return CupertinoPage<T>(
      key: key,
      child: child,
      fullscreenDialog: fullscreenDialog,
    );
  }
  return CustomTransitionPage<T>(
    key: key,
    child: child,
    fullscreenDialog: fullscreenDialog,
    reverseTransitionDuration: KansoMotion.short,
    transitionsBuilder: (context, animation, secondaryAnimation, child) =>
        _kansoPageTransition(
          context,
          animation,
          secondaryAnimation,
          child,
          fullscreenDialog: fullscreenDialog,
        ),
  );
}

/// The app's imperative page route: swipe back on an iPhone, owned transitions
/// elsewhere.
class KansoPageRoute<T> extends PageRoute<T>
    with CupertinoRouteTransitionMixin<T> {
  KansoPageRoute({
    required this.builder,
    super.settings,
    super.fullscreenDialog,
  });

  final WidgetBuilder builder;

  @override
  String? get title => null;

  @override
  bool get maintainState => true;

  @override
  Duration get transitionDuration => KansoMotion.content;

  @override
  Duration get reverseTransitionDuration => KansoMotion.short;

  @override
  Widget buildContent(BuildContext context) => builder(context);

  @override
  Widget buildTransitions(
    BuildContext context,
    Animation<double> animation,
    Animation<double> secondaryAnimation,
    Widget child,
  ) {
    // Keep Cupertino's gesture lifecycle, even with reduced motion, so an
    // iPhone can still swipe back and interrupted drags remain well behaved.
    if (defaultTargetPlatform == TargetPlatform.iOS && _isPhone(context)) {
      return super.buildTransitions(
        context,
        animation,
        secondaryAnimation,
        child,
      );
    }
    return _kansoPageTransition(
      context,
      animation,
      secondaryAnimation,
      child,
      fullscreenDialog: fullscreenDialog,
    );
  }
}

bool _isPhone(BuildContext context) =>
    MediaQuery.sizeOf(context).shortestSide < KansoLayout.tabletShortestSide;

Widget _kansoPageTransition(
  BuildContext context,
  Animation<double> animation,
  Animation<double> secondaryAnimation,
  Widget child, {
  required bool fullscreenDialog,
}) {
  if (MediaQuery.disableAnimationsOf(context)) return child;

  final isPhone = _isPhone(context);
  final curved = animation.drive(
    CurveTween(curve: isPhone ? KansoMotion.enter : KansoMotion.standard),
  );
  if (!isPhone) return FadeTransition(opacity: curved, child: child);

  return FadeTransition(
    opacity: curved,
    child: SlideTransition(
      position: Tween<Offset>(
        begin: fullscreenDialog ? const Offset(0, .06) : const Offset(.05, 0),
        end: Offset.zero,
      ).animate(curved),
      child: child,
    ),
  );
}
