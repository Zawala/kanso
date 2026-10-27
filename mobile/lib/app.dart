// Kanso — app.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import 'core/configs/kanso_theme.dart';
import 'core/router/kanso_router.dart';
import 'core/state/appearance/appearance_bloc.dart';
import 'core/utils/context_extensions.dart';
import 'core/utils/kanso_scroll_behavior.dart';
import 'core/views/widgets/layout_class_reporter.dart';
import 'crash/views/widgets/crash_notice_host.dart';
import 'gen/l10n/app_localizations.dart';
import 'reminders/utils/reminders_coordinator.dart';
import 'reminders/views/widgets/reminders_host.dart';
import 'settings/views/widgets/backup_notice_host.dart';

/// The Kanso app: the themes chosen (drawn from the theme files' tokens),
/// the language and the routes.
class App extends StatelessWidget {
  const App({required this.router, this.reminders, super.key});

  /// The routes of the app, and how many panes there is room for.
  final KansoRouter router;

  /// Keeps reminders planned and opens the tasks they are for; none in a
  /// test that does not run it.
  final RemindersCoordinator? reminders;

  @override
  Widget build(BuildContext context) {
    return BlocBuilder<AppearanceBloc, AppearanceState>(
      builder: (context, state) {
        return MaterialApp.router(
          debugShowCheckedModeBanner: false,
          onGenerateTitle: (context) => context.l10n.appName,
          theme: KansoTheme.build(
            state.light,
            fonts: state.fonts,
            textSize: state.textSize,
          ),
          darkTheme: KansoTheme.build(
            state.dark,
            fonts: state.fonts,
            textSize: state.textSize,
          ),
          themeMode: state.themeMode,
          scrollBehavior: const KansoScrollBehavior(),
          localizationsDelegates: AppLocalizations.localizationsDelegates,
          supportedLocales: AppLocalizations.supportedLocales,
          routerConfig: router.config,
          builder: (context, child) {
            final coordinator = reminders;
            final content = CrashNoticeHost(
              child: BackupNoticeHost(child: child!),
            );
            return LayoutClassReporter(
              notifier: router.layoutClass,
              child: coordinator == null
                  ? content
                  : RemindersHost(
                      coordinator: coordinator,
                      router: router.config,
                      child: content,
                    ),
            );
          },
        );
      },
    );
  }
}
