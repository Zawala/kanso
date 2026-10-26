// Kanso — app_module.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:injectable/injectable.dart';

import '../models/data/design_gallery_access.dart';

/// What differs between the environments besides where the library lives.
@module
abstract class AppModule {
  /// The gallery of the design proposal is for developers.
  @Environment('development')
  @singleton
  DesignGalleryAccess get developmentGallery =>
      const DesignGalleryAccess(available: true);

  @Environment('production')
  @Environment('integration')
  @singleton
  DesignGalleryAccess get closedGallery =>
      const DesignGalleryAccess(available: false);
}
