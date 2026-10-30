// Kanso — focus_live_preview_sync.dart
// Created 2026-10-30.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/widgets.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../shell/state/focus_mode/focus_mode_bloc.dart';
import '../../models/data/note_mode.dart';
import '../../state/note/note_bloc.dart';

/// With Live Preview in focus mode turned on, shows the note of the
/// `NoteBloc` above it in Live as focus mode starts, and in the mode it was
/// in as focus mode ends. A mode picked while in focus mode stands: leaving
/// only undoes the switch to Live while Live still shows.
///
/// Turning the setting on or off counts from the next time focus mode
/// starts. It needs a `FocusModeBloc` above it.
class FocusLivePreviewSync extends StatefulWidget {
  const FocusLivePreviewSync({required this.child, super.key});

  final Widget child;

  @override
  State<FocusLivePreviewSync> createState() => _FocusLivePreviewSyncState();
}

class _FocusLivePreviewSyncState extends State<FocusLivePreviewSync> {
  /// The mode focus mode set aside for Live, and the note it was set aside
  /// in; null while there is nothing to go back to.
  NoteMode? _before;
  String? _beforeNote;

  void _onFocusChanged(BuildContext context, FocusModeState focus) {
    final note = context.read<NoteBloc>();
    final mode = note.state.mode;
    if (focus.active) {
      if (focus.livePreview && mode != NoteMode.live) {
        _before = mode;
        _beforeNote = note.state.noteId;
        note.add(const NoteEvent.modeChanged(NoteMode.live));
      } else {
        _forget();
      }
      return;
    }
    final before = _before;
    final sameNote = _beforeNote == note.state.noteId;
    _forget();
    if (before != null && sameNote && mode == NoteMode.live) {
      note.add(NoteEvent.modeChanged(before));
    }
  }

  void _forget() {
    _before = null;
    _beforeNote = null;
  }

  @override
  Widget build(BuildContext context) {
    return MultiBlocListener(
      listeners: <BlocListener<dynamic, dynamic>>[
        BlocListener<FocusModeBloc, FocusModeState>(
          listenWhen: (before, after) => before.active != after.active,
          listener: _onFocusChanged,
        ),
        // Another mode than Live, or another note, was picked meanwhile.
        BlocListener<NoteBloc, NoteState>(
          listenWhen: (before, after) =>
              before.mode != after.mode || before.noteId != after.noteId,
          listener: (context, note) {
            if (note.mode != NoteMode.live || note.noteId != _beforeNote) {
              _forget();
            }
          },
        ),
      ],
      child: widget.child,
    );
  }
}
