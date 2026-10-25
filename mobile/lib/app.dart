// Kanso — app.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import 'core/configs/kanso_theme.dart';
import 'core/state/appearance/appearance_bloc.dart';
import 'core/utils/appearance_mode_theme_mode.dart';
import 'core/utils/context_extensions.dart';
import 'core/utils/kanso_scroll_behavior.dart';
import 'gen/l10n/app_localizations.dart';
import 'home/views/pages/home_page.dart';

/// The Kanso app: themes, language and the first page.
class App extends StatelessWidget {
  const App({super.key});

  @override
  Widget build(BuildContext context) {
    return BlocBuilder<AppearanceBloc, AppearanceState>(
      builder: (context, state) {
        return MaterialApp(
          debugShowCheckedModeBanner: false,
          onGenerateTitle: (context) => context.l10n.appName,
          theme: KansoTheme.light,
          darkTheme: KansoTheme.dark,
          themeMode: state.mode.themeMode,
          scrollBehavior: const KansoScrollBehavior(),
          localizationsDelegates: AppLocalizations.localizationsDelegates,
          supportedLocales: AppLocalizations.supportedLocales,
          home: const HomePage(),
        );
      },
    );
  }
}
