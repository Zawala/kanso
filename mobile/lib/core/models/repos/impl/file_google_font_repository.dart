// Kanso — file_google_font_repository.dart
// Created 2026-10-29.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:io';

import 'package:dartz/dartz.dart';
import 'package:flutter/services.dart';
import 'package:injectable/injectable.dart';
import 'package:path/path.dart' as p;

import '../../data/application_error.dart';
import '../../data/library_folder.dart';
import '../abs/google_font_repository.dart';
import 'google_font_source.dart';

/// Registers downloaded TrueType files with the running Flutter engine.
// ignore: one_member_abstracts
abstract interface class GoogleFontRegistrar {
  /// Makes [bytes] available under [family].
  Future<void> register(String family, List<int> bytes);
}

/// The Flutter engine font registrar.
@LazySingleton(as: GoogleFontRegistrar)
class FlutterGoogleFontRegistrar implements GoogleFontRegistrar {
  @override
  Future<void> register(String family, List<int> bytes) async {
    await (FontLoader(family)..addFont(
          Future<ByteData>.value(
            ByteData.sublistView(Uint8List.fromList(bytes)),
          ),
        ))
        .load();
  }
}

/// Stores downloaded fonts beside this device's private Kanso library.
@LazySingleton(as: GoogleFontRepository)
class FileGoogleFontRepository implements GoogleFontRepository {
  FileGoogleFontRepository(this._folder, this._source, this._registrar);

  final LibraryFolder _folder;
  final GoogleFontSource _source;
  final GoogleFontRegistrar _registrar;
  final Set<String> _registered = <String>{};

  @override
  Future<Either<ApplicationError, String>> load(String family) async {
    final name = family.trim();
    if (name.isEmpty ||
        name.length > 80 ||
        !RegExp(r'^[a-zA-Z0-9 _-]+$').hasMatch(name)) {
      return const Left<ApplicationError, String>(
        ApplicationError.invalid(message: 'Invalid Google Font family.'),
      );
    }
    final file = File(p.join(_folder.path, 'fonts', '${_cacheKey(name)}.ttf'));
    try {
      List<int>? bytes;
      if (file.existsSync()) {
        bytes = await file.readAsBytes();
        if (!isFlutterFontFile(bytes)) {
          // A cache written by an older build may hold WOFF or EOT. It is
          // not usable by the engine, so replace it instead of failing every
          // launch with the same bad file.
          await file.delete();
          bytes = null;
        }
      }
      if (bytes == null) {
        bytes = await _source.download(name);
        if (bytes.isEmpty) {
          return const Left<ApplicationError, String>(
            ApplicationError.network(
              message: 'Google Fonts returned an empty file.',
            ),
          );
        }
        if (!isFlutterFontFile(bytes)) {
          return const Left<ApplicationError, String>(
            ApplicationError.invalid(
              message: 'Google Fonts returned a format Flutter cannot load.',
            ),
          );
        }
        await file.parent.create(recursive: true);
        await File('${file.path}.tmp').writeAsBytes(bytes, flush: true);
        await File('${file.path}.tmp').rename(file.path);
      }
      if (_registered.add(name)) {
        await _registrar.register(name, bytes);
      }
      return Right<ApplicationError, String>(name);
    } on SocketException catch (error) {
      return Left<ApplicationError, String>(
        ApplicationError.network(message: error.message),
      );
    } on HttpException catch (error) {
      return Left<ApplicationError, String>(
        ApplicationError.invalid(message: error.message),
      );
    } on FormatException catch (error) {
      return Left<ApplicationError, String>(
        ApplicationError.invalid(message: error.message),
      );
    } on Object catch (error) {
      return Left<ApplicationError, String>(
        ApplicationError.storage(message: error.toString()),
      );
    }
  }

  String _cacheKey(String value) =>
      value.toLowerCase().replaceAll(RegExp('[^a-z0-9]+'), '-');
}
