// Kanso — kanso_text_field.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../configs/kanso_radii.dart';
import '../../configs/kanso_sizes.dart';
import '../../configs/kanso_spacing.dart';
import '../../utils/context_extensions.dart';
import 'kanso_selection_controls.dart';
import 'kanso_text_context_menu.dart';

/// A labelled field a person types in, drawn by Kanso: the label above, a
/// fill with an accent outline while it has focus, and a [hint] under it
/// (or an [error], in the danger colour, which replaces it).
///
/// With [obscure] the text is dots, it is not corrected or suggested, and
/// the keyboard is told it is a password. The field keeps the text in the
/// [controller] given.
class KansoTextField extends StatefulWidget {
  const KansoTextField({
    required this.label,
    required this.controller,
    this.hint,
    this.error,
    this.obscure = false,
    this.autofocus = false,
    this.enabled = true,
    this.textInputAction = TextInputAction.done,
    this.onChanged,
    this.onSubmitted,
    super.key,
  });

  final String label;
  final TextEditingController controller;
  final String? hint;
  final String? error;
  final bool obscure;
  final bool autofocus;
  final bool enabled;
  final TextInputAction textInputAction;
  final ValueChanged<String>? onChanged;
  final ValueChanged<String>? onSubmitted;

  @override
  State<KansoTextField> createState() => _KansoTextFieldState();
}

class _KansoTextFieldState extends State<KansoTextField> {
  late final FocusNode focusNode;
  final TextSelectionControls selectionControls = KansoSelectionControls();

  @override
  void initState() {
    super.initState();
    focusNode = FocusNode();
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
    final error = widget.error;
    final note = error ?? widget.hint;
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: <Widget>[
        Padding(
          padding: const EdgeInsetsDirectional.only(
            start: KansoSpacing.xxs,
            bottom: KansoSpacing.xxs,
          ),
          child: Text(widget.label, style: typography.label),
        ),
        ListenableBuilder(
          listenable: focusNode,
          builder: (context, _) {
            final focused = focusNode.hasFocus;
            final border = error != null
                ? colors.danger
                : (focused ? colors.accent : colors.fill);
            return Opacity(
              opacity: widget.enabled ? 1 : 0.5,
              child: Container(
                height: KansoSizes.searchField,
                padding: const EdgeInsets.symmetric(
                  horizontal: KansoSpacing.sm,
                ),
                alignment: AlignmentDirectional.centerStart,
                decoration: BoxDecoration(
                  color: colors.fill,
                  borderRadius: KansoRadii.mediumBorder,
                  border: Border.all(
                    color: border,
                    width: focused || error != null ? 1.5 : 1,
                  ),
                ),
                child: EditableText(
                  controller: widget.controller,
                  focusNode: focusNode,
                  readOnly: !widget.enabled,
                  autofocus: widget.autofocus,
                  obscureText: widget.obscure,
                  style: typography.body,
                  cursorColor: colors.accent,
                  backgroundCursorColor: colors.textMuted,
                  selectionColor: colors.accent.withValues(alpha: 0.3),
                  cursorRadius: const Radius.circular(1),
                  keyboardType: widget.obscure
                      ? TextInputType.visiblePassword
                      : TextInputType.text,
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
            );
          },
        ),
        if (note != null)
          Padding(
            padding: const EdgeInsetsDirectional.only(
              start: KansoSpacing.xxs,
              top: KansoSpacing.xxs,
            ),
            child: Text(
              note,
              style: typography.meta.copyWith(
                color: error != null ? colors.danger : null,
              ),
            ),
          ),
      ],
    );
  }
}
