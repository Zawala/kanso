// Kanso — mock_repositories.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:kanso/core/models/repos/abs/about_repository.dart';
import 'package:kanso/core/models/repos/abs/board_repository.dart';
import 'package:kanso/core/models/repos/abs/haptics_repository.dart';
import 'package:kanso/core/models/repos/abs/journal_repository.dart';
import 'package:kanso/core/models/repos/abs/library_repository.dart';
import 'package:kanso/core/models/repos/abs/licences_repository.dart';
import 'package:kanso/core/models/repos/abs/reminders_repository.dart';
import 'package:kanso/core/models/repos/abs/sync_repository.dart';
import 'package:kanso/core/models/repos/abs/templates_repository.dart';
import 'package:mocktail/mocktail.dart';

class MockLibraryRepository extends Mock implements LibraryRepository {}

class MockSyncRepository extends Mock implements SyncRepository {}

class MockHapticsRepository extends Mock implements HapticsRepository {}

class MockAboutRepository extends Mock implements AboutRepository {}

class MockLicencesRepository extends Mock implements LicencesRepository {}

class MockJournalRepository extends Mock implements JournalRepository {}

class MockTemplatesRepository extends Mock implements TemplatesRepository {}

class MockBoardRepository extends Mock implements BoardRepository {}

class MockRemindersRepository extends Mock implements RemindersRepository {}
