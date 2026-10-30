// Kanso — replace_page.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:go_router/go_router.dart';

import '../../../core/bridge/api/types.dart';
import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_button.dart';
import '../../../core/views/widgets/kanso_checkbox.dart';
import '../../../core/views/widgets/kanso_grouped_section.dart';
import '../../../core/views/widgets/kanso_icon_button.dart';
import '../../../core/views/widgets/kanso_inline_message.dart';
import '../../../core/views/widgets/kanso_page.dart';
import '../../../core/views/widgets/kanso_pressable.dart';
import '../../../core/views/widgets/kanso_progress.dart';
import '../../../core/views/widgets/kanso_section_header.dart';
import '../../../core/views/widgets/kanso_settings_row.dart';
import '../../../core/views/widgets/kanso_switch.dart';
import '../../../core/views/widgets/kanso_text_field.dart';
import '../../../core/views/widgets/kanso_top_bar.dart';
import '../../../gen/l10n/app_localizations.dart';
import '../../../injection.dart';
import '../../models/data/replace_option.dart';
import '../../models/data/replace_preview_status.dart';
import '../../models/data/replace_problem.dart';
import '../../state/replace_notes/replace_notes_bloc.dart';

/// Finds and replaces text across the library, with a single undoable batch.
class ReplacePage extends StatefulWidget {
  const ReplacePage({super.key});

  @override
  State<ReplacePage> createState() => _ReplacePageState();
}

class _ReplacePageState extends State<ReplacePage> {
  late final TextEditingController findController;
  late final TextEditingController replacementController;

  @override
  void initState() {
    super.initState();
    findController = TextEditingController();
    replacementController = TextEditingController();
  }

  @override
  void dispose() {
    findController.dispose();
    replacementController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    return BlocProvider<ReplaceNotesBloc>(
      create: (_) => getIt<ReplaceNotesBloc>(),
      child: KansoPage(
        backgroundColor: context.colors.app,
        body: Column(
          children: <Widget>[
            KansoTopBar(
              title: l10n.replaceTitle,
              leading: KansoIconButton(
                icon: KansoIcons.close,
                semanticLabel: l10n.replaceClose,
                onPressed: () => context.pop(),
              ),
            ),
            Expanded(
              child: BlocBuilder<ReplaceNotesBloc, ReplaceNotesState>(
                builder: (context, state) {
                  final bloc = context.read<ReplaceNotesBloc>();
                  return ListView(
                    padding: const EdgeInsets.fromLTRB(
                      KansoSpacing.md,
                      KansoSpacing.md,
                      KansoSpacing.md,
                      KansoSpacing.xxl,
                    ),
                    children: <Widget>[
                      KansoTextField(
                        controller: findController,
                        label: l10n.replaceFind,
                        textInputAction: TextInputAction.next,
                        onChanged: (text) => bloc.add(
                          ReplaceNotesEvent.findChanged(text),
                        ),
                      ),
                      const SizedBox(height: KansoSpacing.sm),
                      KansoTextField(
                        controller: replacementController,
                        label: l10n.replaceWith,
                        textInputAction: TextInputAction.done,
                        onChanged: (text) => bloc.add(
                          ReplaceNotesEvent.replacementChanged(text),
                        ),
                      ),
                      const SizedBox(height: KansoSpacing.lg),
                      KansoGroupedSection(
                        title: l10n.replaceOptionsLabel,
                        dividerIndent: KansoSettingsRow.iconInset,
                        children: <Widget>[
                          for (final option in ReplaceOption.values)
                            KansoSettingsRow(
                              label: _optionLabel(l10n, option),
                              onTap: () => bloc.add(
                                ReplaceNotesEvent.optionToggled(option),
                              ),
                              trailing: KansoSwitch(
                                value: _optionValue(state, option),
                                onChanged: (_) => bloc.add(
                                  ReplaceNotesEvent.optionToggled(option),
                                ),
                              ),
                            ),
                        ],
                      ),
                      if (state.problem != null) ...<Widget>[
                        const SizedBox(height: KansoSpacing.md),
                        KansoInlineMessage(
                          message: switch (state.problem!) {
                            ReplaceProblem.badPattern =>
                              l10n.replaceProblemBadPattern,
                            ReplaceProblem.applyFailed =>
                              l10n.replaceProblemApply,
                            ReplaceProblem.undoFailed =>
                              l10n.replaceProblemUndo,
                          },
                        ),
                      ],
                      if (state.outcome case final outcome?) ...<Widget>[
                        const SizedBox(height: KansoSpacing.md),
                        KansoInlineMessage(
                          message: <String>[
                            l10n.replaceDone(outcome.replaced, outcome.notes),
                            if (outcome.skipped > 0)
                              l10n.replaceSkipped(outcome.skipped),
                            if (state.undone case final undone?)
                              l10n.replaceUndone(undone.restored),
                          ].join('\n'),
                          tone: KansoMessageTone.success,
                        ),
                        if (state.undone == null)
                          Padding(
                            padding: const EdgeInsets.only(
                              top: KansoSpacing.xs,
                            ),
                            child: KansoButton(
                              label: state.undoing
                                  ? l10n.replaceUndoing
                                  : l10n.replaceUndo,
                              variant: KansoButtonVariant.secondary,
                              icon: KansoIcons.undo,
                              onPressed: state.undoing
                                  ? null
                                  : () => bloc.add(
                                      const ReplaceNotesEvent.undoRequested(),
                                    ),
                            ),
                          ),
                      ],
                      const SizedBox(height: KansoSpacing.lg),
                      ..._preview(context, state, bloc, l10n),
                    ],
                  );
                },
              ),
            ),
          ],
        ),
      ),
    );
  }

  /// What the preview of the matches says, note by note, or why it can't.
  List<Widget> _preview(
    BuildContext context,
    ReplaceNotesState state,
    ReplaceNotesBloc bloc,
    AppLocalizations l10n,
  ) {
    if (state.previewStatus == ReplacePreviewStatus.loading) {
      return <Widget>[
        Semantics(
          label: l10n.replaceSearching,
          child: const KansoProgressBar(),
        ),
      ];
    }
    if (state.options.find.isEmpty) {
      return <Widget>[_Quiet(l10n.replaceTypeToFind)];
    }
    if (state.previewStatus == ReplacePreviewStatus.failed) {
      return <Widget>[
        KansoInlineMessage(message: l10n.replaceProblemSearch),
      ];
    }
    if (state.preview case final preview?) {
      if (preview.notes.isEmpty) {
        return <Widget>[_Quiet(l10n.replaceNoMatches)];
      }
      return <Widget>[
        KansoSectionHeader(
          label: l10n.replaceFound(preview.totalMatches, preview.notes.length),
          horizontalPadding: 0,
        ),
        KansoGroupedSection(
          children: <Widget>[
            for (final note in preview.notes)
              _PreviewNoteRow(
                note: note,
                included: !state.excluded.contains(note.id),
                onToggle: () => bloc.add(
                  ReplaceNotesEvent.noteToggled(note.id),
                ),
              ),
          ],
        ),
        const SizedBox(height: KansoSpacing.md),
        KansoButton(
          label: state.applying
              ? l10n.replaceApplying
              : l10n.replaceApply(state.matches, state.chosen.length),
          onPressed: state.planned.isEmpty || state.applying
              ? null
              : () => bloc.add(const ReplaceNotesEvent.applyRequested()),
        ),
      ];
    }
    if (state.previewStatus == ReplacePreviewStatus.ready) {
      return <Widget>[_Quiet(l10n.replaceNoMatches)];
    }
    return const <Widget>[];
  }
}

String _optionLabel(AppLocalizations l10n, ReplaceOption option) {
  return switch (option) {
    ReplaceOption.matchCase => l10n.replaceMatchCase,
    ReplaceOption.wholeWord => l10n.replaceWholeWord,
    ReplaceOption.regex => l10n.replaceRegex,
    ReplaceOption.skipCode => l10n.replaceSkipCode,
    ReplaceOption.includeTrash => l10n.replaceIncludeTrash,
  };
}

bool _optionValue(ReplaceNotesState state, ReplaceOption option) {
  return switch (option) {
    ReplaceOption.matchCase => state.options.matchCase,
    ReplaceOption.wholeWord => state.options.wholeWord,
    ReplaceOption.regex => state.options.regex,
    ReplaceOption.skipCode => state.options.skipCode,
    ReplaceOption.includeTrash => state.options.includeTrash,
  };
}

/// A note the replace would change, with its matches, and a box that leaves
/// it out when cleared.
class _PreviewNoteRow extends StatelessWidget {
  const _PreviewNoteRow({
    required this.note,
    required this.included,
    required this.onToggle,
  });

  final ReplaceNote note;
  final bool included;
  final VoidCallback onToggle;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final title = note.title.isEmpty ? l10n.replaceUntitled : note.title;
    final details = <String>[
      if (note.trashed) l10n.replaceInTrash,
      for (final match in note.matches.take(3))
        l10n.replaceLine(match.lineNumber),
      if (note.matches.length > 3) l10n.replaceMore(note.matches.length - 3),
    ];
    return KansoPressable(
      onTap: onToggle,
      child: Padding(
        padding: const EdgeInsets.symmetric(
          horizontal: KansoSpacing.md,
          vertical: KansoSpacing.sm,
        ),
        child: Row(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: <Widget>[
            Padding(
              padding: const EdgeInsets.only(top: 1),
              child: KansoCheckbox(
                value: included,
                onChanged: (_) => onToggle(),
              ),
            ),
            const SizedBox(width: KansoSpacing.sm),
            Expanded(
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: <Widget>[
                  Text(
                    title,
                    maxLines: 1,
                    overflow: TextOverflow.ellipsis,
                    style: context.typography.listTitle,
                  ),
                  for (final line in details)
                    Text(
                      line,
                      maxLines: 1,
                      overflow: TextOverflow.ellipsis,
                      style: context.typography.meta,
                    ),
                ],
              ),
            ),
            const SizedBox(width: KansoSpacing.xs),
            Text('${note.count}', style: context.typography.meta),
          ],
        ),
      ),
    );
  }
}

/// A sentence that only says where things stand.
class _Quiet extends StatelessWidget {
  const _Quiet(this.text);

  final String text;

  @override
  Widget build(BuildContext context) {
    return Text(text, style: context.typography.secondary);
  }
}
