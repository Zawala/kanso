// Kanso — shorebird_patch_repository.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/foundation.dart';
import 'package:injectable/injectable.dart';
import 'package:shorebird_code_push/shorebird_code_push.dart';

import '../abs/patch_repository.dart';

/// The patch number from Shorebird's updater. Without the Shorebird engine
/// (a local run, a test) the updater says it is unavailable and there is none.
@LazySingleton(as: PatchRepository)
class ShorebirdPatchRepository implements PatchRepository {
  ShorebirdPatchRepository() : _updater = null;

  /// With [updater], for tests.
  @visibleForTesting
  ShorebirdPatchRepository.withUpdater(ShorebirdUpdater updater)
    : _updater = updater;

  ShorebirdUpdater? _updater;

  @override
  Future<int?> currentPatch() async {
    try {
      final updater = _updater ??= ShorebirdUpdater();
      if (!updater.isAvailable) {
        return null;
      }
      return (await updater.readCurrentPatch())?.number;
    } on Object {
      return null;
    }
  }
}
