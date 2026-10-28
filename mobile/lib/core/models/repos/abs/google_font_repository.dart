// Kanso — google_font_repository.dart
// Created 2026-10-29.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';

import '../../data/application_error.dart';

/// Downloads, caches and registers a Google Font family on this device.
// ignore: one_member_abstracts
abstract interface class GoogleFontRepository {
  /// Loads [family] from the cache or downloads it and registers it with the
  /// current Flutter engine. Returns the normalized family name.
  Future<Either<ApplicationError, String>> load(String family);

  /// Reads a previously downloaded font file for a WebView preview.
  Future<List<int>?> cachedBytes(String family);
}
