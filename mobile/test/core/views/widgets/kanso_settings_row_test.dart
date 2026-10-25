// Kanso — kanso_settings_row_test.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/configs/kanso_icons.dart';
import 'package:kanso/core/views/widgets/kanso_icon.dart';
import 'package:kanso/core/views/widgets/kanso_settings_row.dart';

import '../../../helpers/pump.dart';

void main() {
  group('KansoSettingsRow', () {
    testWidgets('shows its label and value', (tester) async {
      await pumpKanso(
        tester,
        const KansoSettingsRow(label: 'Theme', value: 'Kanso Dark'),
      );
      expect(find.text('Theme'), findsOneWidget);
      expect(find.text('Kanso Dark'), findsOneWidget);
    });

    testWidgets('a chevron row draws a chevron, an external one a link mark', (
      tester,
    ) async {
      await pumpKanso(
        tester,
        const KansoSettingsRow(label: 'A', chevron: true),
      );
      expect(
        tester.widgetList<KansoIcon>(find.byType(KansoIcon)).single.icon,
        KansoIcons.chevronRight,
      );
      await pumpKanso(
        tester,
        const KansoSettingsRow(label: 'A', external: true),
      );
      expect(
        tester.widgetList<KansoIcon>(find.byType(KansoIcon)).single.icon,
        KansoIcons.externalLink,
      );
    });

    testWidgets('is at least 48 points tall', (tester) async {
      await pumpKanso(tester, const KansoSettingsRow(label: 'A'));
      expect(
        tester.getSize(find.byType(KansoSettingsRow)).height,
        greaterThanOrEqualTo(48),
      );
    });

    testWidgets('a long label wraps instead of overflowing', (tester) async {
      await pumpKanso(
        tester,
        const SizedBox(
          width: 320,
          child: KansoSettingsRow(
            icon: KansoIcons.bell,
            label: 'Wake this device when notes change elsewhere',
            value: 'On',
            chevron: true,
          ),
        ),
      );
      expect(tester.takeException(), isNull);
    });

    testWidgets('a tap calls onTap', (tester) async {
      var taps = 0;
      await pumpKanso(
        tester,
        KansoSettingsRow(label: 'A', onTap: () => taps++),
      );
      await tester.tap(find.text('A'));
      expect(taps, 1);
    });
  });
}
