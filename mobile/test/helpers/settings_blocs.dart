// Kanso — settings_blocs.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/widgets.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/models/data/load_status.dart';
import 'package:kanso/core/state/account/account_bloc.dart';
import 'package:kanso/core/state/sync_status/sync_status_bloc.dart';
import 'package:kanso/settings/state/about/about_bloc.dart';

import 'close_bloc_later.dart';
import 'fixture_about_repository.dart';
import 'fixture_sync_repository.dart';
import 'seeded_account_blocs.dart';

/// The BLoCs the settings screens read, read: the version of the app
/// (0.1.0, build 3), the account and a sync status, by default no account.
/// The appearance is the golden helper's.
class SettingsBlocs {
  SettingsBlocs._(this.about, this.sync, this.account);

  final AboutBloc about;
  final SyncStatusBloc sync;
  final SeededAccountBloc account;

  /// Makes them and waits until they have read. The account is [account],
  /// by default none.
  static Future<SettingsBlocs> start({
    SyncStatus status = signedOutStatus,
    AccountState account = const AccountState.signedOut(),
  }) async {
    final about = AboutBloc(FixtureAboutRepository())
      ..add(const AboutEvent.started());
    closeBlocLater(about);
    final sync = SyncStatusBloc(FixtureSyncRepository(current: status))
      ..add(const SyncStatusEvent.started());
    closeBlocLater(sync);
    await about.stream.firstWhere((state) => state.status == LoadStatus.ready);
    if (sync.state.status == null) {
      await sync.stream.firstWhere((state) => state.status != null);
    }
    final seeded = SeededAccountBloc(AccountBlocState(account: account));
    closeBlocLater(seeded);
    return SettingsBlocs._(about, sync, seeded);
  }

  /// Providers of them, to put above a screen, including one that is a
  /// route pushed later.
  List<BlocProvider<dynamic>> get providers => <BlocProvider<dynamic>>[
    BlocProvider<AboutBloc>.value(value: about),
    BlocProvider<SyncStatusBloc>.value(value: sync),
    BlocProvider<AccountBloc>.value(value: account),
  ];

  /// [child] with them above it.
  Widget provide(Widget child) =>
      MultiBlocProvider(providers: providers, child: child);
}
