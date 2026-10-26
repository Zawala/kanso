// Kanso — note_theme_sync.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/widgets.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../state/note/note_bloc.dart';

/// Tells the `NoteBloc` above it which built-in theme [themeId] the screen
/// is drawn in whenever that changes, so the text is highlighted in the
/// colours around it.
class NoteThemeSync extends StatefulWidget {
  const NoteThemeSync({required this.themeId, required this.child, super.key});

  final String themeId;
  final Widget child;

  @override
  State<NoteThemeSync> createState() => _NoteThemeSyncState();
}

class _NoteThemeSyncState extends State<NoteThemeSync> {
  @override
  void didUpdateWidget(NoteThemeSync oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (oldWidget.themeId != widget.themeId) {
      context.read<NoteBloc>().add(
        NoteEvent.syntaxThemeChanged(widget.themeId),
      );
    }
  }

  @override
  Widget build(BuildContext context) => widget.child;
}
