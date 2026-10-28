// Kanso — kanso_dialog.dart
// Created 2026-10-29.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../configs/kanso_motion.dart';
import '../../configs/kanso_radii.dart';
import '../../configs/kanso_shadows.dart';
import '../../configs/kanso_spacing.dart';
import '../../utils/context_extensions.dart';
import 'kanso_button.dart';
import 'kanso_scrim.dart';

/// Shows an app-owned dialog above the whole app and completes with what its
/// action pops with. A tap on the veil, or the system's back, closes it.
Future<T?> showKansoDialog<T>(
  BuildContext context, {
  required WidgetBuilder builder,
}) {
  return Navigator.of(
    context,
    rootNavigator: true,
  ).push<T>(KansoDialogRoute<T>(builder: builder));
}

/// A binary confirmation. True only when the person confirms.
Future<bool> showKansoConfirm(
  BuildContext context, {
  required String title,
  required String body,
  required String cancelLabel,
  required String confirmLabel,
  bool danger = false,
}) async {
  final answer = await showKansoDialog<bool>(
    context,
    builder: (dialogContext) => KansoDialog(
      title: title,
      body: body,
      actions: <Widget>[
        KansoButton(
          label: confirmLabel,
          variant: danger
              ? KansoButtonVariant.danger
              : KansoButtonVariant.primary,
          onPressed: () => Navigator.of(dialogContext).pop(true),
        ),
        const SizedBox(height: KansoSpacing.xs),
        KansoButton(
          label: cancelLabel,
          variant: KansoButtonVariant.secondary,
          onPressed: () => Navigator.of(dialogContext).pop(false),
        ),
      ],
    ),
  );
  return answer ?? false;
}

/// The route of [showKansoDialog]. It is not Material's dialog: the veil,
/// surface, type and buttons are Kanso's own.
class KansoDialogRoute<T> extends PopupRoute<T> {
  KansoDialogRoute({required this.builder});

  final WidgetBuilder builder;

  @override
  Duration get transitionDuration => KansoMotion.content;

  @override
  Duration get reverseTransitionDuration => KansoMotion.short;

  @override
  bool get barrierDismissible => true;

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
    return _KansoDialogSurface(child: builder(context));
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
            scale: Tween<double>(begin: 0.96, end: 1).animate(curved),
            child: child,
          ),
        ),
      ],
    );
  }
}

/// The content of a Kanso dialog. The route owns the surface around it.
class KansoDialog extends StatelessWidget {
  const KansoDialog({
    required this.title,
    required this.actions,
    this.body,
    this.child,
    super.key,
  });

  final String title;
  final String? body;
  final Widget? child;
  final List<Widget> actions;

  @override
  Widget build(BuildContext context) {
    final content = child;
    return Column(
      mainAxisSize: MainAxisSize.min,
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: <Widget>[
        Text(title, style: context.typography.barTitle),
        if (body case final text?) ...<Widget>[
          const SizedBox(height: KansoSpacing.xs),
          Text(text, style: context.typography.body),
        ],
        if (content != null) ...<Widget>[
          const SizedBox(height: KansoSpacing.lg),
          content,
        ],
        const SizedBox(height: KansoSpacing.lg),
        ...actions,
      ],
    );
  }
}

class _KansoDialogSurface extends StatefulWidget {
  const _KansoDialogSurface({required this.child});

  final Widget child;

  @override
  State<_KansoDialogSurface> createState() => _KansoDialogSurfaceState();
}

class _KansoDialogSurfaceState extends State<_KansoDialogSurface> {
  @override
  void dispose() {
    FocusManager.instance.primaryFocus?.unfocus();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final media = MediaQuery.of(context);
    return AnimatedPadding(
      duration: KansoMotion.resolve(context, KansoMotion.short),
      curve: KansoMotion.standard,
      padding: EdgeInsets.only(bottom: media.viewInsets.bottom),
      child: SafeArea(
        child: Center(
          child: Padding(
            padding: const EdgeInsets.symmetric(horizontal: KansoSpacing.xl),
            child: ConstrainedBox(
              constraints: const BoxConstraints(maxWidth: 360),
              child: DecoratedBox(
                decoration: BoxDecoration(
                  color: context.colors.app,
                  borderRadius: KansoRadii.largeBorder,
                  border: Border.all(color: context.colors.divider),
                  boxShadow: KansoShadows.floating(Theme.brightnessOf(context)),
                ),
                child: ClipRRect(
                  borderRadius: KansoRadii.largeBorder,
                  child: SingleChildScrollView(
                    padding: const EdgeInsets.fromLTRB(
                      KansoSpacing.lg,
                      KansoSpacing.lg,
                      KansoSpacing.lg,
                      KansoSpacing.md,
                    ),
                    child: widget.child,
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
