// Kanso — kanso_refresh_indicator.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:math' as math;

import 'package:flutter/cupertino.dart' show RefreshIndicatorMode;
import 'package:flutter/widgets.dart';

import '../../configs/kanso_icons.dart';
import '../../utils/context_extensions.dart';
import 'kanso_icon.dart';

/// What shows above a list that is being pulled down to refresh: the
/// refresh icon, turning as far as the pull has gone, and spinning while
/// the refresh runs.
class KansoRefreshIndicator extends StatefulWidget {
  const KansoRefreshIndicator({
    required this.mode,
    required this.pulled,
    required this.trigger,
    super.key,
  });

  final RefreshIndicatorMode mode;

  /// How far the list has been pulled down, in points.
  final double pulled;

  /// How far it must be pulled for letting go to refresh.
  final double trigger;

  @override
  State<KansoRefreshIndicator> createState() => _KansoRefreshIndicatorState();
}

class _KansoRefreshIndicatorState extends State<KansoRefreshIndicator>
    with SingleTickerProviderStateMixin {
  late final AnimationController spin;

  @override
  void initState() {
    super.initState();
    spin = AnimationController(
      vsync: this,
      duration: const Duration(milliseconds: 900),
    );
    syncSpin();
  }

  @override
  void didUpdateWidget(KansoRefreshIndicator oldWidget) {
    super.didUpdateWidget(oldWidget);
    syncSpin();
  }

  @override
  void dispose() {
    spin.dispose();
    super.dispose();
  }

  void syncSpin() {
    if (widget.mode == RefreshIndicatorMode.refresh) {
      if (!spin.isAnimating) {
        spin.repeat();
      }
    } else if (spin.isAnimating) {
      spin.stop();
    }
  }

  @override
  Widget build(BuildContext context) {
    final fraction = (widget.pulled / widget.trigger).clamp(0.0, 1.0);
    final refreshing = widget.mode == RefreshIndicatorMode.refresh;
    final reduced = MediaQuery.disableAnimationsOf(context);
    final icon = KansoIcon(
      KansoIcons.refreshCw,
      color: context.colors.textMuted,
    );
    return Center(
      child: Opacity(
        opacity: refreshing ? 1 : fraction,
        child: refreshing && !reduced
            ? RotationTransition(turns: spin, child: icon)
            : Transform.rotate(angle: fraction * math.pi, child: icon),
      ),
    );
  }
}
