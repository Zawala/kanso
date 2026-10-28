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
      cssRequest.headers.set(
        HttpHeaders.userAgentHeader,
        // Google Fonts serves TrueType to this Android WebView generation.
        'Mozilla/5.0 (Linux; Android 4.4.4) AppleWebKit/537.36 '
        '(KHTML, like Gecko) Chrome/36.0.1985.135 Mobile Safari/537.36',
      );
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
      return await fontResponse.fold<List<int>>(
        <int>[],
        (bytes, chunk) => bytes..addAll(chunk),
      );
    } finally {
      client.close(force: true);
    }
  }
}
