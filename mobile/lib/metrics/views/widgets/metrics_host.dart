// Kanso — metrics_host.dart
// Created 2026-10-31.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../ai_settings/state/ai_status/ai_status_bloc.dart';
import '../../../core/configs/kanso_layout.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/state/appearance/appearance_bloc.dart';
import '../../../core/views/widgets/kanso_plain_text.dart';
import '../../../core/views/widgets/kanso_scrim.dart';
import '../../models/repos/abs/usage_metrics_repository.dart';
import '../../utils/metrics_device.dart';
import 'metrics_notice_card.dart';

/// [child] with the anonymous count's one-time notice over it. After the
/// notice was acknowledged, or when it was already seen, sends what is due
/// while the app is open: once after a short settle, then hourly, and when
/// the app goes to the background, after noting how long it was in front.
///
/// This widget is only mounted for production release builds. The Rust side
/// independently refuses to count or send in tests and debug builds, so a
/// stray notice can never turn into a request there.
class MetricsHost extends StatefulWidget {
  const MetricsHost({
    required this.metrics,
    required this.child,
    this.firstSendDelay = const Duration(seconds: 20),
    this.sendInterval = const Duration(hours: 1),
    this.clock = DateTime.now,
    super.key,
  });

  final UsageMetricsRepository metrics;
  final Widget child;

  /// The wait after launch when the notice was already seen.
  final Duration firstSendDelay;

  /// How often to send what is due while open.
  final Duration sendInterval;

  /// The time, for how long the app was in front. Tests pass their own.
  final DateTime Function() clock;

  @override
  State<MetricsHost> createState() => _MetricsHostState();
}

class _MetricsHostState extends State<MetricsHost> with WidgetsBindingObserver {
  bool _noticeVisible = false;
  bool _started = false;
  bool _sending = false;

  /// Counting is on: the notice was seen here or just acknowledged.
  bool _counting = false;

  /// When the app last came to the front; null while it is in the
  /// background.
  DateTime? _inFrontSince;
  Timer? _timer;

  @override
  void initState() {
    super.initState();
    _inFrontSince = widget.clock();
    WidgetsBinding.instance.addObserver(this);
    WidgetsBinding.instance.addPostFrameCallback((_) => _start());
  }

  @override
  void didChangeAppLifecycleState(AppLifecycleState state) {
    switch (state) {
      case AppLifecycleState.resumed:
        _inFrontSince ??= widget.clock();
      case AppLifecycleState.hidden:
      case AppLifecycleState.paused:
      case AppLifecycleState.detached:
        unawaited(_endSession());
      case AppLifecycleState.inactive:
        break;
    }
  }

  /// The app left the front: notes how long it was there, then sends.
  Future<void> _endSession() async {
    final since = _inFrontSince;
    _inFrontSince = null;
    if (since == null || !_counting) {
      return;
    }
    final seconds = widget.clock().difference(since).inSeconds;
    try {
      await widget.metrics.endSession(seconds < 0 ? 0 : seconds);
    } on Object {
      // Counting never disturbs the app.
    }
    await _sendDue();
  }

  Future<void> _start() async {
    if (_started) {
      return;
    }
    _started = true;
    final bool seen;
    try {
      seen = await widget.metrics.noticeSeen();
    } on Object {
      return;
    }
    if (!mounted) {
      return;
    }
    if (seen) {
      _begin();
      _scheduleFirstSend();
    } else {
      setState(() => _noticeVisible = true);
    }
  }

  /// Counting starts: this launch is noted for the next send.
  void _begin() {
    _counting = true;
    unawaited(widget.metrics.launched());
  }

  void _scheduleFirstSend() {
    _timer?.cancel();
    _timer = Timer(widget.firstSendDelay, () {
      unawaited(_sendDue());
      _timer = Timer.periodic(
        widget.sendInterval,
        (_) => unawaited(_sendDue()),
      );
    });
  }

  Future<void> _acknowledge() async {
    final bool saved;
    try {
      saved = await widget.metrics.markNoticeSeen();
    } on Object {
      return;
    }
    if (!mounted) {
      return;
    }
    setState(() => _noticeVisible = false);
    if (!saved) {
      // It was not acknowledged, so nothing may be counted or sent. The
      // notice will be offered again at the next launch.
      return;
    }
    _begin();
    unawaited(_sendDue());
    _timer?.cancel();
    _timer = Timer.periodic(
      widget.sendInterval,
      (_) => unawaited(_sendDue()),
    );
  }

  Future<void> _sendDue() async {
    if (!mounted || _sending) {
      return;
    }
    _sending = true;
    try {
      final ai = context.read<AiStatusBloc>().state.enabled;
      final tablet =
          MediaQuery.sizeOf(context).shortestSide >=
          KansoLayout.tabletShortestSide;
      final locale = Localizations.localeOf(context).languageCode;
      final look = _look();
      await widget.metrics.sendDue(
        tablet: tablet,
        osVersion: metricsOsVersion(),
        ai: ai,
        locale: locale,
        themeMode: look.mode,
        theme: look.theme,
      );
    } on Object {
      // Metrics never disturb the app; the ledger keeps the batch for the
      // next open.
    } finally {
      _sending = false;
    }
  }

  /// The theme mode and the ID of the theme in use, or empty where the app
  /// has no appearance (the core leaves those out).
  ({String mode, String theme}) _look() {
    final AppearanceState state;
    try {
      state = context.read<AppearanceBloc>().state;
    } on Object {
      return (mode: '', theme: '');
    }
    final selection = state.selection;
    final dark = MediaQuery.platformBrightnessOf(context) == Brightness.dark;
    final theme = selection.followSystem
        ? (dark ? selection.dark : selection.light)
        : selection.ui;
    return (mode: state.themeMode.name, theme: theme);
  }

  @override
  void dispose() {
    WidgetsBinding.instance.removeObserver(this);
    _timer?.cancel();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return Stack(
      children: <Widget>[
        Positioned.fill(child: widget.child),
        if (_noticeVisible) ...<Widget>[
          const Positioned.fill(child: AbsorbPointer(child: KansoScrim())),
          Positioned.fill(
            child: KansoPlainText(
              child: Center(
                child: SingleChildScrollView(
                  child: Padding(
                    padding: const EdgeInsets.all(KansoSpacing.lg),
                    child: MetricsNoticeCard(
                      onOk: () => unawaited(_acknowledge()),
                    ),
                  ),
                ),
              ),
            ),
          ),
        ],
      ],
    );
  }
}
