// Kanso — browse_shell.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/widgets.dart';

import '../../../core/models/data/browse_location.dart';

/// The frame around every place, search and note.
class BrowseShell extends StatelessWidget {
  const BrowseShell({
    required this.location,
    required this.child,
    this.designGallery = false,
    super.key,
  });

  /// Where the person is; null for an address that is neither a place nor
  /// a search.
  final BrowseLocation? location;

  /// Whether the design gallery can be opened from here.
  final bool designGallery;

  /// The navigator showing the pages of the current address.
  final Widget child;

  @override
  Widget build(BuildContext context) => child;
}
