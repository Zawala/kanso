// Kanso — kanso_selection_controls.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/widgets.dart';

/// The selection controls of Kanso's text fields: no handles to drag, and
/// no toolbar of their own, so the field's `contextMenuBuilder` (the
/// `KansoTextContextMenu`) draws the only menu there is.
///
/// A person selects by a long press or a double tap, and changes what is
/// selected by selecting again; there is nothing the platform draws, so iOS
/// and Android look alike.
class KansoSelectionControls extends TextSelectionControls
    with TextSelectionHandleControls {
  @override
  Size getHandleSize(double textLineHeight) => Size.zero;

  @override
  Offset getHandleAnchor(TextSelectionHandleType type, double textLineHeight) =>
      Offset.zero;

  @override
  Widget buildHandle(
    BuildContext context,
    TextSelectionHandleType type,
    double textLineHeight, [
    VoidCallback? onTap,
  ]) => const SizedBox.shrink();
}
