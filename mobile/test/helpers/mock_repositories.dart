// Kanso — mock_repositories.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:bloc_test/bloc_test.dart';
import 'package:kanso/core/models/repos/abs/about_repository.dart';
import 'package:kanso/core/models/repos/abs/haptics_repository.dart';
import 'package:kanso/core/models/repos/abs/library_repository.dart';
import 'package:kanso/core/models/repos/abs/licences_repository.dart';
import 'package:kanso/core/models/repos/abs/sync_repository.dart';
import 'package:kanso/core/state/undo_notice/undo_notice_bloc.dart';
import 'package:kanso/editor/models/repos/abs/web_link_repository.dart';
import 'package:mocktail/mocktail.dart';

class MockLibraryRepository extends Mock implements LibraryRepository {}

class MockSyncRepository extends Mock implements SyncRepository {}

class MockHapticsRepository extends Mock implements HapticsRepository {}

class MockWebLinkRepository extends Mock implements WebLinkRepository {}

class MockAboutRepository extends Mock implements AboutRepository {}

class MockLicencesRepository extends Mock implements LicencesRepository {}

class MockUndoNoticeBloc extends MockBloc<UndoNoticeEvent, UndoNoticeState>
    implements UndoNoticeBloc {}
