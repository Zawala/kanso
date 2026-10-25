// Kanso — bootstrap.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/widgets.dart';

import 'app.dart';
import 'core/configs/app_environment.dart';
import 'core/utils/license_registration.dart';
import 'injection.dart';
import 'providers.dart';

/// Prepares the app and runs it for [environment].
Future<void> bootstrap(AppEnvironment environment) async {
  WidgetsFlutterBinding.ensureInitialized();
  registerBundledLicenses();
  configureDependencies(environment.injectableName);
  runApp(const AppProviders(child: App()));
}
