// Kanso — mock_data.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'mock_block.dart';
import 'mock_completion.dart';

/// The content of the mock screens that are not built yet: fixed, invented
/// and neutral, so every golden image is the same on every run.
///
/// Note titles, text and notebook names are content as a person would have
/// written it, not interface copy, so they live here and not in the
/// localisation files.
abstract final class MockData {
  static const String openNoteTitle = 'Sync design notes';

  /// The same note while it is edited: the part around the caret, and a
  /// wiki link being typed.
  static const List<MockBlock> editBlocks = <MockBlock>[
    MockBlock.heading(level: 2, text: 'Open questions'),
    MockBlock.bullets(<String>[
      'How long a device may stay offline',
      'What a conflict looks like to a person',
    ]),
  ];

  /// What has been typed after the last block of [editBlocks]; the caret
  /// follows it.
  static const String editTyping = 'See also [[Sy';

  static const int editPropertiesCount = 3;

  static const List<MockCompletion> completions = <MockCompletion>[
    MockCompletion(title: 'Sync design notes', notebook: 'Projects / Kanso'),
    MockCompletion(title: 'Syncing across devices', notebook: 'Reading'),
    MockCompletion(title: 'Sync wishlist', notebook: 'Inbox'),
  ];
}
