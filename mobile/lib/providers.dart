// Kanso — providers.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/widgets.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import 'core/state/appearance/appearance_bloc.dart';
import 'core/state/capture_inbox/capture_inbox_bloc.dart';
import 'core/state/library_index/library_index_bloc.dart';
import 'core/state/list_navigation/list_navigation_bloc.dart';
import 'core/state/navigation_history/navigation_history_bloc.dart';
import 'core/state/sync_status/sync_status_bloc.dart';
import 'core/state/widget_colors/widget_colors_bloc.dart';
import 'core/views/widgets/capture_inbox_host.dart';
import 'core/views/widgets/widget_colors_host.dart';
import 'injection.dart';
import 'settings/state/backup_notice/backup_notice_bloc.dart';
import 'shell/state/library_pane/library_pane_bloc.dart';

/// The BLoCs the whole app shares, above the [child].
///
/// [appearance] is read before the app runs (see `bootstrap`), so the first
/// frame is already in the chosen themes.
class AppProviders extends StatelessWidget {
  const AppProviders({
    required this.appearance,
    required this.child,
    super.key,
  });

  final AppearanceBloc appearance;
  final Widget child;

  @override
  Widget build(BuildContext context) {
    return MultiBlocProvider(
      providers: <BlocProvider<dynamic>>[
        BlocProvider<AppearanceBloc>.value(value: appearance),
        BlocProvider<LibraryIndexBloc>(
          create: (_) =>
              getIt<LibraryIndexBloc>()..add(const LibraryIndexEvent.started()),
        ),
        BlocProvider<SyncStatusBloc>(
          create: (_) =>
              getIt<SyncStatusBloc>()..add(const SyncStatusEvent.started()),
        ),
        BlocProvider<BackupNoticeBloc>(
          create: (_) =>
              getIt<BackupNoticeBloc>()..add(const BackupNoticeEvent.started()),
        ),
        BlocProvider<ListNavigationBloc>(
          create: (_) => getIt<ListNavigationBloc>(),
        ),
        BlocProvider<LibraryPaneBloc>(
          create: (_) => getIt<LibraryPaneBloc>(),
        ),
        BlocProvider<NavigationHistoryBloc>(
          create: (_) => getIt<NavigationHistoryBloc>(),
        ),
        BlocProvider<CaptureInboxBloc>(
          create: (_) =>
              getIt<CaptureInboxBloc>()
                ..add(const CaptureInboxEvent.saveRequested()),
        ),
        BlocProvider<WidgetColorsBloc>(
          create: (_) => getIt<WidgetColorsBloc>()
            ..add(
              WidgetColorsEvent.changed(
                light: appearance.state.light.colors,
                dark: appearance.state.dark.colors,
              ),
            ),
        ),
      ],
      // What other apps shared is saved as the app starts (above) and each
      // time it comes back to the front; the widgets follow the themes.
      child: CaptureInboxHost(child: WidgetColorsHost(child: child)),
    );
  }
}
