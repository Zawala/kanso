// Kanso — kanso_viewport.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/painting.dart';
import 'package:kanso/core/models/data/layout_class.dart';

/// A window the app is tested in: the size the app gets on a real device (or
/// a window of one), its density and its safe area, and the panes it must
/// have room for.
///
/// Sizes are in logical pixels (points on iOS, dp on Android) as the device
/// reports them to the app:
///
/// * iPhone 17: 393 x 852, the size class of every current iPhone but the
///   Plus and Max (Apple, "Layout" in the Human Interface Guidelines, device
///   screen dimensions).
/// * iPad (10th/11th generation) and iPad Air 11-inch: 820 x 1180; iPad Pro
///   11-inch: 834 x 1210 (reported as 834 x 1194 in landscape); iPad Pro
///   13-inch (M4, M5): 1032 x 1376 (same HIG table). The 11-inch iPad is
///   the matrix's iPad; the 13-inch one is its own entry because it is the
///   widest iPad in portrait.
/// * Pixel Tablet: 2560 x 1600 pixels at a density of 2 (xhdpi), so
///   1280 x 800 dp in landscape (Android developer documentation, "Pixel
///   Tablet" hardware profile; the same as the `pixel_tablet` AVD profile).
/// * Split View and Slide Over on an 11-inch iPad in landscape (1180 wide):
///   the app gets about a third (375), a half (590) or two thirds (800) of
///   the width, less the 10 pt divider; Slide Over is the compact width of
///   an iPhone, 375. The height stays the screen's (Apple, "Multitasking"
///   in the HIG; iPadOS reports these as the window's size, not the
///   screen's).
///
/// Whether the widths where one layout becomes another (740 and 1020) are
/// right is `kanso_viewport_test.dart`'s question: it checks each entry
/// against `KansoLayout.classFor`.
enum KansoViewport {
  /// An iPhone in portrait.
  phone(
    slug: 'phone',
    size: Size(393, 852),
    devicePixelRatio: 3,
    safeArea: EdgeInsets.only(top: 59, bottom: 34),
    layout: LayoutClass.single,
  ),

  /// An 11-inch iPad in portrait: the list beside the note.
  iPadPortrait(
    slug: 'ipad_portrait',
    size: Size(820, 1180),
    devicePixelRatio: 2,
    safeArea: EdgeInsets.only(top: 24, bottom: 20),
    layout: LayoutClass.double,
  ),

  /// An 11-inch iPad in landscape: the library, the list and the note.
  iPadLandscape(
    slug: 'ipad_landscape',
    size: Size(1180, 820),
    devicePixelRatio: 2,
    safeArea: EdgeInsets.only(top: 24, bottom: 20),
    layout: LayoutClass.triple,
  ),

  /// A 13-inch iPad in portrait: wide enough for three panes.
  iPad13Portrait(
    slug: 'ipad13_portrait',
    size: Size(1032, 1376),
    devicePixelRatio: 2,
    safeArea: EdgeInsets.only(top: 24, bottom: 20),
    layout: LayoutClass.triple,
  ),

  /// A Pixel Tablet in landscape.
  androidTablet(
    slug: 'android_tablet',
    size: Size(1280, 800),
    devicePixelRatio: 2,
    safeArea: EdgeInsets.only(top: 24, bottom: 24),
    layout: LayoutClass.triple,
  ),

  /// A Pixel Tablet in portrait.
  androidTabletPortrait(
    slug: 'android_tablet_portrait',
    size: Size(800, 1280),
    devicePixelRatio: 2,
    safeArea: EdgeInsets.only(top: 24, bottom: 24),
    layout: LayoutClass.double,
  ),

  /// An 11-inch iPad in landscape, the app in the left two thirds of a Split
  /// View: the list beside the note.
  splitViewTwoThirds(
    slug: 'split_two_thirds',
    size: Size(800, 820),
    devicePixelRatio: 2,
    safeArea: EdgeInsets.only(top: 24, bottom: 20),
    layout: LayoutClass.double,
  ),

  /// The app in half of a Split View: one pane, though the device is a
  /// tablet.
  splitViewHalf(
    slug: 'split_half',
    size: Size(590, 820),
    devicePixelRatio: 2,
    safeArea: EdgeInsets.only(top: 24, bottom: 20),
    layout: LayoutClass.single,
  ),

  /// The app in Slide Over: the width of an iPhone.
  slideOver(
    slug: 'slide_over',
    size: Size(375, 820),
    devicePixelRatio: 2,
    safeArea: EdgeInsets.only(top: 24, bottom: 20),
    layout: LayoutClass.single,
  );

  const KansoViewport({
    required this.slug,
    required this.size,
    required this.devicePixelRatio,
    required this.safeArea,
    required this.layout,
  });

  /// The viewport's name in the names of its golden files.
  final String slug;

  /// The window's size, in logical pixels.
  final Size size;

  /// Physical pixels per logical pixel.
  final double devicePixelRatio;

  /// The status bar and home indicator (or navigation bar), in logical
  /// pixels.
  final EdgeInsets safeArea;

  /// The panes this window has room for.
  final LayoutClass layout;

  /// The size on the screen of the device, in physical pixels.
  Size get physicalSize => size * devicePixelRatio;

  /// Whether the device is a tablet, whatever the width of its window.
  bool get tablet => this != phone;

  /// The viewports the screens' goldens are made in: a phone and the three
  /// tablets the issue asks for (an iPad in both orientations and an
  /// Android tablet).
  static const List<KansoViewport> goldenSet = <KansoViewport>[
    iPadPortrait,
    iPadLandscape,
    androidTablet,
  ];
}
