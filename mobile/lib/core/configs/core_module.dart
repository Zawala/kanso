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

/// Where the library lives: `kanso/library.db` in the app's support folder
/// (`filesDir` on Android, `Library/Application Support` on iOS).
///
/// The background sync opens the same file, so changing this changes it
/// there too.
const String libraryFolderName = 'kanso';

/// The file name of the library's database.
const String libraryFileName = 'library.db';

/// Opens the library and hands out its parts.
@module
abstract class CoreModule {
  /// The open library. `KansoCore.init` must have run first.
  @singleton
  @preResolve
  Future<KansoLibrary> library() async {
    final support = await getApplicationSupportDirectory();
    final folder = Directory(p.join(support.path, libraryFolderName));
    await folder.create(recursive: true);
    return KansoLibrary.open(path: p.join(folder.path, libraryFileName));
  }

  @lazySingleton
  KansoLocks locks(KansoLibrary library) => library.locks();

  @lazySingleton
  KansoSync sync(KansoLibrary library) => library.syncing();

  @lazySingleton
  KansoRenderer renderer(KansoLibrary library) => library.renderer();
}
