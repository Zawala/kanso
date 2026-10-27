// Kanso — store_sets.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/foundation.dart';
import 'package:flutter/painting.dart';

/// A set of pictures for a store listing: a device size and the platform the
/// store is for. `mobile/store/README.md` documents the same table.
class StoreSet {
  const StoreSet({
    required this.name,
    required this.pixels,
    required this.devicePixelRatio,
    required this.platform,
    required this.safeArea,
  });

  /// The folder the pictures are written to.
  final String name;

  /// The picture's size in physical pixels, portrait.
  final Size pixels;

  final double devicePixelRatio;

  /// The platform the app draws itself for: iOS for the App Store.
  final TargetPlatform platform;

  /// The status bar and home indicator, in logical pixels.
  final EdgeInsets safeArea;

  /// The window the app is laid out in, in logical pixels.
  Size get logical => pixels / devicePixelRatio;

  bool get tablet => logical.shortestSide >= 600;
}

const EdgeInsets _iPhoneSafeArea = EdgeInsets.only(top: 62, bottom: 34);
const EdgeInsets _iPadSafeArea = EdgeInsets.only(top: 24, bottom: 20);
const EdgeInsets _androidSafeArea = EdgeInsets.only(top: 28, bottom: 24);

/// The sets the store listings need.
const List<StoreSet> storeSets = <StoreSet>[
  StoreSet(
    name: 'ios-iphone-6.9',
    pixels: Size(1320, 2868),
    devicePixelRatio: 3,
    platform: TargetPlatform.iOS,
    safeArea: _iPhoneSafeArea,
  ),
  StoreSet(
    name: 'ios-iphone-6.5',
    pixels: Size(1284, 2778),
    devicePixelRatio: 3,
    platform: TargetPlatform.iOS,
    safeArea: EdgeInsets.only(top: 47, bottom: 34),
  ),
  StoreSet(
    name: 'ios-ipad-13',
    pixels: Size(2064, 2752),
    devicePixelRatio: 2,
    platform: TargetPlatform.iOS,
    safeArea: _iPadSafeArea,
  ),
  StoreSet(
    name: 'android-phone',
    pixels: Size(1080, 2160),
    devicePixelRatio: 3,
    platform: TargetPlatform.android,
    safeArea: _androidSafeArea,
  ),
  StoreSet(
    name: 'android-tablet-7',
    pixels: Size(1200, 1920),
    devicePixelRatio: 2,
    platform: TargetPlatform.android,
    safeArea: _androidSafeArea,
  ),
  StoreSet(
    name: 'android-tablet-10',
    pixels: Size(1600, 2560),
    devicePixelRatio: 2,
    platform: TargetPlatform.android,
    safeArea: _androidSafeArea,
  ),
];

/// The Play feature graphic's size.
const Size featureGraphicSize = Size(1024, 500);

/// The Play icon's size.
const Size playIconSize = Size(512, 512);
