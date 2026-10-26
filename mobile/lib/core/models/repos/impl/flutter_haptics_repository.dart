// Kanso — flutter_haptics_repository.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/services.dart';
import 'package:injectable/injectable.dart';

import '../abs/haptics_repository.dart';

/// Haptics through Flutter's `HapticFeedback`: the system's own taps on
/// iOS and Android.
@LazySingleton(as: HapticsRepository)
class FlutterHapticsRepository implements HapticsRepository {
  @override
  Future<void> selection() => HapticFeedback.selectionClick();

  @override
  Future<void> impact() => HapticFeedback.lightImpact();
}
