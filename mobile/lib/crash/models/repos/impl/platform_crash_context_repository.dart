// Kanso — platform_crash_context_repository.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:io';

import 'package:flutter/services.dart';
import 'package:injectable/injectable.dart';
import 'package:package_info_plus/package_info_plus.dart';

import '../../../../core/models/repos/abs/patch_repository.dart';
import '../../../configs/crash_configs.dart';
import '../../../utils/crash_scrubber.dart';
import '../../data/crash_app.dart';
import '../../data/crash_context.dart';
import '../../data/crash_os.dart';
import '../../data/crash_os_name.dart';
import '../abs/crash_context_repository.dart';

/// The running app's version (package information), patch (Shorebird) and
/// the OS and device model (the native `app.kanso/crash` channel, with
/// `dart:io` as the fallback for the OS version).
@LazySingleton(as: CrashContextRepository)
class PlatformCrashContextRepository implements CrashContextRepository {
  PlatformCrashContextRepository(this._patches);

  final PatchRepository _patches;

  static const MethodChannel _channel = MethodChannel(CrashConfigs.channel);

  @override
  Future<CrashContext> read() async {
    var version = '0.0.0';
    var build = '0';
    try {
      final info = await PackageInfo.fromPlatform();
      version = CrashScrubber.appVersion(info.version) ?? version;
      build = CrashScrubber.appBuild(info.buildNumber) ?? build;
    } on Object {
      // Unknown stays 0.0.0 (0).
    }
    final patch = CrashScrubber.patch(await _patches.currentPatch());
    final osVersion =
        CrashScrubber.osVersion(await _ask('osVersion')) ??
        CrashScrubber.osVersionIn(Platform.operatingSystemVersion) ??
        CrashScrubber.unknown;
    final device =
        CrashScrubber.device(await _ask('deviceModel')) ??
        CrashScrubber.unknown;
    return CrashContext(
      app: CrashApp(version: version, build: build, patch: patch),
      os: CrashOs(
        name: Platform.isAndroid ? CrashOsName.android : CrashOsName.ios,
        version: osVersion,
      ),
      device: device,
    );
  }

  Future<String?> _ask(String method) async {
    try {
      return await _channel.invokeMethod<String>(method);
    } on Object {
      return null;
    }
  }
}
