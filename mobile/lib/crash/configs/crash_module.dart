// Kanso — crash_module.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:io';

import 'package:injectable/injectable.dart';
import 'package:path/path.dart' as p;
import 'package:path_provider/path_provider.dart';

import '../models/data/crash_folder.dart';
import '../models/repos/abs/crash_context_repository.dart';
import '../models/repos/abs/crash_report_repository.dart';
import '../models/repos/impl/file_crash_report_repository.dart';
import 'crash_configs.dart';

/// Where the crash inbox lives and the repository over it.
@module
abstract class CrashModule {
  /// `crash-reports/` in the app's support folder: the same place the native
  /// code writes its raw drops (`filesDir` on Android, `Library/Application
  /// Support` on iOS).
  @Environment('development')
  @Environment('production')
  @singleton
  @preResolve
  Future<CrashFolder> crashFolder() async {
    final support = await getApplicationSupportDirectory();
    final folder = Directory(p.join(support.path, crashFolderName));
    await folder.create(recursive: true);
    return CrashFolder(folder.path);
  }

  /// A new, empty folder for an integration test.
  @Environment('integration')
  @singleton
  @preResolve
  Future<CrashFolder> temporaryCrashFolder() async {
    final root = await getTemporaryDirectory();
    final folder = await root.createTemp('kanso_crash_');
    return CrashFolder(folder.path);
  }

  @lazySingleton
  CrashReportRepository crashReports(
    CrashFolder folder,
    CrashContextRepository context,
  ) => FileCrashReportRepository(
    folder: folder,
    context: context,
    endpoint: CrashConfigs.endpoint,
  );
}
