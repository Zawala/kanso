// Kanso — kanso_search_input.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../configs/kanso_icons.dart';
import '../../configs/kanso_radii.dart';
import '../../configs/kanso_sizes.dart';
import '../../configs/kanso_spacing.dart';
import '../../utils/context_extensions.dart';
import 'kanso_icon.dart';
import 'kanso_pressable.dart';
import 'kanso_search_field.dart' show KansoSearchField;
import 'kanso_selection_controls.dart';
import 'kanso_text_context_menu.dart';

/// A search field a person types in: [KansoSearchField] made to work.
///
/// It is drawn as that is, with the accent outline while it has focus and
/// a clear button while it has text, over Flutter's `EditableText` with
/// Kanso's own caret (2 points wide, rounded, in the accent colour, not
/// blinking) and context menu, no selection handles and no magnifier, so
/// iOS and Android draw it alike. [onChanged] is told of every change;
/// [onCleared] of the clear button, after the field is emptied.
///
/// The field keeps its own text; [initialText] is its first. Give the field
/// a new key to start it over.
class KansoSearchInput extends StatefulWidget {
  const KansoSearchInput({
    required this.hint,
    required this.onChanged,
    this.initialText = '',
    this.autofocus = false,
    this.onCleared,
    this.clearLabel,
    super.key,
  });

  final String hint;
  final String initialText;
  final bool autofocus;
  final ValueChanged<String> onChanged;
  final VoidCallback? onCleared;

  /// What the clear button says to a screen reader.
  final String? clearLabel;

  /// How tall the caret is, in points.
  static const double caretHeight = 18;

  @override
  State<KansoSearchInput> createState() => _KansoSearchInputState();
}

class _KansoSearchInputState extends State<KansoSearchInput>
    implements TextSelectionGestureDetectorBuilderDelegate {
  late final TextEditingController controller;
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
    controller = TextEditingController(text: widget.initialText);
    focusNode = FocusNode();
    gestures = TextSelectionGestureDetectorBuilder(delegate: this);
  }

  @override
  void dispose() {
    focusNode.dispose();
    controller.dispose();
    super.dispose();
  }

  void clear() {
    controller.clear();
    focusNode.requestFocus();
    (widget.onCleared ?? () => widget.onChanged(''))();
  }

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
    final typography = context.typography;
    // The framework sizes and places the caret differently on Apple's
    // platforms and on Android's. What each is asked for here makes the
    // same caret on both (the pictures of the two are compared by the
    // goldens, pixel for pixel).
    final (caretAsked, caretShift) = switch (Theme.of(context).platform) {
      TargetPlatform.iOS ||
      TargetPlatform.macOS => (KansoSearchInput.caretHeight - 2, 0.0),
      _ => (KansoSearchInput.caretHeight, 0.0),
    };
    return ListenableBuilder(
      listenable: Listenable.merge(<Listenable>[controller, focusNode]),
      builder: (context, _) {
        final focused = focusNode.hasFocus;
        final empty = controller.text.isEmpty;
        return Container(
          height: KansoSizes.searchField,
          padding: const EdgeInsetsDirectional.only(start: KansoSpacing.sm),
          decoration: BoxDecoration(
            color: colors.fill,
            borderRadius: KansoRadii.mediumBorder,
            border: Border.all(
              color: focused ? colors.accent : colors.fill,
              width: focused ? 1.5 : 1,
            ),
          ),
          child: Row(
            children: <Widget>[
              KansoIcon(
                KansoIcons.search,
                size: KansoSizes.iconSmall + 2,
                color: colors.textMuted,
              ),
              const SizedBox(width: KansoSpacing.xs),
              Expanded(
                child: Stack(
                  alignment: AlignmentDirectional.centerStart,
                  children: <Widget>[
                    if (empty)
                      IgnorePointer(
                        child: Text(
                          widget.hint,
                          maxLines: 1,
                          overflow: TextOverflow.ellipsis,
                          style: typography.body.copyWith(
                            color: colors.textMuted,
                          ),
                        ),
                      ),
                    // A bare `EditableText` takes no touches: the gestures
                    // that place the caret and select words are added.
                    gestures.buildGestureDetector(
                      behavior: HitTestBehavior.translucent,
                      child: EditableText(
                        key: editableTextKey,
                        // The gestures above place the caret and select.
                        rendererIgnoresPointer: true,
                        controller: controller,
                        focusNode: focusNode,
                        autofocus: widget.autofocus,
                        style: typography.body,
                        cursorColor: colors.accent,
                        backgroundCursorColor: colors.textMuted,
                        selectionColor: colors.accent.withValues(alpha: 0.3),
                        cursorRadius: const Radius.circular(1),
                        cursorHeight: caretAsked,
                        cursorOffset: Offset(0, caretShift),
                        keyboardType: TextInputType.text,
                        textInputAction: TextInputAction.search,
                        autocorrect: false,
                        enableSuggestions: false,
                        selectionControls: selectionControls,
                        contextMenuBuilder: (context, state) =>
                            KansoTextContextMenu(state: state),
                        onChanged: widget.onChanged,
                      ),
                    ),
                  ],
                ),
              ),
              if (!empty)
                KansoPressable(
                  onTap: clear,
                  semanticLabel: widget.clearLabel,
                  child: SizedBox.square(
                    dimension: KansoSizes.searchField,
                    child: Center(
                      child: KansoIcon(
                        KansoIcons.close,
                        size: KansoSizes.iconSmall,
                        color: colors.textMuted,
                      ),
                    ),
                  ),
                ),
            ],
          ),
        );
      },
    );
  }
}
