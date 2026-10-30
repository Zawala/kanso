// Kanso — kanso_test_app.dart
// Created 2026-10-29.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/foundation.dart';
import 'package:flutter/widgets.dart';
import 'package:kanso/core/configs/kanso_theme.dart';
import 'package:kanso/core/configs/kanso_theme_scope.dart';
import 'package:kanso/core/utils/kanso_scroll_behavior.dart';
import 'package:kanso/core/views/widgets/kanso_page_route.dart';
import 'package:kanso/gen/l10n/app_localizations.dart';

/// A Kanso app for a test: the real theme tokens, the real strings and
/// Kanso's own scrolling, with [home] on top.
///
/// It replaces the framework app the tests used to pump, so what a test
/// draws goes through the same [KansoThemeScope] a device does. [brightness]
/// or [theme] chooses the palette; [platform] overrides the platform the
/// page sees, for a test that checks a platform-shaped detail.
class KansoTestApp extends StatelessWidget {
  const KansoTestApp({
    required this.home,
    this.brightness = Brightness.light,
    this.theme,
    this.platform,
    this.builder,
    super.key,
  });

  final Widget home;
  final Brightness brightness;

  /// Wraps [home] once inside the app's own [MediaQuery], for a test that
  /// changes the safe area or the text scale.
  final Widget Function(BuildContext context, Widget home)? builder;

  /// The tokens to draw with; the matching [brightness] theme when null.
  final KansoThemeData? theme;

  /// What `context.platform` answers, when a test needs one platform.
  final TargetPlatform? platform;

  @override
  Widget build(BuildContext context) {
    final selected = brightness == Brightness.dark
        ? KansoTheme.dark
        : KansoTheme.light;
    final tokens = theme ?? selected;
    return _KansoPlatformOverride(
      platform: platform,
      child: ScrollConfiguration(
        behavior: const KansoScrollBehavior(),
        child: WidgetsApp(
          debugShowCheckedModeBanner: false,
          color: tokens.colors.button,
          localizationsDelegates: AppLocalizations.localizationsDelegates,
          supportedLocales: AppLocalizations.supportedLocales,
          pageRouteBuilder: <T>(settings, pageBuilder) =>
              KansoPageRoute<T>(settings: settings, builder: pageBuilder),
          home: KansoTestScope(
            theme: tokens,
            child: switch (builder) {
              final wrap? => Builder(
                builder: (context) => wrap(context, home),
              ),
              null => home,
            },
          ),
        ),
      ),
    );
  }
}

/// The same shell as [KansoTestApp] around a router, for a test of routes.
class KansoTestRouterApp extends StatelessWidget {
  const KansoTestRouterApp({
    required this.routerConfig,
    this.brightness = Brightness.light,
    this.theme,
    this.platform,
    this.builder,
    super.key,
  });

  final RouterConfig<Object> routerConfig;
  final Brightness brightness;

  /// The tokens to draw with; the matching [brightness] theme when null.
  final KansoThemeData? theme;

  /// What `context.platform` answers, when a test needs one platform.
  final TargetPlatform? platform;

  /// Wraps the routed app, for a test that installs a repository scope
  /// inside the real strings and theme.
  final Widget Function(BuildContext context, Widget child)? builder;

  @override
  Widget build(BuildContext context) {
    final selected = brightness == Brightness.dark
        ? KansoTheme.dark
        : KansoTheme.light;
    final tokens = theme ?? selected;
    return ScrollConfiguration(
      behavior: const KansoScrollBehavior(),
      child: _KansoPlatformOverride(
        platform: platform,
        child: WidgetsApp.router(
          debugShowCheckedModeBanner: false,
          color: tokens.colors.button,
          localizationsDelegates: AppLocalizations.localizationsDelegates,
          supportedLocales: AppLocalizations.supportedLocales,
          routerConfig: routerConfig,
          builder: (context, child) => KansoTestScope(
            theme: tokens,
            child: switch (builder) {
              final wrap? => wrap(context, child ?? const SizedBox.shrink()),
              null => child ?? const SizedBox.shrink(),
            },
          ),
        ),
      ),
    );
  }
}

/// Hands [theme] to what is under it the way the app does.
class KansoTestScope extends StatelessWidget {
  const KansoTestScope({required this.theme, required this.child, super.key});

  final KansoThemeData theme;
  final Widget child;

  @override
  Widget build(BuildContext context) {
    return KansoThemeTransition(
      theme: theme,
      child: DefaultTextStyle(
        style: theme.typography.body,
        child: DefaultSelectionStyle(
          selectionColor: theme.colors.accent.withValues(alpha: 0.3),
          cursorColor: theme.colors.accent,
          child: child,
        ),
      ),
    );
  }
}

/// Sets the debug default target platform for the life of the widget.
class _KansoPlatformOverride extends StatefulWidget {
  const _KansoPlatformOverride({required this.platform, required this.child});

  final TargetPlatform? platform;
  final Widget child;

  @override
  State<_KansoPlatformOverride> createState() => _KansoPlatformOverrideState();
}

class _KansoPlatformOverrideState extends State<_KansoPlatformOverride> {
  /// Whether this widget set the global override. A null platform leaves
  /// one the test itself set alone, which is how the route tests cover an
  /// iPhone while the rest of the harness is platform-neutral.
  bool ownsOverride = false;

  @override
  void initState() {
    super.initState();
    final platform = widget.platform;
    if (platform != null) {
      ownsOverride = true;
      debugDefaultTargetPlatformOverride = platform;
    }
  }

  @override
  void didUpdateWidget(_KansoPlatformOverride oldWidget) {
    super.didUpdateWidget(oldWidget);
    final platform = widget.platform;
    if (platform != null) {
      ownsOverride = true;
      debugDefaultTargetPlatformOverride = platform;
    }
  }

  @override
  void dispose() {
    if (ownsOverride) {
      debugDefaultTargetPlatformOverride = null;
    }
    super.dispose();
  }

  @override
  Widget build(BuildContext context) => widget.child;
}
