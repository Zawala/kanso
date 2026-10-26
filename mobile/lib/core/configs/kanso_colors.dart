// Kanso — kanso_colors.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

/// The colours of the Kanso interface: the `ui` tokens of a Kanso theme file.
///
/// [dark] and [light] are copied from `themes/kanso-dark.json` and
/// `themes/kanso-light.json`, the files the desktop app's `theme.rs` agrees
/// with; a test reads the files so the three cannot drift.
///
/// This is a [ThemeExtension] rather than a Freezed model because the
/// framework requires `copyWith` and `lerp` overrides.
@immutable
class KansoColors extends ThemeExtension<KansoColors> {
  const KansoColors({
    required this.app,
    required this.sidebar,
    required this.list,
    required this.editor,
    required this.elevated,
    required this.fill,
    required this.text,
    required this.textSecondary,
    required this.textMuted,
    required this.divider,
    required this.accent,
    required this.link,
    required this.rowActive,
    required this.rowHover,
    required this.sidebarActive,
    required this.sidebarHover,
    required this.hover,
    required this.selected,
    required this.activeLine,
    required this.button,
    required this.buttonText,
    required this.success,
    required this.warning,
    required this.danger,
  });

  /// Kanso Dark, from `themes/kanso-dark.json`.
  static const KansoColors dark = KansoColors(
    app: Color(0xFF181816),
    sidebar: Color(0xFF1D1D1A),
    list: Color(0xFF232320),
    editor: Color(0xFF282824),
    elevated: Color(0xFF30302B),
    fill: Color(0xFF30302B),
    text: Color(0xFFF1EFE7),
    textSecondary: Color(0xFFB7B3A8),
    textMuted: Color(0xFF7E7A70),
    divider: Color(0xFF3A3933),
    accent: Color(0xFF7E96B3),
    link: Color(0xFF869DB9),
    rowActive: Color(0xFF30302B),
    rowHover: Color(0xFF2A2A26),
    sidebarActive: Color(0xFF2E2E2A),
    sidebarHover: Color(0xFF252522),
    hover: Color(0xFF3A3933),
    selected: Color(0xFF3A3933),
    activeLine: Color(0xFF2D2D29),
    button: Color(0xFF4A6385),
    buttonText: Color(0xFFF1EFE7),
    success: Color(0xFF7BAA78),
    warning: Color(0xFFC49A5A),
    danger: Color(0xFFC96E63),
  );

  /// Kanso Light, from `themes/kanso-light.json`.
  static const KansoColors light = KansoColors(
    app: Color(0xFFF4F1EA),
    sidebar: Color(0xFFEBE7DD),
    list: Color(0xFFF8F5EF),
    editor: Color(0xFFFCFAF4),
    elevated: Color(0xFFFFFDF8),
    fill: Color(0xFFE6E0D4),
    text: Color(0xFF26241F),
    textSecondary: Color(0xFF655F55),
    textMuted: Color(0xFF857C6E),
    divider: Color(0xFFDDD6C9),
    accent: Color(0xFF5E7895),
    link: Color(0xFF4D6684),
    rowActive: Color(0xFFECE7DC),
    rowHover: Color(0xFFF2EEE6),
    sidebarActive: Color(0xFFE2DDD0),
    sidebarHover: Color(0xFFE5E0D5),
    hover: Color(0xFFF0ECE3),
    selected: Color(0xFFE6E0D4),
    activeLine: Color(0xFFF6F3EB),
    button: Color(0xFF4D6684),
    buttonText: Color(0xFFFFFDF8),
    success: Color(0xFF4F7F4C),
    warning: Color(0xFF8A6424),
    danger: Color(0xFFA6493F),
  );

  /// The `app` token.
  final Color app;

  /// The `sidebar` token.
  final Color sidebar;

  /// The `list` token.
  final Color list;

  /// The `editor` token.
  final Color editor;

  /// The `elevated` token.
  final Color elevated;

  /// The `fill` token.
  final Color fill;

  /// The `text` token.
  final Color text;

  /// The `text_secondary` token.
  final Color textSecondary;

  /// The `text_muted` token.
  final Color textMuted;

  /// The `divider` token.
  final Color divider;

  /// The `accent` token.
  final Color accent;

  /// The `link` token.
  final Color link;

  /// The `row_active` token.
  final Color rowActive;

  /// The `row_hover` token.
  final Color rowHover;

  /// The `sidebar_active` token.
  final Color sidebarActive;

  /// The `sidebar_hover` token.
  final Color sidebarHover;

  /// The `hover` token.
  final Color hover;

  /// The `selected` token.
  final Color selected;

  /// The `active_line` token.
  final Color activeLine;

  /// The `button` token.
  final Color button;

  /// The `button_text` token.
  final Color buttonText;

  /// The `success` token.
  final Color success;

  /// The `warning` token.
  final Color warning;

  /// The `danger` token.
  final Color danger;

  /// Every colour, in the order of the fields: what equality and the hash
  /// are made of.
  List<Color> get values => <Color>[
    app,
    sidebar,
    list,
    editor,
    elevated,
    fill,
    text,
    textSecondary,
    textMuted,
    divider,
    accent,
    link,
    rowActive,
    rowHover,
    sidebarActive,
    sidebarHover,
    hover,
    selected,
    activeLine,
    button,
    buttonText,
    success,
    warning,
    danger,
  ];

  @override
  bool operator ==(Object other) {
    if (identical(this, other)) {
      return true;
    }
    if (other is! KansoColors) {
      return false;
    }
    final mine = values;
    final theirs = other.values;
    for (var i = 0; i < mine.length; i++) {
      if (mine[i] != theirs[i]) {
        return false;
      }
    }
    return true;
  }

  @override
  int get hashCode => Object.hashAll(values);

  @override
  KansoColors copyWith({
    Color? app,
    Color? sidebar,
    Color? list,
    Color? editor,
    Color? elevated,
    Color? fill,
    Color? text,
    Color? textSecondary,
    Color? textMuted,
    Color? divider,
    Color? accent,
    Color? link,
    Color? rowActive,
    Color? rowHover,
    Color? sidebarActive,
    Color? sidebarHover,
    Color? hover,
    Color? selected,
    Color? activeLine,
    Color? button,
    Color? buttonText,
    Color? success,
    Color? warning,
    Color? danger,
  }) {
    return KansoColors(
      app: app ?? this.app,
      sidebar: sidebar ?? this.sidebar,
      list: list ?? this.list,
      editor: editor ?? this.editor,
      elevated: elevated ?? this.elevated,
      fill: fill ?? this.fill,
      text: text ?? this.text,
      textSecondary: textSecondary ?? this.textSecondary,
      textMuted: textMuted ?? this.textMuted,
      divider: divider ?? this.divider,
      accent: accent ?? this.accent,
      link: link ?? this.link,
      rowActive: rowActive ?? this.rowActive,
      rowHover: rowHover ?? this.rowHover,
      sidebarActive: sidebarActive ?? this.sidebarActive,
      sidebarHover: sidebarHover ?? this.sidebarHover,
      hover: hover ?? this.hover,
      selected: selected ?? this.selected,
      activeLine: activeLine ?? this.activeLine,
      button: button ?? this.button,
      buttonText: buttonText ?? this.buttonText,
      success: success ?? this.success,
      warning: warning ?? this.warning,
      danger: danger ?? this.danger,
    );
  }

  @override
  KansoColors lerp(ThemeExtension<KansoColors>? other, double t) {
    if (other is! KansoColors) {
      return this;
    }
    return KansoColors(
      app: Color.lerp(app, other.app, t)!,
      sidebar: Color.lerp(sidebar, other.sidebar, t)!,
      list: Color.lerp(list, other.list, t)!,
      editor: Color.lerp(editor, other.editor, t)!,
      elevated: Color.lerp(elevated, other.elevated, t)!,
      fill: Color.lerp(fill, other.fill, t)!,
      text: Color.lerp(text, other.text, t)!,
      textSecondary: Color.lerp(textSecondary, other.textSecondary, t)!,
      textMuted: Color.lerp(textMuted, other.textMuted, t)!,
      divider: Color.lerp(divider, other.divider, t)!,
      accent: Color.lerp(accent, other.accent, t)!,
      link: Color.lerp(link, other.link, t)!,
      rowActive: Color.lerp(rowActive, other.rowActive, t)!,
      rowHover: Color.lerp(rowHover, other.rowHover, t)!,
      sidebarActive: Color.lerp(sidebarActive, other.sidebarActive, t)!,
      sidebarHover: Color.lerp(sidebarHover, other.sidebarHover, t)!,
      hover: Color.lerp(hover, other.hover, t)!,
      selected: Color.lerp(selected, other.selected, t)!,
      activeLine: Color.lerp(activeLine, other.activeLine, t)!,
      button: Color.lerp(button, other.button, t)!,
      buttonText: Color.lerp(buttonText, other.buttonText, t)!,
      success: Color.lerp(success, other.success, t)!,
      warning: Color.lerp(warning, other.warning, t)!,
      danger: Color.lerp(danger, other.danger, t)!,
    );
  }
}
