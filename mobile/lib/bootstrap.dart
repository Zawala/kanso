// Kanso — bootstrap.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/widgets.dart';

import 'app.dart';
import 'core/bridge/frb_generated.dart';
import 'core/configs/app_environment.dart';
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
  runApp(const AppProviders(child: App()));
}
