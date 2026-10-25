// Kanso — library_pane.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_sizes.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_dot.dart';
import '../../../core/views/widgets/kanso_icon_button.dart';
import '../../../core/views/widgets/kanso_mark.dart';
import '../../../core/views/widgets/kanso_scroll_host.dart';
import '../../../core/views/widgets/kanso_search_field.dart';
import '../../../core/views/widgets/kanso_section_header.dart';
import '../../../core/views/widgets/kanso_sidebar_row.dart';
import '../../models/data/mock_data.dart';

/// The library: the app's mark and settings, a search field, then All
/// Notes, Templates, Pinned, Recent and Tasks, the notebooks with their
/// counts, the tags and Trash.
///
/// A drawer on a phone, the first pane on a tablet. [startAtEnd] shows it
/// scrolled to its foot; [selectedNotebook] selects that notebook instead
/// of All Notes.
class LibraryPane extends StatelessWidget {
  const LibraryPane({
    this.startAtEnd = false,
    this.selectedNotebook,
    super.key,
  });

  final bool startAtEnd;
  final String? selectedNotebook;

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
    final l10n = context.l10n;
    final bottom = MediaQuery.paddingOf(context).bottom;
    return ColoredBox(
      color: colors.sidebar,
      child: Column(
        children: <Widget>[
          SafeArea(
            bottom: false,
            child: SizedBox(
              height: KansoSizes.barHeight,
              child: Padding(
                padding: const EdgeInsetsDirectional.only(
                  start: KansoSpacing.md,
                  end: KansoSpacing.xxs,
                ),
                child: Row(
                  children: <Widget>[
                    const KansoMark(),
                    const SizedBox(width: KansoSpacing.sm),
                    Expanded(
                      child: Text(
                        l10n.appName,
                        style: context.typography.barTitle,
                      ),
                    ),
                    KansoIconButton(
                      icon: KansoIcons.settings,
                      semanticLabel: l10n.openSettings,
                      onPressed: () {},
                    ),
                  ],
                ),
              ),
            ),
          ),
          Padding(
            padding: const EdgeInsets.fromLTRB(
              KansoSpacing.md,
              0,
              KansoSpacing.md,
              KansoSpacing.xs,
            ),
            child: KansoSearchField(hint: l10n.searchHint),
          ),
          Expanded(
            child: KansoScrollHost(
              startAtEnd: startAtEnd,
              builder: (context, controller) {
                return ListView(
                  controller: controller,
                  padding: EdgeInsets.only(
                    top: KansoSpacing.xxs,
                    bottom: bottom + KansoSpacing.md,
                  ),
                  children: <Widget>[
                    KansoSidebarRow(
                      icon: KansoIcons.files,
                      label: l10n.libraryAllNotes,
                      count: MockData.allNotesCount,
                      selected: selectedNotebook == null,
                    ),
                    KansoSidebarRow(
                      icon: KansoIcons.layoutTemplate,
                      label: l10n.libraryTemplates,
                    ),
                    KansoSidebarRow(
                      icon: KansoIcons.pin,
                      label: l10n.libraryPinned,
                      count: MockData.pinnedCount,
                    ),
                    KansoSidebarRow(
                      icon: KansoIcons.clock,
                      label: l10n.libraryRecent,
                    ),
                    KansoSidebarRow(
                      icon: KansoIcons.listChecks,
                      label: l10n.libraryTasks,
                      count: MockData.tasksCount,
                    ),
                    KansoSectionHeader(
                      label: l10n.libraryNotebooks,
                      trailing: <Widget>[
                        KansoIconButton(
                          icon: KansoIcons.arrowUpDown,
                          semanticLabel: l10n.librarySortNotebooks,
                          iconSize: KansoSizes.iconSmall + 2,
                          onPressed: () {},
                        ),
                        KansoIconButton(
                          icon: KansoIcons.plus,
                          semanticLabel: l10n.libraryNewNotebook,
                          iconSize: KansoSizes.iconSmall + 2,
                          onPressed: () {},
                        ),
                      ],
                    ),
                    for (final notebook in MockData.notebooks)
                      KansoSidebarRow(
                        icon: KansoIcons.notebook,
                        label: notebook.name,
                        count: notebook.count,
                        depth: notebook.depth,
                        selected: notebook.name == selectedNotebook,
                        chevron: true,
                      ),
                    const SizedBox(height: KansoSpacing.xs),
                    KansoSectionHeader(label: l10n.libraryTags),
                    for (final tag in MockData.tags)
                      KansoSidebarRow(
                        leading: KansoDot(color: tag.color),
                        label: tag.name,
                        count: tag.count,
                      ),
                    const SizedBox(height: KansoSpacing.xs),
                    KansoSidebarRow(
                      icon: KansoIcons.trash,
                      label: l10n.libraryTrash,
                      count: MockData.trashCount,
                    ),
                  ],
                );
              },
            ),
          ),
        ],
      ),
    );
  }
}
