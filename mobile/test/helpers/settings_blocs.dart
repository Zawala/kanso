// Kanso — settings_blocs.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/widgets.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/models/data/load_status.dart';
import 'package:kanso/core/state/sync_status/sync_status_bloc.dart';
import 'package:kanso/lock/state/biometrics/biometrics_bloc.dart';
import 'package:kanso/lock/state/lock_status/lock_status_bloc.dart';
import 'package:kanso/settings/state/about/about_bloc.dart';

import 'close_bloc_later.dart';
import 'fixture_about_repository.dart';
import 'fixture_device_lock_repository.dart';
import 'fixture_session_repository.dart';
import 'fixture_sync_repository.dart';

/// The BLoCs the settings screens read, read: the version of the app
/// (0.1.0, build 3) and a sync status, by default no account. The
/// appearance is the golden helper's.
class SettingsBlocs {
  SettingsBlocs._(
    this.about,
    this.sync,
    this.session,
    this.device,
    this.lock,
    this.biometrics,
  );

  final AboutBloc about;
  final SyncStatusBloc sync;

  /// The lock session and the phone's biometrics (see
  /// [FixtureSessionRepository]), and the BLoCs over them.
  final FixtureSessionRepository session;
  final FixtureDeviceLockRepository device;
  final LockStatusBloc lock;
  final BiometricsBloc biometrics;

  /// Makes them and waits until they have read.
  ///
  /// By default nothing is locked, so there is no Locked notes section; pass
  /// a [session] to have one.
  static Future<SettingsBlocs> start({
    SyncStatus status = signedOutStatus,
    FixtureSessionRepository? session,
    FixtureDeviceLockRepository? device,
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
    final lockSession =
        session ?? FixtureSessionRepository(hasLockPassword: false);
    final lockDevice = device ?? FixtureDeviceLockRepository();
    final lock = LockStatusBloc(lockSession, lockDevice)
      ..add(const LockStatusEvent.started());
    closeBlocLater(lock);
    await lock.stream.firstWhere((state) => state.loaded);
    final biometrics = BiometricsBloc(lockSession, lockDevice);
    closeBlocLater(biometrics);
    return SettingsBlocs._(
      about,
      sync,
      lockSession,
      lockDevice,
      lock,
      biometrics,
    );
  }

  /// Providers of them, to put above a screen, including one that is a
  /// route pushed later.
  List<BlocProvider<dynamic>> get providers => <BlocProvider<dynamic>>[
    BlocProvider<AboutBloc>.value(value: about),
    BlocProvider<SyncStatusBloc>.value(value: sync),
    BlocProvider<LockStatusBloc>.value(value: lock),
    BlocProvider<BiometricsBloc>.value(value: biometrics),
  ];

  /// [child] with them above it.
  Widget provide(Widget child) =>
      MultiBlocProvider(providers: providers, child: child);
}
