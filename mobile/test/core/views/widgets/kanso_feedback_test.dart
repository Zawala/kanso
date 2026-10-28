import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/configs/kanso_colors.dart';
import 'package:kanso/core/views/widgets/kanso_feedback.dart';

void main() {
  Widget app(Widget child) => MaterialApp(
    theme: ThemeData(
      extensions: const <ThemeExtension<dynamic>>[
        KansoColors.light,
      ],
    ),
    home: Scaffold(body: KansoFeedbackHost(child: child)),
  );

  testWidgets('shows one message and coalesces a duplicate', (tester) async {
    late BuildContext context;
    await tester.pumpWidget(
      app(
        Builder(
          builder: (builderContext) {
            context = builderContext;
            return const SizedBox.expand();
          },
        ),
      ),
    );

    KansoFeedback.show(context, 'Saved');
    await tester.pump();
    KansoFeedback.show(context, 'Saved');
    await tester.pump(const Duration(milliseconds: 2000));

    expect(find.text('Saved'), findsOneWidget);
    await tester.pump(const Duration(milliseconds: 500));
    await tester.pumpAndSettle();
    expect(find.text('Saved'), findsNothing);
  });

  testWidgets('runs the reversing action and hides the pill', (tester) async {
    late BuildContext context;
    var undone = false;
    await tester.pumpWidget(
      app(
        Builder(
          builder: (builderContext) {
            context = builderContext;
            return const SizedBox.expand();
          },
        ),
      ),
    );

    KansoFeedback.show(
      context,
      'Note deleted',
      actionLabel: 'Undo',
      onAction: () => undone = true,
    );
    await tester.pumpAndSettle();
    await tester.tap(find.text('Undo'));
    await tester.pumpAndSettle();

    expect(undone, isTrue);
    expect(find.text('Note deleted'), findsNothing);
  });
}
