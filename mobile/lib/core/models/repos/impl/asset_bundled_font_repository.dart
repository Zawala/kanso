// Kanso — asset_bundled_font_repository.dart
// Created 2026-10-30.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:flutter/services.dart';
import 'package:injectable/injectable.dart';

import '../abs/bundled_font_repository.dart';

/// The upright files of each bundled family, as `pubspec.yaml` declares
/// them, with the weight each covers. Italics are left out: the page is
/// rebuilt often and a variable italic doubles its size, so the web view
/// slants the upright face instead.
const Map<String, List<(String, String)>> bundledFontAssets =
    <String, List<(String, String)>>{
      'Instrument Sans': <(String, String)>[
        ('assets/fonts/InstrumentSans-Regular.ttf', '400'),
        ('assets/fonts/InstrumentSans-Medium.ttf', '500'),
        ('assets/fonts/InstrumentSans-SemiBold.ttf', '600'),
        ('assets/fonts/InstrumentSans-Bold.ttf', '700'),
      ],
      'Inter': <(String, String)>[
        ('assets/fonts/Inter-Variable.ttf', '100 900'),
      ],
      'Source Serif 4': <(String, String)>[
        ('assets/fonts/SourceSerif4-Variable.ttf', '200 900'),
      ],
      'Literata': <(String, String)>[
        ('assets/fonts/Literata-Variable.ttf', '200 900'),
      ],
      'JetBrains Mono': <(String, String)>[
        ('assets/fonts/JetBrainsMono-Regular.ttf', '400'),
        ('assets/fonts/JetBrainsMono-Bold.ttf', '700'),
      ],
      'IBM Plex Mono': <(String, String)>[
        ('assets/fonts/IBMPlexMono-Regular.ttf', '400'),
        ('assets/fonts/IBMPlexMono-Medium.ttf', '500'),
        ('assets/fonts/IBMPlexMono-SemiBold.ttf', '600'),
        ('assets/fonts/IBMPlexMono-Bold.ttf', '700'),
      ],
    };

/// Reads the bundled fonts from the app's assets, once per family.
@LazySingleton(as: BundledFontRepository)
class AssetBundledFontRepository implements BundledFontRepository {
  AssetBundledFontRepository() : _bundle = rootBundle;

  /// A repository reading from another bundle, for the tests.
  AssetBundledFontRepository.from(this._bundle);

  final AssetBundle _bundle;
  final Map<String, Future<List<BundledFontFace>>> _read =
      <String, Future<List<BundledFontFace>>>{};

  @override
  Future<List<BundledFontFace>> faces(String family) {
    final assets = bundledFontAssets[family];
    if (assets == null) {
      return Future<List<BundledFontFace>>.value(const <BundledFontFace>[]);
    }
    return _read.putIfAbsent(family, () async {
      try {
        return <BundledFontFace>[
          for (final (asset, weight) in assets)
            BundledFontFace(
              weight: weight,
              bytes: (await _bundle.load(asset)).buffer.asUint8List(),
            ),
        ];
      } on Object {
        // Read again next time rather than keep the failure.
        unawaited(_read.remove(family));
        return const <BundledFontFace>[];
      }
    });
  }
}
