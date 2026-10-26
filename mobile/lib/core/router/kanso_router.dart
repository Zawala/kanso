// Kanso — kanso_router.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/cupertino.dart';
import 'package:go_router/go_router.dart';
import 'package:injectable/injectable.dart';

import '../../design/views/pages/design_gallery_page.dart';
import '../../note/views/pages/note_page.dart';
import '../../note_list/views/pages/note_list_page.dart';
import '../../search/views/pages/search_page.dart';
import '../../settings/views/pages/fonts_page.dart';
import '../../settings/views/pages/licences_page.dart';
import '../../settings/views/pages/settings_page.dart';
import '../../shell/views/widgets/browse_shell.dart';
import '../../themes/views/pages/theme_picker_page.dart';
import '../bridge/api/types.dart';
import '../models/data/browse_location.dart';
import '../models/data/design_gallery_access.dart';
import '../models/data/search_scope_kind.dart';
import 'deep_links.dart';
import 'kanso_routes.dart';

/// The app's routes.
///
/// * Every place and search is under one shell, [BrowseShell], and a note
///   opened from one is that address plus `/note/:noteId`.
/// * Settings is a shell of its own on top of the root navigator, so it
///   opens over what the person was doing and closes back to it, with
///   Theme, Fonts and Licences sliding inside it.
/// * `kanso://open/…` links and the short forms (`/note/:id`,
///   `/notebook/:id`) land on the addresses above; an address that is
///   nothing goes to All Notes.
@singleton
class KansoRouter {
  KansoRouter(this._gallery) {
    config = _build();
  }

  final DesignGalleryAccess _gallery;

  final GlobalKey<NavigatorState> _rootKey = GlobalKey<NavigatorState>(
    debugLabel: 'root',
  );
  final GlobalKey<NavigatorState> _browseKey = GlobalKey<NavigatorState>(
    debugLabel: 'browse',
  );
  final GlobalKey<NavigatorState> _settingsKey = GlobalKey<NavigatorState>(
    debugLabel: 'settings',
  );

  /// The router the app runs with.
  late final GoRouter config;

  GoRouter _build() {
    return GoRouter(
      navigatorKey: _rootKey,
      initialLocation: KansoRoutes.notes,
      // The app's own links carry a scheme and a host, which the router
      // ignores; they are turned into addresses of the app here.
      redirect: (context, state) =>
          state.uri.hasScheme ? redirectLocation(state.uri) : null,
      onException: (context, state, router) => router.go(KansoRoutes.notes),
      routes: <RouteBase>[
        GoRoute(
          path: KansoRoutes.home,
          redirect: (context, state) => redirectLocation(state.uri),
        ),
        GoRoute(
          path: '${KansoRoutes.noteById}/:noteId',
          redirect: (context, state) => redirectLocation(state.uri),
        ),
        GoRoute(
          path: '${KansoRoutes.notebookById}/:notebookId',
          redirect: (context, state) => redirectLocation(state.uri),
        ),
        ShellRoute(
          navigatorKey: _browseKey,
          builder: (context, state, child) => BrowseShell(
            location: BrowseLocation.parse(state.uri),
            designGallery: _gallery.available,
            child: child,
          ),
          routes: <RouteBase>[
            _placeRoute(KansoRoutes.notes, (_) => const ListPlace.all()),
            _placeRoute(KansoRoutes.pinned, (_) => const ListPlace.pinned()),
            _placeRoute(KansoRoutes.recent, (_) => const ListPlace.recent()),
            _placeRoute(KansoRoutes.trash, (_) => const ListPlace.trash()),
            _placeRoute(
              '${KansoRoutes.notebooks}/:notebookId',
              (state) =>
                  ListPlace.notebook(id: state.pathParameters['notebookId']!),
            ),
            _placeRoute(
              '${KansoRoutes.tags}/:tagId',
              (state) => ListPlace.tag(id: state.pathParameters['tagId']!),
            ),
            GoRoute(
              path: KansoRoutes.search,
              pageBuilder: (context, state) => CupertinoPage<void>(
                key: state.pageKey,
                child: SearchPage(
                  query: state.uri.queryParameters['q'] ?? '',
                  scope: SearchScopeKind.values.firstWhere(
                    (kind) => kind.name == state.uri.queryParameters['scope'],
                    orElse: () => SearchScopeKind.all,
                  ),
                  notebookId: state.uri.queryParameters['in'],
                ),
              ),
              routes: <RouteBase>[_noteRoute()],
            ),
          ],
        ),
        ShellRoute(
          navigatorKey: _settingsKey,
          pageBuilder: (context, state, child) =>
              CupertinoPage<void>(key: state.pageKey, child: child),
          routes: <RouteBase>[
            GoRoute(
              path: KansoRoutes.settings,
              pageBuilder: (context, state) => CupertinoPage<void>(
                key: state.pageKey,
                child: const SettingsPage(),
              ),
              routes: <RouteBase>[
                GoRoute(
                  path: 'theme',
                  pageBuilder: (context, state) => CupertinoPage<void>(
                    key: state.pageKey,
                    child: const ThemePickerPage(),
                  ),
                ),
                GoRoute(
                  path: 'fonts',
                  pageBuilder: (context, state) => CupertinoPage<void>(
                    key: state.pageKey,
                    child: FontsPage(
                      atTextSize:
                          state.uri.queryParameters[KansoRoutes
                              .fontsAtParameter] ==
                          KansoRoutes.fontsAtSize,
                    ),
                  ),
                ),
                GoRoute(
                  path: 'licences',
                  pageBuilder: (context, state) => CupertinoPage<void>(
                    key: state.pageKey,
                    child: const LicencesPage(),
                  ),
                ),
              ],
            ),
          ],
        ),
        if (_gallery.available)
          GoRoute(
            path: KansoRoutes.design,
            pageBuilder: (context, state) => CupertinoPage<void>(
              key: state.pageKey,
              child: const DesignGalleryPage(),
            ),
          ),
      ],
    );
  }

  GoRoute _placeRoute(String path, ListPlace Function(GoRouterState) place) {
    return GoRoute(
      path: path,
      pageBuilder: (context, state) => CupertinoPage<void>(
        key: state.pageKey,
        child: NoteListPage(place: place(state)),
      ),
      routes: <RouteBase>[_noteRoute()],
    );
  }

  GoRoute _noteRoute() {
    return GoRoute(
      path: '${KansoRoutes.note}/:noteId',
      pageBuilder: (context, state) => CupertinoPage<void>(
        key: state.pageKey,
        child: NotePage(noteId: state.pathParameters['noteId']!),
      ),
    );
  }
}
