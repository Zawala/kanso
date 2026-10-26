// Kanso — licences_bloc.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:bloc/bloc.dart';
import 'package:bloc_concurrency/bloc_concurrency.dart';
import 'package:freezed_annotation/freezed_annotation.dart';
import 'package:injectable/injectable.dart';

import '../../../core/models/data/licence_package.dart';
import '../../../core/models/data/load_status.dart';
import '../../../core/models/repos/abs/licences_repository.dart';

part 'licences_bloc.freezed.dart';
part 'licences_event.dart';
part 'licences_state.dart';

/// The licences of what the app is made of: the packages, and the one whose
/// licences are open.
@injectable
class LicencesBloc extends Bloc<LicencesEvent, LicencesState> {
  LicencesBloc(this._licences) : super(const LicencesState()) {
    on<LicencesStartedEvent>(_onStarted, transformer: restartable());
    on<LicencesPackageToggledEvent>(
      (event, emit) => emit(
        state.copyWith(
          expanded: state.expanded == event.name ? null : event.name,
        ),
      ),
    );
  }

  final LicencesRepository _licences;

  Future<void> _onStarted(
    LicencesStartedEvent event,
    Emitter<LicencesState> emit,
  ) async {
    emit(const LicencesState());
    final result = await _licences.licences();
    emit(
      result.fold(
        (error) => const LicencesState(status: LoadStatus.failed),
        (packages) =>
            LicencesState(status: LoadStatus.ready, packages: packages),
      ),
    );
  }
}
