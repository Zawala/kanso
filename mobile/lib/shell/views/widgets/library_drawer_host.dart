// Kanso — library_drawer_host.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/widgets.dart';

import '../../../core/models/data/browse_location.dart';
import '../../../core/views/widgets/kanso_drawer.dart';
import 'library_navigation.dart';

/// Puts the library in a drawer over [child]: the way to the library when
/// the window has no room for it beside the list.
class LibraryDrawerHost extends StatelessWidget {
  const LibraryDrawerHost({
    required this.location,
    required this.width,
    required this.child,
    this.designGallery = false,
    this.edgeDragEnabled = false,
    super.key,
  });

  final BrowseLocation? location;
  final bool designGallery;

  /// How wide the drawer is.
  final double width;

  /// Whether a drag from the start edge opens it.
  final bool edgeDragEnabled;
  final Widget child;

  @override
  Widget build(BuildContext context) {
    return KansoDrawer(
      width: width,
      edgeDragEnabled: edgeDragEnabled,
      drawer: LibraryNavigation(
        location: location,
        designGallery: designGallery,
      ),
      child: child,
    );
  }
}
