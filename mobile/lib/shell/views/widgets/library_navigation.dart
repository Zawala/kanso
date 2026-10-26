// Kanso — library_navigation.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:go_router/go_router.dart';

import '../../../core/bridge/api/types.dart';
import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_sizes.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/models/data/browse_location.dart';
import '../../../core/router/kanso_routes.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/utils/place_icons.dart';
import '../../../core/utils/place_labels.dart';
import '../../../core/views/widgets/kanso_drawer_scope.dart';
import '../../../core/views/widgets/kanso_icon_button.dart';
import '../../../core/views/widgets/kanso_mark.dart';
import '../../../core/views/widgets/kanso_sidebar_row.dart';

/// The way around the library: the app's mark and settings, and a row for
/// each place. It is the first pane of a wide window and the drawer of a
/// narrow one.
class LibraryNavigation extends StatelessWidget {
  const LibraryNavigation({
    required this.location,
    this.designGallery = false,
    super.key,
  });

  /// Where the person is; the row of its place is selected.
  final BrowseLocation? location;

  /// Whether to offer the design gallery.
  final bool designGallery;

  /// The places with a row of their own, in order.
  static const List<ListPlace> places = <ListPlace>[
    ListPlace.all(),
    ListPlace.pinned(),
    ListPlace.recent(),
    ListPlace.trash(),
  ];

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final here = location;
    final current = here is PlaceLocation ? here.place : null;
    void leave(VoidCallback go) {
      // A drawer slides away as the page behind it changes.
      KansoDrawerScope.maybeOf(context)?.close();
      go();
    }

    return ColoredBox(
      color: context.colors.sidebar,
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
                      onPressed: () =>
                          leave(() => context.push(KansoRoutes.settings)),
                    ),
                  ],
                ),
              ),
            ),
          ),
          Expanded(
            child: ListView(
              padding: EdgeInsets.only(
                top: KansoSpacing.xxs,
                bottom: MediaQuery.paddingOf(context).bottom + KansoSpacing.md,
              ),
              children: <Widget>[
                for (final place in places)
                  KansoSidebarRow(
                    icon: placeIcon(place),
                    label: placeLabel(l10n, place),
                    selected: current == place,
                    onTap: () => leave(
                      () => context.go(BrowseLocation.place(place).location),
                    ),
                  ),
                if (designGallery)
                  KansoSidebarRow(
                    icon: KansoIcons.palette,
                    label: l10n.libraryDesignGallery,
                    onTap: () => leave(() => context.push(KansoRoutes.design)),
                  ),
              ],
            ),
          ),
        ],
      ),
    );
  }
}
