// Kanso — crash_folder.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:freezed_annotation/freezed_annotation.dart';
import 'package:path/path.dart' as p;

part 'crash_folder.freezed.dart';

/// The name of the inbox folder in the app's support folder.
const String crashFolderName = 'crash-reports';

/// The on-device inbox: `raw/` holds what the native code dropped, `pending/`
/// the scrubbed reports.
///
/// It is in the app's private storage and out of backups (the native code
/// marks it on iOS; Android's backup rules exclude all of `files/`).
@freezed
abstract class CrashFolder with _$CrashFolder {
  const factory CrashFolder(String path) = _CrashFolder;

  const CrashFolder._();

  /// Where the native code writes raw crashes.
  String get rawPath => p.join(path, 'raw');

  /// Where scrubbed reports wait.
  String get pendingPath => p.join(path, 'pending');
}
