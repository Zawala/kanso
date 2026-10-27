// Kanso — patch_repository.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

/// Which Shorebird patch the running app has.
// ignore: one_member_abstracts, a seam with one reader.
abstract interface class PatchRepository {
  /// The number of the patch the app is running, or null without Shorebird or
  /// before the first patch. Never throws.
  Future<int?> currentPatch();
}
