// Kanso — day_stepper.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../core/configs/kanso_icons.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_icon_button.dart';
import '../../state/journal/journal_bloc.dart';
import '../../utils/journal_flows.dart';

/// One of the journal's controls.
enum DayControl {
  /// The day before.
  previous,

  /// The day after.
  next,

  /// The calendar, to choose a day.
  calendar,
}

/// The controls of the journal: the day before, the day after and the
/// calendar. [openNote] is the daily note open, if any; the days are
/// counted from it (from today when it is none). It needs the
/// [JournalBloc] of a `JournalOpener` above it.
class DayStepper extends StatelessWidget {
  const DayStepper({this.openNote, super.key});

  final String? openNote;

  @override
  Widget build(BuildContext context) {
    return Row(
      mainAxisSize: MainAxisSize.min,
      children: <Widget>[
        for (final control in DayControl.values)
          DayControlButton(control: control, openNote: openNote),
      ],
    );
  }
}

/// The button of one of [DayStepper]'s controls.
class DayControlButton extends StatelessWidget {
  const DayControlButton({required this.control, this.openNote, super.key});

  final DayControl control;
  final String? openNote;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    return KansoIconButton(
      icon: switch (control) {
        DayControl.previous => KansoIcons.chevronLeft,
        DayControl.next => KansoIcons.chevronRight,
        DayControl.calendar => KansoIcons.calendar,
      },
      semanticLabel: switch (control) {
        DayControl.previous => l10n.journalPreviousDay,
        DayControl.next => l10n.journalNextDay,
        DayControl.calendar => l10n.journalCalendar,
      },
      onPressed: () => runDayControl(context, control, openNote: openNote),
    );
  }
}

/// Does what [control] does, counting from [openNote]. It needs the
/// [JournalBloc] of a `JournalOpener` above [context].
Future<void> runDayControl(
  BuildContext context,
  DayControl control, {
  String? openNote,
}) async {
  switch (control) {
    case DayControl.previous || DayControl.next:
      context.read<JournalBloc>().add(
        JournalEvent.stepRequested(
          openNote: openNote,
          forward: control == DayControl.next,
        ),
      );
    case DayControl.calendar:
      await showCalendar(context, openNote: openNote);
  }
}
