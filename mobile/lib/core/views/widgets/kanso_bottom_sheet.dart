// Kanso — kanso_bottom_sheet.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../configs/kanso_motion.dart';
import '../../configs/kanso_radii.dart';
import '../../configs/kanso_shadows.dart';
import '../../configs/kanso_sizes.dart';
import '../../configs/kanso_spacing.dart';
import '../../utils/context_extensions.dart';

/// The panel of a sheet that rises from the foot of the screen: a rounded
/// surface that is as tall as its [child] and at most nine tenths of the
/// screen, no wider than a tablet's sheet and centred when the screen is
/// wider. It takes the home indicator's height as well, moves above the
/// keyboard, and goes down when its handle is dragged down.
///
/// The veil behind it and the way it moves in are the route's
/// (`showKansoBottomSheet`).
class KansoBottomSheet extends StatefulWidget {
  const KansoBottomSheet({required this.child, super.key});

  final Widget child;

  /// How much of the available screen height the panel may take.
  static const double heightFraction = 0.92;

  /// How far the handle is dragged before letting go closes the sheet.
  static const double dismissDistance = 120;

  /// How fast a flick closes the sheet, in points per second.
  static const double dismissVelocity = 700;

  /// The grab handle, for tests and for a steady place to drag from.
  static const Key handleKey = ValueKey<String>('kanso-sheet-handle');

  @override
  State<KansoBottomSheet> createState() => _KansoBottomSheetState();
}

class _KansoBottomSheetState extends State<KansoBottomSheet> {
  final ValueNotifier<double> _drag = ValueNotifier<double>(0);

  @override
  void dispose() {
    // A field in a sheet must not keep the keyboard alive after the route
    // goes. This also covers a system back or a veil tap.
    FocusManager.instance.primaryFocus?.unfocus();
    _drag.dispose();
    super.dispose();
  }

  void _dismissKeyboard() {
    FocusScope.of(context).unfocus();
  }

  void _dragStarted(DragStartDetails details) {
    _dismissKeyboard();
  }

  void _dragged(DragUpdateDetails details) {
    final delta = details.primaryDelta ?? 0;
    if (delta <= 0) return;
    _drag.value += delta;
  }

  void _dragEnded(DragEndDetails details) {
    final velocity = details.primaryVelocity ?? 0;
    if (_drag.value > KansoBottomSheet.dismissDistance ||
        velocity > KansoBottomSheet.dismissVelocity) {
      Navigator.of(context).maybePop();
      return;
    }
    _drag.value = 0;
  }

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
    return LayoutBuilder(
      builder: (context, constraints) {
        final media = MediaQuery.of(context);
        final keyboard = media.viewInsets.bottom;
        final available = (constraints.maxHeight - keyboard).clamp(
          0.0,
          constraints.maxHeight,
        );
        final maxHeight = available * KansoBottomSheet.heightFraction;
        return ValueListenableBuilder<double>(
          valueListenable: _drag,
          builder: (context, drag, _) {
            final settling = drag == 0;
            final duration = KansoMotion.resolve(
              context,
              settling ? KansoMotion.short : const Duration(milliseconds: 1),
            );
            return Align(
              alignment: Alignment.bottomCenter,
              child: AnimatedPadding(
                duration: KansoMotion.resolve(context, KansoMotion.short),
                curve: KansoMotion.standard,
                padding: EdgeInsets.only(bottom: keyboard),
                child: AnimatedContainer(
                  duration: duration,
                  curve: KansoMotion.standard,
                  transform: Matrix4.translationValues(0, drag, 0),
                  child: ConstrainedBox(
                    constraints: BoxConstraints(
                      maxWidth: KansoSizes.sheetWidth,
                      maxHeight: maxHeight,
                    ),
                    child: GestureDetector(
                      behavior: HitTestBehavior.opaque,
                      onTap: _dismissKeyboard,
                      child: DecoratedBox(
                        decoration: BoxDecoration(
                          color: colors.app,
                          borderRadius: const BorderRadius.vertical(
                            top: Radius.circular(KansoRadii.large),
                          ),
                          border: Border.all(color: colors.divider),
                          boxShadow: KansoShadows.sheet(
                            Theme.brightnessOf(context),
                          ),
                        ),
                        child: ClipRRect(
                          borderRadius: const BorderRadius.vertical(
                            top: Radius.circular(KansoRadii.large),
                          ),
                          child: SafeArea(
                            top: false,
                            child: Column(
                              mainAxisSize: MainAxisSize.min,
                              children: <Widget>[
                                GestureDetector(
                                  key: KansoBottomSheet.handleKey,
                                  behavior: HitTestBehavior.opaque,
                                  onVerticalDragStart: _dragStarted,
                                  onVerticalDragUpdate: _dragged,
                                  onVerticalDragEnd: _dragEnded,
                                  child: Semantics(
                                    button: true,
                                    child: Padding(
                                      padding: const EdgeInsets.symmetric(
                                        vertical: KansoSpacing.xs,
                                      ),
                                      child: Container(
                                        width: 40,
                                        height: 5,
                                        decoration: BoxDecoration(
                                          color: colors.textMuted.withValues(
                                            alpha: 0.55,
                                          ),
                                          borderRadius: KansoRadii.pillBorder,
                                        ),
                                      ),
                                    ),
                                  ),
                                ),
                                Flexible(child: widget.child),
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
          },
        );
      },
    );
  }
}
