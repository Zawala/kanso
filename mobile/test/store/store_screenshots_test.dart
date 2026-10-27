// Kanso — store_screenshots_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

@Tags(<String>['store-screenshots'])
library;

import 'dart:io';
import 'dart:ui' as ui;

import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:flutter/rendering.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/configs/kanso_colors.dart';

import '../helpers/browse_blocs.dart';
import '../helpers/fixture_library_repository.dart';
import '../helpers/library_fixtures.dart';
import '../helpers/pump_browse_app.dart';
import '../helpers/rgb_png.dart';
import '../helpers/store_sets.dart';

/// Renders the pictures for the store listings (mobile/store/README.md):
/// the app over the mock library, at the exact pixel size of each set, plus
/// Play's feature graphic and icon. They are written to the folder in
/// `KANSO_STORE_SHOTS_DIR` as 24-bit PNGs without alpha, and the test is
/// skipped when it is not set, so a plain `flutter test` writes nothing.
/// `scripts/mobile-store-render.sh` runs it and checks what it wrote.

final String? _outDir = Platform.environment['KANSO_STORE_SHOTS_DIR'];
final Object _skip = (_outDir ?? '').isEmpty
    ? 'Set KANSO_STORE_SHOTS_DIR to render the store pictures.'
    : false;

/// One picture of the story.
class _Shot {
  const _Shot(this.name, this.at, {this.dark = false, this.library = false});

  final String name;
  final String at;
  final bool dark;

  /// Opens the library drawer (where the library is not a pane).
  final bool library;
}

String _noteAt(String id) => '/notes/note/${Uri.encodeComponent(id)}';

final List<_Shot> _story = <_Shot>[
  const _Shot('01-notes', '/notes'),
  _Shot('02-note', _noteAt(MockIds.syncDesign)),
  const _Shot('03-library', '/notes', library: true),
  const _Shot('04-search', '/search?q=sync'),
  const _Shot('05-notes-dark', '/notes', dark: true),
  _Shot('06-note-dark', _noteAt(MockIds.syncDesign), dark: true),
];

/// The open note's text: Markdown that reads well as plain text, as the
/// editor shows it.
const String _noteBody = '''
# Goals

Every device keeps a full copy. A change is **encrypted on the device**, sent once and *merged* on the others.

## Open questions

- How long a device may stay offline
- What a conflict looks like to a person

## Next steps

- [x] Write down the wire format
- [ ] Try a conflict on three devices
- [ ] Decide how deleted notes are kept

> Calm software says less and remembers more.

See also [[Weekly review]]
''';

FixtureLibraryRepository _library() {
  final library = FixtureLibraryRepository();
  library.notes.addAll(<String, NoteView>{
    ...readableMockNotes(),
    MockIds.syncDesign: mockNote(
      body: _noteBody,
      tags: <String>[MockIds.tagIdea, MockIds.tagDraft],
      status: 'active',
    ),
  });
  return library;
}

void main() {
  for (final set in storeSets) {
    group(set.name, () {
      for (final shot in _story) {
        testWidgets(shot.name, skip: _skip != false, (tester) async {
          final file = File('$_outDir/${set.name}/${shot.name}.png')
            ..parent.createSync(recursive: true);
          tester.platformDispatcher
            ..textScaleFactorTestValue = 1
            ..platformBrightnessTestValue = shot.dark
                ? Brightness.dark
                : Brightness.light
            ..accessibilityFeaturesTestValue = const FakeAccessibilityFeatures(
              disableAnimations: true,
              reduceMotion: true,
            );
          addTearDown(() {
            tester.view.reset();
            tester.platformDispatcher
              ..clearTextScaleFactorTestValue()
              ..clearPlatformBrightnessTestValue()
              ..clearAccessibilityFeaturesTestValue();
          });
          debugDefaultTargetPlatformOverride = set.platform;
          debugDisableShadows = false;
          try {
            await atMockNow(() async {
              await pumpBrowseApp(
                tester,
                set.logical,
                at: shot.at,
                library: _library(),
                devicePixelRatio: set.devicePixelRatio,
                safeArea: set.safeArea,
                tabletDevice: set.tablet,
              );
              if (shot.library) {
                final label = find.bySemanticsLabel('Library');
                if (label.evaluate().isNotEmpty) {
                  await tester.tap(label.first);
                  await tester.pumpAndSettle();
                }
              }
            });
            final boundary = tester.renderObject<RenderRepaintBoundary>(
              find.byKey(appBoundaryKey),
            );
            final png = (await tester.runAsync(() async {
              final image = await boundary.toImage(
                pixelRatio: set.devicePixelRatio,
              );
              expect(image.width, set.pixels.width.round());
              expect(image.height, set.pixels.height.round());
              final data = await image.toByteData();
              final width = image.width;
              final height = image.height;
              image.dispose();
              return encodeRgbPng(width, height, data!.buffer.asUint8List());
            }))!;
            file.writeAsBytesSync(png);
            await tester.pumpWidget(const SizedBox.shrink());
          } finally {
            debugDisableShadows = true;
            debugDefaultTargetPlatformOverride = null;
          }
        });
      }
    });
  }

  testWidgets('play feature graphic', skip: _skip != false, (tester) async {
    tester.view
      ..devicePixelRatio = 1
      ..physicalSize = featureGraphicSize;
    addTearDown(tester.view.reset);
    final icon = await tester.runAsync(() async {
      final bytes = File(_iconPath).readAsBytesSync();
      final codec = await ui.instantiateImageCodec(bytes, targetWidth: 400);
      return (await codec.getNextFrame()).image;
    });
    const key = ValueKey<String>('feature-graphic');
    await tester.pumpWidget(
      RepaintBoundary(
        key: key,
        child: Directionality(
          textDirection: TextDirection.ltr,
          child: _FeatureGraphic(icon: icon!),
        ),
      ),
    );
    await tester.pumpAndSettle();
    final png = await _capture(tester, key, 1);
    File('$_outDir/play/feature-graphic.png')
      ..parent.createSync(recursive: true)
      ..writeAsBytesSync(png);
  });

  testWidgets('play icon', skip: _skip != false, (tester) async {
    tester.view
      ..devicePixelRatio = 1
      ..physicalSize = playIconSize;
    addTearDown(tester.view.reset);
    const key = ValueKey<String>('play-icon');
    await tester.pumpWidget(
      const RepaintBoundary(
        key: key,
        child: SizedBox.expand(child: CustomPaint(painter: _IconPainter())),
      ),
    );
    final png = await _capture(tester, key, 1);
    File('$_outDir/play/icon-512.png')
      ..parent.createSync(recursive: true)
      ..writeAsBytesSync(png);
  });
}

/// The enso mark of the icon on its paper colour, full bleed: Play rounds the
/// corners itself, so the picture must be a plain square.
class _IconPainter extends CustomPainter {
  const _IconPainter();

  @override
  void paint(Canvas canvas, Size size) {
    final rect = Offset.zero & size;
    canvas.drawRect(
      rect,
      Paint()
        ..shader = const LinearGradient(
          begin: Alignment.topCenter,
          end: Alignment.bottomCenter,
          colors: <Color>[Color(0xFFFCFAF5), Color(0xFFF1ECE1)],
        ).createShader(rect),
    );
    final scale = size.width / 512;
    const centre = Offset(250, 262);
    final radius = 113 * scale;
    canvas
      ..drawArc(
        Rect.fromCircle(center: centre * scale, radius: radius),
        -18 * 3.14159265 / 180,
        323 * 3.14159265 / 180,
        false,
        Paint()
          ..style = PaintingStyle.stroke
          ..strokeCap = StrokeCap.round
          ..strokeWidth = 30 * scale
          ..color = const Color(0xFF3A342D)
          ..isAntiAlias = true,
      )
      ..drawCircle(
        const Offset(349, 164) * scale,
        12 * scale,
        Paint()..color = const Color(0xFFD65A38),
      );
  }

  @override
  bool shouldRepaint(_IconPainter oldDelegate) => false;
}

/// Kanso's icon, as the website has it (the app's own iOS and Android icons
/// are still Flutter's placeholder).
const String _iconPath = '../site/public/icon-512.png';

Future<Uint8List> _capture(WidgetTester tester, Key key, double ratio) async {
  final boundary = tester.renderObject<RenderRepaintBoundary>(find.byKey(key));
  return (await tester.runAsync(() async {
    final image = await boundary.toImage(pixelRatio: ratio);
    final data = await image.toByteData();
    final out = encodeRgbPng(
      image.width,
      image.height,
      data!.buffer.asUint8List(),
    );
    image.dispose();
    return out;
  }))!;
}

/// Play's feature graphic: the icon, the name and the one-line promise on
/// Kanso's paper colour, with two note cards.
class _FeatureGraphic extends StatelessWidget {
  const _FeatureGraphic({required this.icon});

  final ui.Image icon;

  static const String _tagline =
      'Private Markdown notes that stay on your phone, and sync '
      'end-to-end encrypted.';

  @override
  Widget build(BuildContext context) {
    const colors = KansoColors.light;
    return Container(
      width: 1024,
      height: 500,
      color: colors.app,
      padding: const EdgeInsets.symmetric(horizontal: 72),
      child: Row(
        children: <Widget>[
          Expanded(
            flex: 11,
            child: Column(
              mainAxisAlignment: MainAxisAlignment.center,
              crossAxisAlignment: CrossAxisAlignment.start,
              children: <Widget>[
                ClipRRect(
                  borderRadius: BorderRadius.circular(22),
                  child: SizedBox(
                    width: 96,
                    height: 96,
                    child: RawImage(image: icon, fit: BoxFit.cover),
                  ),
                ),
                const SizedBox(height: 28),
                DefaultTextStyle(
                  style: TextStyle(
                    fontFamily: 'Instrument Sans',
                    color: colors.text,
                    decoration: TextDecoration.none,
                  ),
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: <Widget>[
                      const Text(
                        'Kanso',
                        style: TextStyle(
                          fontSize: 84,
                          fontWeight: FontWeight.w600,
                          height: 1,
                        ),
                      ),
                      const SizedBox(height: 18),
                      Text(
                        _tagline,
                        style: TextStyle(
                          fontSize: 25,
                          height: 1.4,
                          color: colors.textSecondary,
                        ),
                      ),
                    ],
                  ),
                ),
              ],
            ),
          ),
          const SizedBox(width: 48),
          Expanded(
            flex: 9,
            child: Column(
              mainAxisAlignment: MainAxisAlignment.center,
              children: <Widget>[
                _card(colors, 'Sync design notes', mockRows[1].snippet, 0),
                const SizedBox(height: 18),
                _card(colors, 'Sourdough starter', mockRows[2].snippet, 28),
              ],
            ),
          ),
        ],
      ),
    );
  }

  Widget _card(KansoColors colors, String title, String body, double indent) {
    return Padding(
      padding: EdgeInsets.only(left: indent),
      child: Container(
        padding: const EdgeInsets.all(22),
        decoration: BoxDecoration(
          color: colors.elevated,
          borderRadius: BorderRadius.circular(18),
          border: Border.all(color: colors.divider),
          boxShadow: const <BoxShadow>[
            BoxShadow(
              color: Color(0x1426241F),
              blurRadius: 24,
              offset: Offset(0, 8),
            ),
          ],
        ),
        child: DefaultTextStyle(
          style: TextStyle(
            fontFamily: 'Instrument Sans',
            color: colors.text,
            decoration: TextDecoration.none,
          ),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: <Widget>[
              Text(
                title,
                style: const TextStyle(
                  fontSize: 22,
                  fontWeight: FontWeight.w600,
                ),
              ),
              const SizedBox(height: 8),
              Text(
                body,
                maxLines: 3,
                overflow: TextOverflow.ellipsis,
                style: TextStyle(
                  fontSize: 17,
                  height: 1.4,
                  color: colors.textSecondary,
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }
}
