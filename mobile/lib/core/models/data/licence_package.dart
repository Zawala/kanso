// Kanso — licence_package.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:freezed_annotation/freezed_annotation.dart';

part 'licence_package.freezed.dart';

/// A package the app is made of and the licences it comes under.
@freezed
abstract class LicencePackage with _$LicencePackage {
  const factory LicencePackage({
    /// The package's name.
    required String name,

    /// The text of each licence, its paragraphs joined by blank lines.
    required List<String> licences,
  }) = _LicencePackage;
}
