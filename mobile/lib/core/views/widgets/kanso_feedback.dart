// Kanso — kanso_feedback.dart.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:flutter/material.dart';

import '../../configs/kanso_icons.dart';
import '../../configs/kanso_radii.dart';
import '../../configs/kanso_spacing.dart';
import '../../utils/context_extensions.dart';
import 'kanso_icon.dart';
import 'kanso_pressable.dart';

enum KansoFeedbackTone { success, info, error }

@immutable
class KansoFeedbackMessage {
  const KansoFeedbackMessage({
    required this.text,
    required this.tone,
    this.actionLabel,
    this.onAction,
  });

  final String text;
  final KansoFeedbackTone tone;
  final String? actionLabel;
  final VoidCallback? onAction;
}

/// Holds at most one transient feedback message and coalesces duplicates.
class KansoFeedbackController extends ChangeNotifier {
  KansoFeedbackMessage? _current;
  Timer? _timer;

  KansoFeedbackMessage? get current => _current;

  void show(KansoFeedbackMessage message) {
    if (_current?.text == message.text && _current?.tone == message.tone) {
      _restartTimer(message.tone);
      return;
    }
    _current = message;
    notifyListeners();
    _restartTimer(message.tone);
  }

  void hide() {
    _timer?.cancel();
    _timer = null;
    if (_current == null) return;
    _current = null;
    notifyListeners();
  }

  void _restartTimer(KansoFeedbackTone tone) {
    _timer?.cancel();
    _timer = Timer(switch (tone) {
      KansoFeedbackTone.success => const Duration(milliseconds: 2400),
      KansoFeedbackTone.info => const Duration(milliseconds: 3200),
      KansoFeedbackTone.error => const Duration(milliseconds: 5000),
    }, hide);
  }

  @override
  void dispose() {
    _timer?.cancel();
    super.dispose();
  }
}

class KansoFeedbackScope extends InheritedWidget {
  const KansoFeedbackScope({
    required this.controller,
    required super.child,
    super.key,
  });

  final KansoFeedbackController controller;

  @override
  bool updateShouldNotify(KansoFeedbackScope oldWidget) =>
      oldWidget.controller != controller;
}

abstract final class KansoFeedback {
  static KansoFeedbackController? maybeOf(BuildContext context) => context
      .dependOnInheritedWidgetOfExactType<KansoFeedbackScope>()
      ?.controller;

  static void show(
    BuildContext context,
    String message, {
    KansoFeedbackTone tone = KansoFeedbackTone.success,
    String? actionLabel,
    VoidCallback? onAction,
  }) => maybeOf(context)?.show(
    KansoFeedbackMessage(
      text: message,
      tone: tone,
      actionLabel: actionLabel,
      onAction: onAction,
    ),
  );

  static void hide(BuildContext context) => maybeOf(context)?.hide();
}

/// App-level overlay that renders a single safe-area aware feedback pill.
class KansoFeedbackHost extends StatefulWidget {
  const KansoFeedbackHost({required this.child, super.key});

  final Widget child;

  @override
  State<KansoFeedbackHost> createState() => _KansoFeedbackHostState();
}

class _KansoFeedbackHostState extends State<KansoFeedbackHost> {
  final _controller = KansoFeedbackController();

  @override
  void dispose() {
    _controller.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) => KansoFeedbackScope(
    controller: _controller,
    child: Stack(
      fit: StackFit.expand,
      children: <Widget>[
        widget.child,
        AnimatedBuilder(
          animation: _controller,
          builder: (context, _) {
            final message = _controller.current;
            return Positioned(
              left: 20,
              right: 20,
              top: 0,
              child: IgnorePointer(
                ignoring: message == null,
                child: AnimatedSwitcher(
                  duration: const Duration(milliseconds: 180),
                  reverseDuration: const Duration(milliseconds: 120),
                  transitionBuilder: (child, animation) => FadeTransition(
                    opacity: animation,
                    child: SlideTransition(
                      position: Tween<Offset>(
                        begin: const Offset(0, -.25),
                        end: Offset.zero,
                      ).animate(animation),
                      child: child,
                    ),
                  ),
                  child: message == null
                      ? const SizedBox.shrink(key: ValueKey<String>('empty'))
                      : _KansoFeedbackPill(
                          key: ValueKey<KansoFeedbackMessage>(message),
                          message: message,
                        ),
                ),
              ),
            );
          },
        ),
      ],
    ),
  );
}

class _KansoFeedbackPill extends StatelessWidget {
  const _KansoFeedbackPill({required this.message, super.key});

  final KansoFeedbackMessage message;

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
    final icon = switch (message.tone) {
      KansoFeedbackTone.success => KansoIcons.circleCheck,
      KansoFeedbackTone.info => KansoIcons.info,
      KansoFeedbackTone.error => KansoIcons.circleX,
    };
    return SafeArea(
      bottom: false,
      child: Padding(
        padding: const EdgeInsets.only(top: 8),
        child: Center(
          child: ConstrainedBox(
            constraints: const BoxConstraints(maxWidth: 560),
            child: Semantics(
              liveRegion: true,
              container: true,
              label: message.text,
              child: GestureDetector(
                onTap: () => KansoFeedback.hide(context),
                child: DecoratedBox(
                  decoration: BoxDecoration(
                    color: colors.text,
                    borderRadius: KansoRadii.pillBorder,
                  ),
                  child: Padding(
                    padding: EdgeInsets.fromLTRB(
                      16,
                      message.actionLabel == null ? 11 : 6,
                      message.actionLabel == null ? 16 : 6,
                      message.actionLabel == null ? 11 : 6,
                    ),
                    child: Row(
                      mainAxisSize: MainAxisSize.min,
                      children: <Widget>[
                        if (message.actionLabel == null) ...<Widget>[
                          KansoIcon(icon, size: 18, color: colors.editor),
                          const SizedBox(width: 8),
                        ],
                        Flexible(
                          child: Text(
                            message.text,
                            style: context.typography.body.copyWith(
                              color: colors.editor,
                            ),
                            softWrap: true,
                          ),
                        ),
                        if (message.actionLabel case final label?) ...<Widget>[
                          const SizedBox(width: 12),
                          KansoPressable(
                            onTap: () {
                              KansoFeedback.hide(context);
                              message.onAction?.call();
                            },
                            child: Padding(
                              padding: const EdgeInsets.symmetric(
                                horizontal: KansoSpacing.sm,
                                vertical: KansoSpacing.xxs,
                              ),
                              child: Text(
                                label,
                                style: context.typography.label.copyWith(
                                  color: colors.accent,
                                ),
                              ),
                            ),
                          ),
                        ],
                      ],
                    ),
                  ),
                ),
              ),
            ),
          ),
        ),
      ),
    );
  }
}
