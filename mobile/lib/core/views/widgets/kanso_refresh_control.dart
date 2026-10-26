// Kanso — kanso_refresh_control.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/cupertino.dart'
    show CupertinoSliverRefreshControl, RefreshCallback;
import 'package:flutter/widgets.dart';

import '../../configs/kanso_sizes.dart';
import 'kanso_refresh_indicator.dart';

/// The pull-to-refresh of a list: the first sliver of a `CustomScrollView`
/// that bounces. Pulling the list down past [trigger] points and letting go
/// calls [onRefresh]; the list holds its place until the future it returns
/// completes. Drawn by Kanso, the same on iOS and Android.
class KansoRefreshControl extends StatelessWidget {
  const KansoRefreshControl({required this.onRefresh, super.key});

  final RefreshCallback onRefresh;

  /// How far the list is pulled for letting go to refresh.
  static const double trigger = 72;

  @override
  Widget build(BuildContext context) {
    return CupertinoSliverRefreshControl(
      onRefresh: onRefresh,
      refreshTriggerPullDistance: trigger,
      refreshIndicatorExtent: KansoSizes.touchTarget,
      builder: (context, mode, pulled, triggerDistance, extent) {
        return KansoRefreshIndicator(
          mode: mode,
          pulled: pulled,
          trigger: triggerDistance,
        );
      },
    );
  }
}
