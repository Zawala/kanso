// Kanso — locked_property_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:math';

import 'package:flutter_test/flutter_test.dart';
import 'package:glados/glados.dart'
    show ExploreConfig, Glados2, IntAnys, ListAnys, any;
import 'package:kanso/core/bridge/api/library.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/models/repos/impl/rust_library_repository.dart';
import 'package:kanso/core/models/repos/impl/rust_renderer_repository.dart';
import 'package:kanso/core/models/repos/impl/rust_session_repository.dart';

import '../../helpers/core_library.dart';
import '../../helpers/dump.dart';
import 'suites/open_library.dart';

/// A locked note's text never reaches Dart while it is closed: whatever the
/// repositories are asked, in whatever order, nothing they return holds the
/// words only the text has. The same property the Rust tests check
/// (`crates/kanso-mobile/tests/locked.rs`), through the bridge and the
/// repositories, so a cache or a copy on the Dart side would fail it.
void main() {
  setUpAll(initCoreForTests);

  // The property above is only as good as its marker: an open note does
  // show its text (search never reads a locked note, open or not).
  test('control: while the lock is open the marker shows', () async {
    final library = await openTestLibrary();
    final notes = LibraryRepositoryUnderTest(library);
    final id = await makeNote(
      library,
      'Title',
      '---\nowner: $_marker\n---\nbody of $_marker',
    );
    await library.locks().lockNote(id: id, password: lockPassword);
    expect(dump(await notes.library.note(id)), contains(_marker));
    expect(dump(await notes.library.properties(id)), contains(_marker));
    await library.locks().lock();
    expect(dump(await notes.library.note(id)), isNot(contains(_marker)));
  });

  Glados2<int, List<int>>(
    any.intInRange(1, 8),
    any.listWithLengthInRange(1, 14, any.intInRange(0, 50)),
    ExploreConfig(random: Random(20261017), numRuns: 10),
  ).test(
    'a closed note never returns its text, whatever is asked',
    (lockedMask, operations) async {
      final library = await openTestLibrary();
      final notes = LibraryRepositoryUnderTest(library);
      // Three notes; the ones the mask picks are locked. Each text holds a
      // marker that only the text has.
      final ids = <String>[];
      for (var i = 0; i < 3; i++) {
        ids.add(
          await makeNote(
            library,
            'Title $i',
            '---\nowner: $_marker$i\nrank: $i\n---\nbody of $_marker$i',
          ),
        );
      }
      final lockedIndexes = <int>[
        for (var i = 0; i < 3; i++)
          if (lockedMask & (1 << i) != 0) i,
      ];
      for (final i in lockedIndexes) {
        await library.locks().lockNote(id: ids[i], password: lockPassword);
      }
      await library.locks().lock();

      var closed = true;
      var unlocks = 0;
      final seen = StringBuffer();

      Future<String> perform(int operation) async {
        final index = operation ~/ 10 % 3;
        switch (operation % 10) {
          case 0:
            final place = <ListPlace>[
              const ListPlace.all(),
              const ListPlace.pinned(),
              const ListPlace.recent(),
            ][index];
            return dump(
              await notes.library.list(
                ListQuery(
                  place: place,
                  pinnedOnly: false,
                  sort: SortKey.updated,
                  descending: true,
                ),
              ),
            );
          case 1:
            return dump(await notes.library.note(ids[index]));
          case 2:
            final scope = <SearchScope>[
              const SearchScope.all(),
              const SearchScope.titles(),
              const SearchScope.tags(),
            ][index];
            return dump(await notes.library.search(_marker, scope: scope));
          case 3:
            return dump(await notes.library.properties(ids[index]));
          case 4:
            return dump(
              await notes.library.queryView(
                'from: status:none has:rank\ncolumns: owner, rank',
              ),
            );
          case 5:
            return dump(await notes.renderer.previewHtml(ids[index], _palette));
          case 6:
            return dump(
              <Object?>[
                await notes.library.save(
                  id: ids[index],
                  baseRev: 'x',
                  body: 'edited',
                ),
                await notes.library.change(<String>[
                  ids[index],
                ], const NoteChange.togglePin()),
                await notes.library.overview(),
              ],
            );
          case 7:
            return dump(
              await notes.library.create(
                const NewNote(title: 'Another', body: 'plain'),
              ),
            );
          case 8:
            closed = true;
            return dump(await notes.session.lock());
          default:
            // Real key derivation: a few times a case, not more.
            if (unlocks >= 2) {
              return 'skipped';
            }
            unlocks++;
            final unlocked = await notes.session.unlock(lockPassword);
            closed = false;
            return '$unlocked';
        }
      }

      for (final operation in operations) {
        final returned = await perform(operation);
        if (closed) {
          seen.writeln(returned);
        }
      }
      // Only while closed: an open session may show its text.
      final lockedMarkers = <String>[
        for (final i in lockedIndexes) '$_marker$i',
      ];
      for (final marker in lockedMarkers) {
        expect(
          seen.toString(),
          isNot(contains(marker)),
          reason: 'a closed locked note gave back $marker',
        );
      }
    },
    timeout: const Timeout(Duration(minutes: 5)),
  );
}

const String _marker = 'SECRETMARKER';

const PaletteDto _palette = PaletteDto(
  dark: false,
  background: 0xFFFFFF,
  text: 0x222222,
  muted: 0x888888,
  accent: 0x3366AA,
  border: 0xDDDDDD,
  surface: 0xF4F4F4,
  quote: 0x666666,
  taskDone: 0x888888,
  inlineCodeBackground: 0xEEEEEE,
);

/// The repositories over one library.
class LibraryRepositoryUnderTest {
  LibraryRepositoryUnderTest(KansoLibrary library)
    : library = RustLibraryRepository(library),
      session = RustSessionRepository(library.locks()),
      renderer = RustRendererRepository(library.renderer());

  final RustLibraryRepository library;
  final RustSessionRepository session;
  final RustRendererRepository renderer;
}
