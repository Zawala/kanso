// Kanso — kanso_status_glyph_test.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/configs/kanso_colors.dart';
import 'package:kanso/core/configs/kanso_icons.dart';
import 'package:kanso/core/models/data/note_status.dart';
import 'package:kanso/core/views/widgets/kanso_icon.dart';
import 'package:kanso/core/views/widgets/kanso_status_glyph.dart';

import '../../../helpers/pump.dart';

void main() {
  group('KansoStatusGlyph', () {
    test('picks the icon the desktop app uses for each status', () {
      expect(KansoStatusGlyph.iconFor(NoteStatus.active), KansoIcons.circleDot);
      expect(
        KansoStatusGlyph.iconFor(NoteStatus.onHold),
        KansoIcons.circlePause,
      );
      expect(
        KansoStatusGlyph.iconFor(NoteStatus.completed),
        KansoIcons.circleCheck,
      );
      expect(KansoStatusGlyph.iconFor(NoteStatus.dropped), KansoIcons.circleX);
      expect(KansoStatusGlyph.iconFor(NoteStatus.none), isNull);
    });

    test('picks accent, warning, success and quiet', () {
      const colors = KansoColors.dark;
      expect(
        KansoStatusGlyph.colorFor(NoteStatus.active, colors),
        colors.accent,
      );
      expect(
        KansoStatusGlyph.colorFor(NoteStatus.onHold, colors),
        colors.warning,
      );
      expect(
        KansoStatusGlyph.colorFor(NoteStatus.completed, colors),
        colors.success,
      );
      expect(
        KansoStatusGlyph.colorFor(NoteStatus.dropped, colors),
        colors.textMuted,
      );
    });

    test('no two statuses share a glyph', () {
      final icons = <KansoIcons>{
        for (final status in NoteStatus.values)
          ?KansoStatusGlyph.iconFor(status),
      };
      expect(icons.length, NoteStatus.values.length - 1);
    });

    testWidgets('draws an icon, or nothing for no status', (tester) async {
      await pumpKanso(
        tester,
        const KansoStatusGlyph(status: NoteStatus.completed),
      );
      final icon = tester.widget<KansoIcon>(find.byType(KansoIcon));
      expect(icon.icon, KansoIcons.circleCheck);
      expect(icon.color, KansoColors.light.success);
      await pumpKanso(tester, const KansoStatusGlyph(status: NoteStatus.none));
      expect(find.byType(KansoIcon), findsNothing);
    });
  });
}
