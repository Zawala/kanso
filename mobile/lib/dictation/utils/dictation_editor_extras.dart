// Kanso — dictation_editor_extras.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:flutter/foundation.dart';
import 'package:flutter/widgets.dart';

import '../../core/configs/kanso_icons.dart';
import '../../editor_extras/models/data/editor_extras.dart';
import '../models/data/dictation_status.dart';
import '../models/repos/abs/dictation_text_rules.dart';
import '../state/dictation/dictation_bloc.dart';
import 'dictation_insertion.dart';

/// Dictation as the editor's extra: one key in the accessory bar, after the
/// Markdown keys, that turns the microphone on and off.
///
/// While it listens the key is lit ([EditorAccessoryExtra.active]). The
/// dictated range of the note the key was pressed in is kept up to date
/// through the [EditorTextTarget] the editor handed over, with what the
/// [DictationTextRules] make of everything said so far. Words still being
/// spoken are revised in place ([EditorTextTarget.revise]) as the recognizer
/// hears them, so the note shows the transcript live, while those revisions
/// collapse into one undo step. A finished phrase commits that step; each
/// phrase after it is one more step (see [DictationInsertion]). The words
/// being spoken are also shown, as the rules would write them, in the pill
/// above the bar ([preview]).
///
/// Dictation belongs to one note at a time. The editor calls [release] when
/// it closes the note or shows another, which stops the microphone and lets
/// go of the target.
class DictationEditorExtras implements EditorExtras {
  DictationEditorExtras({
    required this.bloc,
    required this.rules,
    this.label = 'Dictate',
  }) : insertion = DictationInsertion(rules: rules) {
    subscription = bloc.stream.listen(onState);
  }

  /// The id of the key in the accessory bar.
  static const String keyId = 'dictation';

  final DictationBloc bloc;
  final DictationTextRules rules;

  /// What a screen reader says of the key; the scope sets it in the
  /// language of the screen.
  String label;

  /// Whether the microphone is on, for the key to light.
  final ValueNotifier<bool> listening = ValueNotifier<bool>(false);

  /// What the words being spoken would be in the note, spacing and spoken
  /// punctuation included; empty when the microphone is off.
  final ValueNotifier<String> preview = ValueNotifier<String>('');

  late final StreamSubscription<DictationState> subscription;
  EditorTextTarget? target;
  DictationInsertion insertion;

  /// The work on the states so far: they are handled one after the other,
  /// as the rules answer later than a state comes.
  Future<void> queue = Future<void>.value();

  /// How many of the session's finished phrases are in the note.
  int written = 0;

  /// Invalidates queued or in-flight work from an older note/session.
  int generation = 0;

  @override
  List<EditorAccessoryExtra> accessoryKeys(EditorTextTarget target) =>
      <EditorAccessoryExtra>[
        EditorAccessoryExtra(
          id: keyId,
          icon: KansoIcons.mic,
          semanticLabel: label,
          onTap: (context, target) => toggle(target),
          active: listening,
        ),
      ];

  @override
  List<EditorSelectionAction> selectionActions(EditorTextTarget target) =>
      const <EditorSelectionAction>[];

  /// The key was pressed in the note [note]: start, or stop if it listens.
  void toggle(EditorTextTarget note) {
    if (bloc.state.active) {
      bloc.add(const DictationEvent.stopped());
      return;
    }
    target = note;
    generation++;
    insertion = DictationInsertion(rules: rules);
    written = 0;
    bloc.add(const DictationEvent.started(showOverlay: true));
  }

  /// The editor let go of [note]: stop listening and write nothing more.
  @override
  void release(EditorTextTarget note) {
    if (!identical(target, note)) {
      return;
    }
    target = null;
    generation++;
    if (bloc.state.active) {
      bloc.add(const DictationEvent.cancelled());
    }
  }

  /// Shows the words being spoken, writes the phrases the recognizer
  /// finished since the last state, and lets go of the note once the
  /// microphone is off.
  void onState(DictationState state) {
    listening.value = state.active;
    final note = target;
    if (note == null) {
      return;
    }
    final ticket = generation;
    queue = queue.then((_) => handle(note, state, ticket));
  }

  Future<void> handle(
    EditorTextTarget note,
    DictationState state,
    int ticket,
  ) async {
    if (!identical(target, note) || ticket != generation || !note.active) {
      return;
    }
    // Finish each phrase as its own undo step. The partial for this phrase
    // is already in the note; revising the same range and then committing
    // folds its final punctuation and wording into that one step.
    while (written < state.segments.length) {
      final phrases = state.segments.take(written + 1).toList();
      final edit = await insertion.next(note.value, phrases);
      if (!identical(target, note) || ticket != generation || !note.active) {
        return;
      }
      if (edit != null) {
        note.revise(edit.range, edit.text, joining: true);
      }
      note.commitRevision();
      written++;
    }
    // Show the words being spoken in the note while they are still changing.
    // Never ask the rules about an empty transcript: after a final phrase the
    // established range already says exactly what the recognizer last heard.
    if (state.partial.trim().isNotEmpty) {
      final edit = await insertion.next(
        note.value,
        state.segments,
        partial: state.partial,
      );
      if (!identical(target, note) || ticket != generation || !note.active) {
        return;
      }
      if (edit != null) {
        note.revise(edit.range, edit.text, joining: true);
      }
    }
    final shown = state.active
        ? await insertion.preview(note.value, state.segments, state.partial)
        : '';
    if (!identical(target, note) || ticket != generation || !note.active) {
      return;
    }
    preview.value = shown;
    if (state.status != DictationStatus.requesting && !state.active) {
      target = null;
      generation++;
    }
  }

  Future<void> dispose() async {
    generation++;
    if (target != null && bloc.state.active) {
      bloc.add(const DictationEvent.cancelled());
    }
    target = null;
    await subscription.cancel();
    listening.dispose();
    preview.dispose();
  }
}
