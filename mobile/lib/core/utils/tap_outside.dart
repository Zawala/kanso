// Kanso — tap_outside.dart
// Created 2026-10-29.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/gestures.dart';
import 'package:flutter/widgets.dart';

/// Clears [focusNode] when the pointer that went down outside its field is
/// released as a tap, not when that pointer turns into a scroll or drag.
///
/// `EditableText.onTapOutside` reports the pointer down, before the gesture
/// arena has decided what the person is doing. Dismissing there makes a
/// flick through a note drop the keyboard on the first frame. The pointer is
/// followed to its release instead, and only a release that stayed within
/// the touch slop clears the field.
void unfocusOnTapOutside(PointerDownEvent event, FocusNode focusNode) {
  final router = GestureBinding.instance.pointerRouter;
  var travelled = 0.0;
  var routed = true;
  late final void Function(PointerEvent) route;

  void removeRoute() {
    if (!routed) return;
    routed = false;
    router.removeRoute(event.pointer, route);
  }

  route = (pointerEvent) {
    if (!routed || pointerEvent.pointer != event.pointer) return;
    if (pointerEvent is PointerMoveEvent) {
      travelled += pointerEvent.delta.distance;
    } else if (pointerEvent is PointerUpEvent) {
      removeRoute();
      if (travelled <= kTouchSlop && focusNode.hasFocus) {
        focusNode.unfocus();
      }
    } else if (pointerEvent is PointerCancelEvent) {
      removeRoute();
    }
  };

  router.addRoute(event.pointer, route);
}
