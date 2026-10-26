// Kanso — kanso_plain_text_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/views/widgets/kanso_plain_text.dart';

void main() {
  testWidgets('text under it has no decoration, with or without a page', (
    tester,
  ) async {
    await tester.pumpWidget(
      const Directionality(
        textDirection: TextDirection.ltr,
        child: KansoPlainText(child: Text('Plain')),
      ),
    );
    final style = DefaultTextStyle.of(tester.element(find.text('Plain')));
    expect(style.style.decoration, TextDecoration.none);
  });
}
