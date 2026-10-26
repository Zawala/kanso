// Kanso — library_navigation.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/widgets.dart';

import '../../../core/models/data/browse_location.dart';
import '../../../library/views/widgets/library_pane.dart';

/// The way around the library: the first pane of a wide window and the
/// drawer of a narrow one.
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

  @override
  Widget build(BuildContext context) {
    return LibraryPane(location: location, designGallery: designGallery);
  }
}
