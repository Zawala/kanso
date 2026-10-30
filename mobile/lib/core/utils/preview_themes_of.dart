// Kanso — preview_themes_of.dart
// Created 2026-10-30.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../models/data/preview_themes.dart';
import '../models/data/theme_selection.dart';
import '../state/appearance/appearance_bloc.dart';
import 'context_extensions.dart';

/// The themes [selection] puts in force for the light or dark slot, as
/// [dark] says: the ones a note's preview and its exports are drawn in.
PreviewThemes previewThemesFor(
  ThemeSelection selection, {
  required bool dark,
}) => (
  ui: selection.uiFor(systemDark: dark),
  syntax: selection.syntaxFor(systemDark: dark),
  preview: selection.previewFor(systemDark: dark),
  dark: dark,
);

/// The themes in force now, for [context]'s brightness. It reads the
/// appearance once, so it can be called outside building, as an export is.
PreviewThemes previewThemesOf(BuildContext context) => previewThemesFor(
  context.read<AppearanceBloc>().state.selection,
  dark: context.brightness == Brightness.dark,
);
