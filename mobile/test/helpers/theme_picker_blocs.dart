// Kanso — theme_picker_blocs.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/widgets.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:kanso/core/models/data/load_status.dart';
import 'package:kanso/themes/state/theme_picker/theme_picker_bloc.dart';

import 'close_bloc_later.dart';
import 'real_themes.dart';

/// A `ThemePickerBloc` that has read the real built-in themes (see
/// [loadRealThemes]).
Future<ThemePickerBloc> startThemePicker() async {
  final picker = ThemePickerBloc(RealThemesRenderer())
    ..add(const ThemePickerEvent.started());
  closeBlocLater(picker);
  await picker.stream.firstWhere((state) => state.status == LoadStatus.ready);
  return picker;
}

/// [child] with such a bloc above it.
Future<Widget> withThemePicker(Widget child) async {
  return BlocProvider<ThemePickerBloc>.value(
    value: await startThemePicker(),
    child: child,
  );
}
