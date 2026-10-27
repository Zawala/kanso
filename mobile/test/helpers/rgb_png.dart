// Kanso — rgb_png.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:convert';
import 'dart:io';
import 'dart:typed_data';

/// The eight bytes every PNG starts with.
const List<int> pngSignature = <int>[137, 80, 78, 71, 13, 10, 26, 10];

/// The colour type of a PNG with red, green and blue and no alpha.
const int pngColorTypeRgb = 2;

/// Encodes [rgba] (four bytes a pixel, row by row) as a 24-bit PNG with no
/// alpha channel, which Google Play requires. Each pixel is composited onto
/// opaque white first, so a picture that is already opaque is unchanged.
Uint8List encodeRgbPng(int width, int height, Uint8List rgba) {
  assert(rgba.length == width * height * 4, 'Four bytes a pixel.');
  final raw = Uint8List(height * (1 + width * 3));
  var out = 0;
  var src = 0;
  for (var y = 0; y < height; y++) {
    raw[out++] = 0; // filter: none
    for (var x = 0; x < width; x++) {
      final a = rgba[src + 3];
      for (var c = 0; c < 3; c++) {
        final v = rgba[src + c];
        raw[out++] = a == 255 ? v : ((v * a + 255 * (255 - a)) / 255).round();
      }
      src += 4;
    }
  }
  final header = ByteData(13)
    ..setUint32(0, width)
    ..setUint32(4, height)
    ..setUint8(8, 8)
    ..setUint8(9, pngColorTypeRgb);
  final bytes = BytesBuilder(copy: false)
    ..add(pngSignature)
    ..add(_chunk('IHDR', header.buffer.asUint8List()))
    ..add(_chunk('IDAT', Uint8List.fromList(ZLibCodec().encode(raw))))
    ..add(_chunk('IEND', Uint8List(0)));
  return bytes.toBytes();
}

/// What the header of a PNG says.
class PngHeader {
  const PngHeader(this.width, this.height, this.bitDepth, this.colorType);

  final int width;
  final int height;
  final int bitDepth;
  final int colorType;

  /// Whether the picture can hold transparency: an alpha channel (types 4
  /// and 6) or a palette (3), which may carry a `tRNS` chunk.
  bool get hasAlpha => colorType == 4 || colorType == 6;
}

/// Reads the header of [png], or throws a [FormatException].
PngHeader readPngHeader(Uint8List png) {
  if (png.length < 33) {
    throw const FormatException('Too short for a PNG.');
  }
  for (var i = 0; i < 8; i++) {
    if (png[i] != pngSignature[i]) {
      throw const FormatException('Not a PNG.');
    }
  }
  final data = ByteData.sublistView(png);
  return PngHeader(
    data.getUint32(16),
    data.getUint32(20),
    png[24],
    png[25],
  );
}

/// Decodes a PNG written by [encodeRgbPng] (8-bit RGB, any filters) to its
/// RGB bytes, three a pixel.
Uint8List decodeRgbPng(Uint8List png) {
  final header = readPngHeader(png);
  if (header.colorType != pngColorTypeRgb || header.bitDepth != 8) {
    throw const FormatException('Only 8-bit RGB is read.');
  }
  final data = ByteData.sublistView(png);
  final idat = BytesBuilder(copy: false);
  var at = 8;
  while (at + 8 <= png.length) {
    final length = data.getUint32(at);
    final type = ascii.decode(png.sublist(at + 4, at + 8));
    if (type == 'IDAT') {
      idat.add(png.sublist(at + 8, at + 8 + length));
    }
    at += 12 + length;
  }
  final raw = ZLibCodec().decode(idat.toBytes());
  final stride = header.width * 3;
  final pixels = Uint8List(header.height * stride);
  for (var y = 0; y < header.height; y++) {
    final filter = raw[y * (stride + 1)];
    for (var i = 0; i < stride; i++) {
      final x = raw[y * (stride + 1) + 1 + i];
      final left = i >= 3 ? pixels[y * stride + i - 3] : 0;
      final up = y > 0 ? pixels[(y - 1) * stride + i] : 0;
      final upLeft = y > 0 && i >= 3 ? pixels[(y - 1) * stride + i - 3] : 0;
      pixels[y * stride + i] =
          (x +
              switch (filter) {
                0 => 0,
                1 => left,
                2 => up,
                3 => (left + up) ~/ 2,
                _ => _paeth(left, up, upLeft),
              }) &
          255;
    }
  }
  return pixels;
}

int _paeth(int a, int b, int c) {
  final p = a + b - c;
  final pa = (p - a).abs();
  final pb = (p - b).abs();
  final pc = (p - c).abs();
  if (pa <= pb && pa <= pc) {
    return a;
  }
  return pb <= pc ? b : c;
}

Uint8List _chunk(String type, Uint8List data) {
  final typed = Uint8List.fromList(<int>[...ascii.encode(type), ...data]);
  final out = ByteData(12 + data.length)..setUint32(0, data.length);
  final bytes = out.buffer.asUint8List()..setRange(4, 8 + data.length, typed);
  out.setUint32(8 + data.length, _crc32(typed));
  return bytes;
}

final List<int> _crcTable = List<int>.generate(256, (n) {
  var c = n;
  for (var k = 0; k < 8; k++) {
    c = (c & 1) != 0 ? 0xEDB88320 ^ (c >> 1) : c >> 1;
  }
  return c;
});

int _crc32(List<int> bytes) {
  var c = 0xFFFFFFFF;
  for (final b in bytes) {
    c = _crcTable[(c ^ b) & 255] ^ (c >> 8);
  }
  return c ^ 0xFFFFFFFF;
}
