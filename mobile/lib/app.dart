// Kanso — app.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import 'core/configs/kanso_theme.dart';
import 'core/state/appearance/appearance_bloc.dart';
import 'core/utils/context_extensions.dart';
import 'core/utils/kanso_scroll_behavior.dart';
import 'design/views/pages/design_gallery_page.dart';
import 'gen/l10n/app_localizations.dart';

/// The Kanso app: themes, language and the first page, which for now is the
/// gallery of the design proposal. The themes are the ones chosen, drawn
/// from the theme files' tokens.
class App extends StatelessWidget {
  const App({super.key});

  @override
  Widget build(BuildContext context) {
    return BlocBuilder<AppearanceBloc, AppearanceState>(
      builder: (context, state) {
        return MaterialApp(
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
          home: const DesignGalleryPage(),
        );
      },
    );
  }
}
