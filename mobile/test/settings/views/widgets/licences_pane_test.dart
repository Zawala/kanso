// Kanso — licences_pane_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:dartz/dartz.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/core/configs/kanso_sizes.dart';
import 'package:kanso/core/models/data/licence_package.dart';
import 'package:kanso/core/views/widgets/kanso_top_bar.dart';
import 'package:kanso/settings/state/licences/licences_bloc.dart';
import 'package:kanso/settings/views/widgets/licences_pane.dart';
import 'package:mocktail/mocktail.dart';

import '../../../helpers/close_bloc_later.dart';
import '../../../helpers/mock_repositories.dart';
import '../../../helpers/pump_routed.dart';

const _packages = <LicencePackage>[
  LicencePackage(name: 'alpha', licences: <String>['Alpha terms.']),
  LicencePackage(name: 'beta', licences: <String>['Beta one.', 'Beta two.']),
];

Future<MockLicencesRepository> _pump(
  WidgetTester tester,
  Future<Either<ApplicationError, List<LicencePackage>>> Function() read,
) async {
  final repository = MockLicencesRepository();
  when(repository.licences).thenAnswer((_) => read());
  final bloc = LicencesBloc(repository)..add(const LicencesEvent.started());
  closeBlocLater(bloc);
  await pumpRouted(
    tester,
    BlocProvider<LicencesBloc>.value(value: bloc, child: const LicencesPane()),
  );
  await tester.pump();
  await tester.pump();
  return repository;
}

void main() {
  group('LicencesPane', () {
    testWidgets('lists the packages and opens one to its licences', (
      tester,
    ) async {
      await _pump(
        tester,
        () async => const Right<ApplicationError, List<LicencePackage>>(
          _packages,
        ),
      );
      expect(find.text('alpha'), findsOneWidget);
      expect(find.text('beta'), findsOneWidget);
      expect(find.text('1 licence'), findsOneWidget);
      expect(find.text('2 licences'), findsOneWidget);
      expect(find.text('Alpha terms.'), findsNothing);

      await tester.tap(find.text('alpha'));
      await tester.pump();
      expect(find.text('Alpha terms.'), findsOneWidget);

      await tester.tap(find.text('beta'));
      await tester.pump();
      expect(find.text('Alpha terms.'), findsNothing);
      expect(find.text('Beta one.'), findsOneWidget);
      expect(find.text('Beta two.'), findsOneWidget);
    });

    testWidgets('shows bars while the licences are read', (tester) async {
      final gate = Completer<Either<ApplicationError, List<LicencePackage>>>();
      await _pump(tester, () => gate.future);
      expect(find.bySemanticsLabel('Loading licences'), findsOneWidget);
      gate.complete(
        const Right<ApplicationError, List<LicencePackage>>(_packages),
      );
      await tester.pump();
      await tester.pump();
      expect(find.text('alpha'), findsOneWidget);
    });

    testWidgets('says plainly when they could not be read and tries again', (
      tester,
    ) async {
      var calls = 0;
      final repository = await _pump(tester, () async {
        calls++;
        return calls == 1
            ? const Left<ApplicationError, List<LicencePackage>>(
                ApplicationError.unexpected(message: 'Unreadable.'),
              )
            : const Right<ApplicationError, List<LicencePackage>>(_packages);
      });
      expect(find.text('The licences didn’t load'), findsOneWidget);
      expect(find.textContaining('Unreadable'), findsNothing);
      await tester.tap(find.text('Try again'));
      await tester.pump();
      await tester.pump();
      verify(repository.licences).called(2);
      expect(find.text('alpha'), findsOneWidget);
    });

    testWidgets('starts under the bar, whatever the status bar takes', (
      tester,
    ) async {
      const inset = FakeViewPadding(top: 59, bottom: 34);
      tester.view
        ..padding = inset
        ..viewPadding = inset;
      await _pump(
        tester,
        () async => const Right<ApplicationError, List<LicencePackage>>(
          _packages,
        ),
      );
      final bar = tester.getBottomLeft(find.byType(KansoTopBar)).dy;
      final first = tester.getTopLeft(find.text('alpha')).dy;
      // A row is one touch target tall, with the card's gap above it.
      expect(first - bar, lessThan(KansoSizes.touchTarget));
    });

    testWidgets('says so when there are none', (tester) async {
      await _pump(
        tester,
        () async => const Right<ApplicationError, List<LicencePackage>>(
          <LicencePackage>[],
        ),
      );
      expect(find.text('No licences'), findsOneWidget);
    });
  });
}
