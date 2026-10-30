// Kanso — graph_filter_picker.dart
// Created 2026-10-28.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../core/bridge/api/types.dart';
import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/state/library_index/library_index_bloc.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_bottom_sheet_route.dart';
import '../../../core/views/widgets/kanso_check_row.dart';
import '../../../core/views/widgets/kanso_grouped_section.dart';
import '../../../core/views/widgets/kanso_icon.dart';
import '../../../core/views/widgets/kanso_pressable.dart';
import '../../../core/views/widgets/kanso_section_header.dart';
import '../../../core/views/widgets/kanso_sheet_group.dart';

/// Chooses all notes, a notebook, or a tag for the library graph.
///
/// The row says what is being shown now; tapping it opens a sheet of the
/// choices, grouped as the library is, with the one in use checked.
class GraphFilterPicker extends StatelessWidget {
  const GraphFilterPicker({
    required this.request,
    required this.onSelected,
    super.key,
  });

  final GraphRequest request;
  final ValueChanged<GraphFilter> onSelected;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final index = context.watch<LibraryIndexBloc>().state;
    final filter = switch (request) {
      GraphRequest_Library(:final filter) => filter,
      GraphRequest_Local() => const GraphFilter.all(),
    };
    final label = switch (filter) {
      GraphFilter_All() => l10n.graphFilterAll,
      GraphFilter_Book(:final id) =>
        index.notebooks.where((row) => row.id == id).firstOrNull?.name ??
            l10n.graphFilterNotebook,
      GraphFilter_Tag(:final id) =>
        index.tags.where((row) => row.id == id).firstOrNull?.name ??
            l10n.graphFilterTag,
    };
    return KansoPressable(
      onTap: () => _pick(context, index, filter),
      semanticLabel: l10n.graphFilterAll,
      child: ConstrainedBox(
        constraints: const BoxConstraints(minHeight: 36),
        child: Row(
          children: <Widget>[
            Expanded(
              child: Text(
                label,
                maxLines: 1,
                overflow: TextOverflow.ellipsis,
                style: context.typography.body,
              ),
            ),
            const SizedBox(width: KansoSpacing.xxs),
            KansoIcon(KansoIcons.chevronDown, color: context.colors.textMuted),
          ],
        ),
      ),
    );
  }

  Future<void> _pick(
    BuildContext context,
    LibraryIndexState index,
    GraphFilter filter,
  ) async {
    final l10n = context.l10n;
    final books = index.notebooks.where((row) => !row.locked).toList();
    final tags = index.tags;
    final chosen = await showKansoBottomSheet<GraphFilter>(
      context,
      builder: (sheet) => Padding(
        padding: const EdgeInsetsDirectional.fromSTEB(
          KansoSpacing.md,
          0,
          KansoSpacing.md,
          KansoSpacing.lg,
        ),
        child: Column(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: <Widget>[
            Text(l10n.graphFilterTitle, style: context.typography.heading1),
            const SizedBox(height: KansoSpacing.md),
            Flexible(
              child: SingleChildScrollView(
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.stretch,
                  children: <Widget>[
                    KansoSheetGroup(
                      children: <Widget>[
                        KansoCheckRow(
                          label: l10n.graphFilterAll,
                          icon: KansoIcons.network,
                          checked: filter is GraphFilter_All,
                          onTap: () =>
                              Navigator.of(sheet).pop(const GraphFilter.all()),
                        ),
                      ],
                    ),
                    if (books.isNotEmpty) ...<Widget>[
                      const SizedBox(height: KansoSpacing.md),
                      KansoSectionHeader(
                        label: l10n.graphFilterNotebooks,
                        horizontalPadding: 0,
                      ),
                      KansoGroupedSection(
                        children: <Widget>[
                          for (final book in books)
                            KansoCheckRow(
                              label: book.name,
                              icon: KansoIcons.book,
                              checked: switch (filter) {
                                GraphFilter_Book(:final id) => id == book.id,
                                _ => false,
                              },
                              onTap: () => Navigator.of(
                                sheet,
                              ).pop(GraphFilter.book(id: book.id)),
                            ),
                        ],
                      ),
                    ],
                    if (tags.isNotEmpty) ...<Widget>[
                      const SizedBox(height: KansoSpacing.md),
                      KansoSectionHeader(
                        label: l10n.graphFilterTags,
                        horizontalPadding: 0,
                      ),
                      KansoGroupedSection(
                        children: <Widget>[
                          for (final tag in tags)
                            KansoCheckRow(
                              label: tag.name,
                              icon: KansoIcons.tag,
                              checked: switch (filter) {
                                GraphFilter_Tag(:final id) => id == tag.id,
                                _ => false,
                              },
                              onTap: () => Navigator.of(
                                sheet,
                              ).pop(GraphFilter.tag(id: tag.id)),
                            ),
                        ],
                      ),
                    ],
                  ],
                ),
              ),
            ),
          ],
        ),
      ),
    );
    if (chosen != null) {
      onSelected(chosen);
    }
  }
}
