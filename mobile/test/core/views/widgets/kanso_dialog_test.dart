// Kanso — kanso_dialog_test.dart
// Created 2026-10-29.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/views/widgets/kanso_dialog.dart';
import 'package:kanso/core/views/widgets/kanso_text_input.dart';

import '../../../helpers/pump.dart';

Future<void> _openDialog(
  WidgetTester tester, {
  required Future<void> Function(BuildContext context) open,
}) async {
  await pumpKanso(
    tester,
    Builder(
      builder: (context) => TextButton(
        onPressed: () => open(context),
        child: const Text('Open dialog'),
      ),
    ),
  );
  await tester.tap(find.text('Open dialog'));
  await tester.pumpAndSettle();
}

void main() {
  group('Kanso dialog', () {
    testWidgets('confirm returns true only on the confirm action', (
      tester,
    ) async {
      bool? answer;
      await _openDialog(
        tester,
        open: (context) async {
          answer = await showKansoConfirm(
            context,
            title: 'Delete backup?',
            body: 'This cannot be undone.',
            cancelLabel: 'Keep',
            confirmLabel: 'Delete',
            danger: true,
          );
        },
      );

      expect(find.text('Delete backup?'), findsOneWidget);
      await tester.tap(find.text('Delete'));
      await tester.pumpAndSettle();
      expect(answer, isTrue);
      expect(find.text('Delete backup?'), findsNothing);
    });

    testWidgets('cancel returns false and leaves the dialog closed', (
      tester,
    ) async {
      bool? answer;
      await _openDialog(
        tester,
        open: (context) async {
          answer = await showKansoConfirm(
            context,
            title: 'Discard draft?',
            body: 'You will lose the changes.',
            cancelLabel: 'Keep',
            confirmLabel: 'Discard',
          );
        },
      );

      await tester.tap(find.text('Keep'));
      await tester.pumpAndSettle();
      expect(answer, isFalse);
      expect(find.text('Discard draft?'), findsNothing);
    });

    testWidgets('dismisses when the veil is tapped', (tester) async {
      bool? answer;
      await _openDialog(
        tester,
        open: (context) async {
          answer = await showKansoConfirm(
            context,
            title: 'Sign out?',
            body: 'You will need your password to return.',
            cancelLabel: 'Stay',
            confirmLabel: 'Sign out',
          );
        },
      );

      await tester.tapAt(const Offset(8, 8));
      await tester.pumpAndSettle();
      expect(answer, isFalse);
      expect(find.text('Sign out?'), findsNothing);
    });

    testWidgets('raises above the keyboard instead of covering the field', (
      tester,
    ) async {
      tester.view.viewInsets = const FakeViewPadding(bottom: 840);
      addTearDown(tester.view.resetViewInsets);
      final controller = TextEditingController();
      addTearDown(controller.dispose);

      await _openDialog(
        tester,
        open: (context) async {
          await showKansoDialog<void>(
            context,
            builder: (dialogContext) => KansoDialog(
              title: 'Rename',
              actions: <Widget>[
                TextButton(
                  onPressed: () => Navigator.of(dialogContext).pop(),
                  child: const Text('Close'),
                ),
              ],
              child: KansoTextInput(
                controller: controller,
                autofocus: true,
              ),
            ),
          );
        },
      );

      final padding = tester.widget<AnimatedPadding>(
        find.ancestor(
          of: find.byType(KansoDialog),
          matching: find.byType(AnimatedPadding),
        ),
      );
      expect(
        (padding.padding as EdgeInsets).bottom,
        closeTo(840 / tester.view.devicePixelRatio, 0.01),
      );
      expect(find.byType(KansoTextInput), findsOneWidget);
    });
  });
}
