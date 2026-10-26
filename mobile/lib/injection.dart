// Kanso — injection.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:get_it/get_it.dart';
import 'package:injectable/injectable.dart';

import 'injection.config.dart';

/// The app's service locator. Resolve from it at composition boundaries
/// (providers, bootstrap), never inside business logic.
final GetIt getIt = GetIt.instance;

/// Registers everything. The library is opened here, so `KansoCore.init`
/// must have run first.
@InjectableInit(preferRelativeImports: true)
Future<void> configureDependencies(String environment) async {
  await getIt.init(environment: environment);
}
