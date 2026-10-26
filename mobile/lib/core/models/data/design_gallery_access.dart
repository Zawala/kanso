// Kanso — design_gallery_access.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:freezed_annotation/freezed_annotation.dart';

part 'design_gallery_access.freezed.dart';

/// Whether this build lets one open the gallery of the design proposal: the
/// development environment does, the others do not.
@freezed
abstract class DesignGalleryAccess with _$DesignGalleryAccess {
  const factory DesignGalleryAccess({required bool available}) =
      _DesignGalleryAccess;
}
