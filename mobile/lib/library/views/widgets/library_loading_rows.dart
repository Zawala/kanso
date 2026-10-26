// Kanso — library_loading_rows.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/widgets.dart';

import '../../../core/configs/kanso_sizes.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_skeleton.dart';

/// The library's rows while they are read: still bars where the names and
/// counts will be, with the rhythm of the real rows.
class LibraryLoadingRows extends StatelessWidget {
  const LibraryLoadingRows({super.key});

  /// How wide each bar's name is, as a share of the row; it repeats.
  static const List<double> nameFractions = <double>[0.5, 0.36, 0.44, 0.58];

  /// How many rows are drawn.
  static const int rows = 9;

  @override
  Widget build(BuildContext context) {
    return Semantics(
      label: context.l10n.libraryLoading,
      container: true,
      child: Column(
        children: <Widget>[
          for (var i = 0; i < rows; i++)
            SizedBox(
              height: KansoSizes.sidebarRow,
              child: Padding(
                padding: const EdgeInsets.symmetric(
                  horizontal: KansoSpacing.md,
                ),
                child: Row(
                  children: <Widget>[
                    const KansoSkeleton(
                      width: KansoSizes.icon,
                      height: KansoSizes.icon,
                    ),
                    const SizedBox(width: KansoSpacing.sm),
                    Expanded(
                      child: Align(
                        alignment: AlignmentDirectional.centerStart,
                        child: FractionallySizedBox(
                          widthFactor: nameFractions[i % nameFractions.length],
                          child: const KansoSkeleton(),
                        ),
                      ),
                    ),
                  ],
                ),
              ),
            ),
        ],
      ),
    );
  }
}
