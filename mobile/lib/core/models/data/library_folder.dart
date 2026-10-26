// Kanso — library_folder.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:freezed_annotation/freezed_annotation.dart';
import 'package:path/path.dart' as p;

part 'library_folder.freezed.dart';

/// The file name of the library's database.
const String libraryFileName = 'library.db';

/// The file name of the preferences, beside the database.
const String preferencesFileName = 'preferences.json';

/// The folder the library lives in: its database and the preferences.
///
/// It is in the app's private storage, and kept out of device backups.
@freezed
abstract class LibraryFolder with _$LibraryFolder {
  const factory LibraryFolder(String path) = _LibraryFolder;

  const LibraryFolder._();

  /// The library's database.
  String get databasePath => p.join(path, libraryFileName);

  /// The preferences file.
  String get preferencesPath => p.join(path, preferencesFileName);
}
