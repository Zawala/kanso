// Kanso — kanso_text_input.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../configs/kanso_radii.dart';
import '../../configs/kanso_sizes.dart';
import '../../configs/kanso_spacing.dart';
import '../../utils/context_extensions.dart';
import 'kanso_selection_controls.dart';
import 'kanso_text_context_menu.dart';

/// A one-line field a person types in, drawn by Kanso: a username, a link.
///
/// It looks as `KansoSearchInput` does, without the magnifier: a quiet fill,
/// an accent outline while it has focus, and Flutter's `EditableText` with
/// Kanso's own caret (2 points wide, rounded, in the accent colour) and
/// context menu, no selection handles and no magnifier, so iOS and Android
/// draw it alike.
///
/// The text is [controller]'s, which the caller owns and disposes. [hint]
/// shows while it is empty. [onSubmitted] is told when the keyboard's action
/// key is pressed, with the text.
class KansoTextInput extends StatefulWidget {
  const KansoTextInput({
    required this.controller,
    required this.hint,
    this.onSubmitted,
    this.onChanged,
    this.autofocus = false,
    this.enabled = true,
    this.keyboardType = TextInputType.text,
    this.textInputAction = TextInputAction.done,
    this.semanticLabel,
    super.key,
  });

  final TextEditingController controller;
  final String hint;
  final ValueChanged<String>? onSubmitted;
  final ValueChanged<String>? onChanged;
  final bool autofocus;

  /// Whether it takes input; dimmed and read only while it does not.
  final bool enabled;
  final TextInputType keyboardType;
  final TextInputAction textInputAction;

  /// What the field is called to a screen reader.
  final String? semanticLabel;

  /// How tall the caret is, in points.
  static const double caretHeight = 18;

  @override
  State<KansoTextInput> createState() => _KansoTextInputState();
}

class _KansoTextInputState extends State<KansoTextInput>
    implements TextSelectionGestureDetectorBuilderDelegate {
  late final FocusNode focusNode;
  late final TextSelectionGestureDetectorBuilder gestures;

  /// No handles and no menu of the platform's: see [KansoSelectionControls].
  final TextSelectionControls selectionControls = KansoSelectionControls();

  @override
  final GlobalKey<EditableTextState> editableTextKey =
      GlobalKey<EditableTextState>();

  @override
  bool get forcePressEnabled => false;

  @override
  bool get selectionEnabled => true;

  @override
  void initState() {
    super.initState();
    focusNode = FocusNode();
    gestures = TextSelectionGestureDetectorBuilder(delegate: this);
  }

  @override
  void dispose() {
    focusNode.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
    final typography = context.typography;
    // The framework sizes and places the caret differently on Apple's
    // platforms and on Android's; what each is asked for here makes the
    // same caret on both.
    final caretAsked = switch (Theme.of(context).platform) {
      TargetPlatform.iOS ||
      TargetPlatform.macOS => KansoTextInput.caretHeight - 2,
      _ => KansoTextInput.caretHeight,
    };
    return ListenableBuilder(
      listenable: Listenable.merge(<Listenable>[
        widget.controller,
        focusNode,
      ]),
      builder: (context, _) {
        final focused = focusNode.hasFocus;
        final empty = widget.controller.text.isEmpty;
        return Semantics(
          textField: true,
          label: widget.semanticLabel ?? widget.hint,
          child: Opacity(
            opacity: widget.enabled ? 1 : 0.6,
            child: Container(
              height: KansoSizes.searchField,
              padding: const EdgeInsets.symmetric(horizontal: KansoSpacing.sm),
              decoration: BoxDecoration(
                color: colors.fill,
                borderRadius: KansoRadii.mediumBorder,
                border: Border.all(
                  color: focused ? colors.accent : colors.fill,
                  width: focused ? 1.5 : 1,
                ),
              ),
              child: Stack(
                alignment: AlignmentDirectional.centerStart,
                children: <Widget>[
                  if (empty)
                    IgnorePointer(
                      child: ExcludeSemantics(
                        child: Text(
                          widget.hint,
                          maxLines: 1,
                          overflow: TextOverflow.ellipsis,
                          style: typography.body.copyWith(
                            color: colors.textMuted,
                          ),
                        ),
                      ),
                    ),
                  // A bare `EditableText` takes no touches: the gestures
                  // that place the caret and select words are added.
                  gestures.buildGestureDetector(
                    behavior: HitTestBehavior.translucent,
                    child: EditableText(
                      key: editableTextKey,
                      rendererIgnoresPointer: true,
                      controller: widget.controller,
                      focusNode: focusNode,
                      readOnly: !widget.enabled,
                      autofocus: widget.autofocus,
                      style: typography.body,
                      cursorColor: colors.accent,
                      backgroundCursorColor: colors.textMuted,
                      selectionColor: colors.accent.withValues(alpha: 0.3),
                      cursorRadius: const Radius.circular(1),
                      cursorHeight: caretAsked,
                      keyboardType: widget.keyboardType,
                      textInputAction: widget.textInputAction,
                      autocorrect: false,
                      enableSuggestions: false,
                      selectionControls: selectionControls,
                      contextMenuBuilder: (context, state) =>
                          KansoTextContextMenu(state: state),
                      onChanged: widget.onChanged,
                      onSubmitted: widget.onSubmitted,
                    ),
                  ),
                ],
              ),
            ),
          ),
        );
      },
    );
  }
}
