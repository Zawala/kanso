// Kanso — crash_context_repository_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/models/repos/impl/shorebird_patch_repository.dart';
import 'package:kanso/crash/configs/crash_configs.dart';
import 'package:kanso/crash/models/data/crash_os_name.dart';
import 'package:kanso/crash/models/repos/impl/platform_crash_context_repository.dart';
import 'package:package_info_plus/package_info_plus.dart';

import '../../helpers/crash_samples.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();
  const channel = MethodChannel(CrashConfigs.channel);

  tearDown(() {
    TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
        .setMockMethodCallHandler(channel, null);
  });

  void answer(Future<Object?> Function(MethodCall call)? handler) {
    TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
        .setMockMethodCallHandler(channel, handler);
  }

  void packageInfo(String version, String build) {
    PackageInfo.setMockInitialValues(
      appName: 'Kanso',
      packageName: 'app.kanso.kanso',
      version: version,
      buildNumber: build,
      buildSignature: '',
    );
  }

  group('PlatformCrashContextRepository', () {
    test('reads the build, the patch, the OS and the device', () async {
      packageInfo('0.9.0', '57');
      answer(
        (call) async => switch (call.method) {
          'deviceModel' => 'iPhone17,1',
          'osVersion' => '18.1',
          _ => null,
        },
      );
      final context = await PlatformCrashContextRepository(
        FixturePatchRepository(2),
      ).read();
      expect(context.app.version, '0.9.0');
      expect(context.app.build, '57');
      expect(context.app.patch, 2);
      expect(context.os.version, '18.1');
      expect(context.os.name, isIn(CrashOsName.values));
      expect(context.device, 'iPhone17,1');
    });

    test('scrubs what the platform says', () async {
      packageInfo('1.0.0-beta+3', 'abc');
      answer((call) async => "Ngoni's phone with a note");
      final context = await PlatformCrashContextRepository(
        FixturePatchRepository(null),
      ).read();
      expect(context.app.version, '0.0.0');
      expect(context.app.build, '0');
      expect(context.app.patch, isNull);
      expect(context.device, 'unknown');
      expect(context.os.version, isNot(contains(' ')));
    });

    test('without the native side the device is unknown', () async {
      packageInfo('0.9.0', '57');
      answer(null);
      final context = await PlatformCrashContextRepository(
        FixturePatchRepository(null),
      ).read();
      expect(context.device, 'unknown');
    });
  });

  group('ShorebirdPatchRepository', () {
    test('without Shorebird there is no patch', () async {
      expect(await ShorebirdPatchRepository().currentPatch(), isNull);
    });
  });
}
