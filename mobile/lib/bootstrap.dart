// Kanso — bootstrap.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/widgets.dart';

import 'app.dart';
import 'core/bridge/frb_generated.dart';
import 'core/configs/app_environment.dart';
import 'core/models/data/appearance_status.dart';
import 'core/router/kanso_router.dart';
import 'core/state/appearance/appearance_bloc.dart';
import 'core/utils/license_registration.dart';
import 'injection.dart';
import 'providers.dart';

/// Prepares the app and runs it for [environment].
Future<void> bootstrap(AppEnvironment environment) async {
  WidgetsFlutterBinding.ensureInitialized();
  registerBundledLicenses();
  // The Rust core: its library is loaded (the build puts it in the app)
  // before anything opens a note library.
  await KansoCore.init();
  await configureDependencies(environment.injectableName);
  // The saved themes and fonts are read before the first frame, so the app
  // never flashes the default look.
  final appearance = getIt<AppearanceBloc>()
    ..add(const AppearanceEvent.started());
  await appearance.stream.firstWhere(
    (state) => state.status != AppearanceStatus.loading,
  );
  runApp(
    AppProviders(
      appearance: appearance,
      child: App(router: getIt<KansoRouter>()),
    ),
  );
}
