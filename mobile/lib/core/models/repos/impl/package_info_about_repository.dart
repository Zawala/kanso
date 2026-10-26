// Kanso — package_info_about_repository.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';
import 'package:injectable/injectable.dart';
import 'package:package_info_plus/package_info_plus.dart';
import 'package:url_launcher/url_launcher.dart';

import '../../../configs/kanso_links.dart';
import '../../data/app_version.dart';
import '../../data/application_error.dart';
import '../abs/about_repository.dart';

/// The app's version from the platform's package information, and the
/// website opened in the browser.
@LazySingleton(as: AboutRepository)
class PackageInfoAboutRepository implements AboutRepository {
  @override
  Future<Either<ApplicationError, AppVersion>> version() async {
    try {
      final info = await PackageInfo.fromPlatform();
      return Right<ApplicationError, AppVersion>(
        AppVersion(version: info.version, build: info.buildNumber),
      );
    } on Object catch (error) {
      return Left<ApplicationError, AppVersion>(
        ApplicationError.unexpected(message: '$error'),
      );
    }
  }

  @override
  Future<Either<ApplicationError, Unit>> openWebsite() async {
    try {
      final opened = await launchUrl(
        KansoLinks.website,
        mode: LaunchMode.externalApplication,
      );
      return opened
          ? const Right<ApplicationError, Unit>(unit)
          : const Left<ApplicationError, Unit>(
              ApplicationError.unexpected(message: 'The browser did not open'),
            );
    } on Object catch (error) {
      return Left<ApplicationError, Unit>(
        ApplicationError.unexpected(message: '$error'),
      );
    }
  }
}
