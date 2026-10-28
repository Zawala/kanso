// Kanso — google_font_source.dart
// Created 2026-10-29.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:convert';
import 'dart:io';

import 'package:injectable/injectable.dart';

/// Fetches the CSS and font asset for a Google Fonts family.
// ignore: one_member_abstracts
abstract interface class GoogleFontSource {
  /// Downloads a TrueType font file for [family].
  Future<List<int>> download(String family);
}

/// Parses the font asset URL from a Google Fonts stylesheet.
// ignore: one_member_abstracts
abstract interface class GoogleFontStylesheetParser {
  /// The downloadable font asset, or null when the stylesheet has none.
  Uri? fontUrl(String css);
}

/// Finds a TrueType URL in Google Fonts CSS.
@LazySingleton(as: GoogleFontStylesheetParser)
class GoogleFontCssParser implements GoogleFontStylesheetParser {
  /// Parses the first font URL from [css].
  @override
  Uri? fontUrl(String css) {
    final match = RegExp(
      r'''src:\s*url\((?:'|")?(https://[^)'"\s]+)(?:'|")?\)''',
      caseSensitive: false,
    ).firstMatch(css);
    final value = match?.group(1);
    return value == null ? null : Uri.tryParse(value);
  }
}

/// The production Google Fonts CSS and file downloader.
@LazySingleton(as: GoogleFontSource)
class GoogleFontHttpSource implements GoogleFontSource {
  GoogleFontHttpSource(this.parser);

  final GoogleFontStylesheetParser parser;

  @override
  Future<List<int>> download(String family) async {
    final client = HttpClient()
      ..connectionTimeout = const Duration(seconds: 15);
    try {
      final cssUri = Uri.https('fonts.googleapis.com', '/css', <String, String>{
        'family': '$family:wght@400',
        'display': 'swap',
      });
      final cssRequest = await client.getUrl(cssUri);
      // Do not impersonate a browser: Google Fonts serves WOFF or EOT to
      // browser user agents, while its fallback for an unknown client is the
      // TrueType file Flutter's FontLoader accepts.
      final cssResponse = await cssRequest.close();
      if (cssResponse.statusCode != HttpStatus.ok) {
        throw HttpException('Google Fonts returned ${cssResponse.statusCode}.');
      }
      final css = utf8.decode(
        await cssResponse.fold<List<int>>(
          <int>[],
          (bytes, chunk) => bytes..addAll(chunk),
        ),
      );
      final fontUri = parser.fontUrl(css);
      if (fontUri == null || !fontUri.host.endsWith('gstatic.com')) {
        throw const FormatException('Google Fonts returned no font file.');
      }
      final fontRequest = await client.getUrl(fontUri);
      final fontResponse = await fontRequest.close();
      if (fontResponse.statusCode != HttpStatus.ok) {
        throw HttpException(
          'Google Fonts returned ${fontResponse.statusCode}.',
        );
      }
      final bytes = await fontResponse.fold<List<int>>(
        <int>[],
        (bytes, chunk) => bytes..addAll(chunk),
      );
      if (!isFlutterFontFile(bytes)) {
        throw const FormatException(
          'Google Fonts returned a format Flutter cannot load.',
        );
      }
      return bytes;
    } finally {
      client.close(force: true);
    }
  }
}

/// Whether [bytes] are a font format Flutter's font loader can register.
///
/// Google Fonts can serve WOFF, WOFF2 or EOT depending on the request's user
/// agent. Those are web formats; Flutter needs a TrueType or OpenType file.
bool isFlutterFontFile(List<int> bytes) {
  if (bytes.length < 12) {
    return false;
  }
  final signature = String.fromCharCodes(bytes.take(4));
  return signature == '\x00\x01\x00\x00' ||
      signature == 'OTTO' ||
      signature == 'true' ||
      signature == 'typ1' ||
      signature == 'ttcf';
}
