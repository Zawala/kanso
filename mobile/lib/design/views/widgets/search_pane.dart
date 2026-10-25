// Kanso — search_pane.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_chip.dart';
import '../../../core/views/widgets/kanso_divider.dart';
import '../../../core/views/widgets/kanso_pressable.dart';
import '../../../core/views/widgets/kanso_search_field.dart';
import '../../models/data/mock_data.dart';
import 'search_result_tile.dart';

/// Search with a query typed: the focused field, filter chips and results
/// with the matched words highlighted. On a tablet it takes the place of
/// the note list.
class SearchPane extends StatelessWidget {
  const SearchPane({super.key});

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
    final l10n = context.l10n;
    final bottom = MediaQuery.paddingOf(context).bottom;
    return ColoredBox(
      color: colors.list,
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: <Widget>[
          SafeArea(
            bottom: false,
            child: Padding(
              padding: const EdgeInsets.fromLTRB(
                KansoSpacing.md,
                KansoSpacing.xs,
                KansoSpacing.md,
                0,
              ),
              child: Row(
                children: <Widget>[
                  Expanded(
                    child: KansoSearchField(
                      hint: l10n.searchHint,
                      query: MockData.searchQuery,
                      focused: true,
                      onClear: () {},
                      clearLabel: l10n.searchClear,
                    ),
                  ),
                  KansoPressable(
                    onTap: () {},
                    child: Padding(
                      padding: const EdgeInsets.symmetric(
                        horizontal: KansoSpacing.sm,
                        vertical: KansoSpacing.sm,
                      ),
                      child: Text(
                        l10n.searchCancel,
                        style: context.typography.body.copyWith(
                          color: colors.accent,
                        ),
                      ),
                    ),
                  ),
                ],
              ),
            ),
          ),
          Padding(
            padding: const EdgeInsets.fromLTRB(
              KansoSpacing.md,
              KansoSpacing.xs,
              KansoSpacing.md,
              KansoSpacing.sm,
            ),
            child: Wrap(
              spacing: KansoSpacing.xs,
              runSpacing: KansoSpacing.xs,
              children: <Widget>[
                KansoChip(label: l10n.searchFilterAll, selected: true),
                KansoChip(label: l10n.searchFilterTitles),
                KansoChip(label: l10n.searchFilterTags),
                KansoChip(label: l10n.searchFilterNotebook),
              ],
            ),
          ),
          const KansoDivider(),
          Padding(
            padding: const EdgeInsets.fromLTRB(
              KansoSpacing.md,
              KansoSpacing.sm,
              KansoSpacing.md,
              KansoSpacing.xxs,
            ),
            child: Text(
              l10n.searchResultCount(MockData.searchResults.length),
              style: context.typography.sectionLabel,
            ),
          ),
          Expanded(
            child: ListView(
              padding: EdgeInsets.only(bottom: bottom + KansoSpacing.md),
              children: <Widget>[
                for (
                  var i = 0;
                  i < MockData.searchResults.length;
                  i++
                ) ...<Widget>[
                  if (i > 0) const KansoDivider(indent: KansoSpacing.md),
                  SearchResultTile(
                    result: MockData.searchResults[i],
                    query: MockData.searchQuery,
                  ),
                ],
              ],
            ),
          ),
        ],
      ),
    );
  }
}
