// Kanso — core_module.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:io';

import 'package:injectable/injectable.dart';
import 'package:path/path.dart' as p;
import 'package:path_provider/path_provider.dart';

import '../bridge/api/library.dart';
import '../bridge/api/locks.dart';
import '../bridge/api/renderer.dart';
import '../bridge/api/sync.dart';
import '../models/data/library_folder.dart';

/// Where the library lives: `kanso/` in the app's support folder
/// (`filesDir` on Android, `Library/Application Support` on iOS), with
/// `library.db` and `preferences.json` in it.
///
/// The background sync opens the same file, so changing this changes it
/// there too.
const String libraryFolderName = 'kanso';

/// Opens the library and hands out its parts.
@module
abstract class CoreModule {
  /// The person's library folder, made if it is not there.
  @Environment('development')
  @Environment('production')
  @singleton
  @preResolve
  Future<LibraryFolder> libraryFolder() async {
    final support = await getApplicationSupportDirectory();
    final folder = Directory(p.join(support.path, libraryFolderName));
    await folder.create(recursive: true);
    return LibraryFolder(folder.path);
  }

  /// A new, empty folder for an integration test.
  @Environment('integration')
  @singleton
  @preResolve
  Future<LibraryFolder> temporaryLibraryFolder() async {
    final root = await getTemporaryDirectory();
    final folder = await root.createTemp('kanso_integration_');
    return LibraryFolder(folder.path);
  }

  /// The open library. `KansoCore.init` must have run first.
  @singleton
  @preResolve
  Future<KansoLibrary> library(LibraryFolder folder) =>
      KansoLibrary.open(path: folder.databasePath);

  @lazySingleton
  KansoLocks locks(KansoLibrary library) => library.locks();

  @lazySingleton
  KansoSync sync(KansoLibrary library) => library.syncing();

  @lazySingleton
  KansoRenderer renderer(KansoLibrary library) => library.renderer();
}
