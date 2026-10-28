// Kanso — google_font_repository_test.dart
// Created 2026-10-29.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:io';

import 'package:dartz/dartz.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/core/models/data/library_folder.dart';
import 'package:kanso/core/models/repos/impl/file_google_font_repository.dart';
import 'package:kanso/core/models/repos/impl/google_font_source.dart';

final List<int> _trueType = <int>[0, 1, 0, 0, ...List<int>.filled(32, 7)];

class FakeGoogleFontSource implements GoogleFontSource {
  int downloads = 0;
  List<int> bytes = _trueType;
  Exception? failure;

  @override
  Future<List<int>> download(String family) async {
    downloads++;
    final error = failure;
    if (error != null) {
      throw error;
    }
    return bytes;
  }
}

class FakeGoogleFontRegistrar implements GoogleFontRegistrar {
  final List<String> families = <String>[];

  @override
  Future<void> register(String family, List<int> bytes) async {
    families.add(family);
  }
}

void main() {
  group('GoogleFontCssParser', () {
    test('reads the downloadable URL from Google Fonts CSS', () {
      const css = '''
        @font-face {
          font-family: 'Lora';
          src: url(https://fonts.gstatic.com/s/lora/v1/lora.ttf) format('truetype');
        }
      ''';
      expect(
        GoogleFontCssParser().fontUrl(css),
        Uri.parse('https://fonts.gstatic.com/s/lora/v1/lora.ttf'),
      );
    });

    test('reports a missing font asset in malformed CSS', () {
      expect(
        GoogleFontCssParser().fontUrl('font-face { src: local(); }'),
        isNull,
      );
    });
  });

  group('FileGoogleFontRepository', () {
    late Directory folder;
    late FakeGoogleFontSource source;
    late FakeGoogleFontRegistrar registrar;
    late FileGoogleFontRepository repository;

    setUp(() async {
      folder = await Directory.systemTemp.createTemp('kanso_google_font_');
      source = FakeGoogleFontSource();
      registrar = FakeGoogleFontRegistrar();
      repository = FileGoogleFontRepository(
        LibraryFolder(folder.path),
        source,
        registrar,
      );
    });

    tearDown(() async {
      await folder.delete(recursive: true);
    });

    test('downloads once, caches the font, then loads it offline', () async {
      expect(
        await repository.load('Lora'),
        const Right<ApplicationError, String>('Lora'),
      );
      expect(
        await repository.load('Lora'),
        const Right<ApplicationError, String>('Lora'),
      );
      expect(source.downloads, 1);
      expect(registrar.families, <String>['Lora']);
      expect(
        await File('${folder.path}/fonts/lora.ttf').readAsBytes(),
        source.bytes,
      );
    });

    test('rejects a web-only font and leaves no cache', () async {
      source.bytes = <int>[0x77, 0x4f, 0x46, 0x32, ...List<int>.filled(32, 1)];
      final result = await repository.load('Lora');
      expect(result.isLeft(), isTrue);
      expect(
        result.fold((error) => error is InvalidError, (_) => false),
        isTrue,
      );
      expect(registrar.families, isEmpty);
      expect(
        File('${folder.path}/fonts/lora.ttf').existsSync(),
        isFalse,
      );
    });

    test('replaces an unusable cached file with a fresh download', () async {
      final cache = File('${folder.path}/fonts/lora.ttf');
      await cache.parent.create(recursive: true);
      await cache.writeAsBytes(<int>[
        0x77,
        0x4f,
        0x46,
        0x32,
        ...List<int>.filled(32, 1),
      ]);

      expect(
        await repository.load('Lora'),
        const Right<ApplicationError, String>('Lora'),
      );
      expect(source.downloads, 1);
      expect(await cache.readAsBytes(), _trueType);
    });

    test('returns a network failure without claiming a cached font', () async {
      source.failure = const SocketException('offline');
      final result = await repository.load('Lora');
      expect(result.isLeft(), isTrue);
      expect(
        result.fold((error) => error is NetworkError, (_) => false),
        isTrue,
      );
      expect(registrar.families, isEmpty);
      expect(Directory('${folder.path}/fonts').existsSync(), isFalse);
    });
  });
}
