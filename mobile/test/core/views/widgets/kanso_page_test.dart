import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/views/widgets/kanso_page.dart';

void main() {
  testWidgets('resizes for the keyboard only when requested', (tester) async {
    tester.view.physicalSize = const Size(400, 800);
    tester.view.devicePixelRatio = 1;
    tester.view.viewInsets = const FakeViewPadding(bottom: 300);
    addTearDown(tester.view.reset);

    double height = 0;
    Widget page({required bool resize}) => MaterialApp(
      home: KansoPage(
        resizeToAvoidBottomInset: resize,
        backgroundColor: Colors.amber,
        body: LayoutBuilder(
          builder: (context, constraints) {
            height = constraints.maxHeight;
            return const SizedBox.expand();
          },
        ),
      ),
    );

    await tester.pumpWidget(page(resize: true));
    await tester.pump(const Duration(milliseconds: 200));
    final resizedHeight = height;
    expect(resizedHeight, 500);

    await tester.pumpWidget(page(resize: false));
    await tester.pump(const Duration(milliseconds: 200));
    expect(height, 800);
    expect(
      tester
          .widget<ColoredBox>(
            find.descendant(
              of: find.byType(KansoPage),
              matching: find.byType(ColoredBox),
            ),
          )
          .color,
      Colors.amber,
    );
  });
}
