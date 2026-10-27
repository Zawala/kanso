// Kanso — crash_context_repository.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import '../../data/crash_context.dart';

/// The build and the phone, as a report says them.
// ignore: one_member_abstracts, a seam with one reader.
abstract interface class CrashContextRepository {
  /// The version, build and patch of the app, the OS and the device model.
  /// Anything unknown is `unknown` or null; never throws.
  Future<CrashContext> read();
}
