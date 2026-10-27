// Kanso — rust_gallery_repository.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';
import 'package:injectable/injectable.dart';

import '../../../bridge/api/gallery.dart';
import '../../../bridge/api/types.dart';
import '../../../utils/kanso_error_mapping.dart';
import '../../data/application_error.dart';
import '../abs/gallery_repository.dart';

/// [GalleryRepository] over the Rust core's gallery.
@LazySingleton(as: GalleryRepository)
class RustGalleryRepository implements GalleryRepository {
  RustGalleryRepository(this._gallery);

  final KansoGallery _gallery;

  @override
  Future<Either<ApplicationError, List<GalleryItem>>> index({
    bool refresh = false,
  }) => attempt(() => _gallery.index(refresh: refresh));

  @override
  Future<Either<ApplicationError, List<GalleryItem>>> install(String id) =>
      attempt(() => _gallery.install(id: id));

  @override
  Future<Either<ApplicationError, List<GalleryItem>>> uninstall(String id) =>
      attempt(() => _gallery.uninstall(id: id));

  @override
  Future<Either<ApplicationError, List<BuiltinTheme>>> installedThemes() =>
      attempt(_gallery.installedThemes);

  @override
  Future<Either<ApplicationError, ThemeTokens>> installedThemeTokens(
    String id,
  ) => attempt(() => _gallery.installedThemeTokens(id: id));
}
