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

class FakeGoogleFontSource implements GoogleFontSource {
  int downloads = 0;
  List<int> bytes = <int>[1, 2, 3, 4];
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
