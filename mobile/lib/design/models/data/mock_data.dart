// Kanso — mock_data.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import '../../../core/models/data/note_status.dart';
import '../../../core/models/data/tag_color.dart';
import 'mock_block.dart';
import 'mock_code_token.dart';
import 'mock_completion.dart';
import 'mock_note.dart';
import 'mock_note_group.dart';
import 'mock_notebook.dart';
import 'mock_search_result.dart';
import 'mock_tag.dart';
import 'mock_task_item.dart';
import 'mock_token_kind.dart';

/// The content of the mock screens: fixed, invented and neutral, so every
/// golden image is the same on every run.
///
/// Note titles, text, notebook and tag names, dates and counts are content
/// as a person would have written it, not interface copy, so they live here
/// and not in the localisation files.
abstract final class MockData {
  static const MockTag tagIdea = MockTag(
    name: 'idea',
    color: TagColor.teal,
    count: 8,
  );
  static const MockTag tagDraft = MockTag(
    name: 'draft',
    color: TagColor.orange,
    count: 5,
  );
  static const MockTag tagReading = MockTag(
    name: 'reading',
    color: TagColor.violet,
    count: 11,
  );
  static const MockTag tagTodo = MockTag(
    name: 'todo',
    color: TagColor.pink,
    count: 7,
  );

  static const List<MockTag> tags = <MockTag>[
    tagIdea,
    tagDraft,
    tagReading,
    tagTodo,
  ];

  /// 142 notes in all: the notebooks' counts add up to it.
  static const int allNotesCount = 142;
  static const int pinnedCount = 3;
  static const int tasksCount = 7;
  static const int trashCount = 4;

  static const List<MockNotebook> notebooks = <MockNotebook>[
    MockNotebook(name: 'Inbox', count: 12),
    MockNotebook(name: 'Journal', count: 48),
    MockNotebook(name: 'Projects', count: 23),
    MockNotebook(name: 'Kanso', count: 17, depth: 1),
    MockNotebook(name: 'Garden', count: 6, depth: 1),
    MockNotebook(name: 'Reading', count: 9),
    MockNotebook(name: 'Recipes', count: 14),
    MockNotebook(name: 'Someday', count: 0),
    MockNotebook(name: 'Travel', count: 5),
    MockNotebook(name: 'Work', count: 31),
  ];

  static const MockNote weeklyReview = MockNote(
    title: 'Weekly review',
    excerpt:
        'Finished the sync conflict screen and moved the Garden notes to '
        'next week. Thursday: call about the allotment, then plan the beds.',
    notebook: 'Journal',
    time: 'Mon',
    tags: <MockTag>[tagTodo],
    pinned: true,
  );

  static const MockNote syncDesign = MockNote(
    title: 'Sync design notes',
    excerpt:
        'Every device keeps a full copy. A change is encrypted before it '
        'leaves the device, sent once and merged on the others.',
    notebook: 'Projects / Kanso',
    time: '09:41',
    status: NoteStatus.active,
    tags: <MockTag>[tagIdea, tagDraft],
  );

  static const List<MockNoteGroup> noteGroups = <MockNoteGroup>[
    MockNoteGroup(label: 'Pinned', notes: <MockNote>[weeklyReview]),
    MockNoteGroup(
      label: 'Today',
      notes: <MockNote>[
        syncDesign,
        MockNote(
          title: 'Sourdough starter',
          excerpt:
              'Feed twice a day at room temperature, once a day in the '
              'fridge. It is ready when it doubles within six hours.',
          notebook: 'Recipes',
          time: '08:12',
        ),
      ],
    ),
    MockNoteGroup(
      label: 'Yesterday',
      notes: <MockNote>[
        MockNote(
          title: 'Lisbon in spring',
          excerpt:
              'Stay near the river, walk up to the miradouros in the '
              'morning. Book the train to Sintra a day ahead.',
          notebook: 'Travel',
          time: '18:30',
          status: NoteStatus.onHold,
          tags: <MockTag>[tagIdea],
        ),
        MockNote(
          title: 'Reading list 2026',
          excerpt:
              'Slow software, a book on typefaces, the garden almanac. '
              'Finish one before starting the next.',
          notebook: 'Reading',
          time: '11:05',
          tags: <MockTag>[tagReading],
        ),
      ],
    ),
    MockNoteGroup(
      label: 'Earlier this week',
      notes: <MockNote>[
        MockNote(
          title: 'Garden beds plan',
          excerpt:
              'Three raised beds along the south fence: beans, leafy '
              'greens and herbs. Order the soil before the weekend.',
          notebook: 'Projects / Garden',
          time: 'Thu',
          status: NoteStatus.completed,
          tags: <MockTag>[tagTodo],
        ),
        MockNote(
          title: 'Private journal',
          excerpt: 'Locked. Unlock to read.',
          notebook: 'Journal',
          time: 'Wed',
          locked: true,
        ),
        MockNote(
          title: 'Quarterly goals',
          excerpt:
              'Ship the mobile beta, write the sync chapter of the manual '
              'and take two proper weeks off.',
          notebook: 'Work',
          time: 'Wed',
          status: NoteStatus.dropped,
        ),
      ],
    ),
    MockNoteGroup(
      label: 'October',
      notes: <MockNote>[
        MockNote(
          title: 'Notes on slow software',
          excerpt:
              'Software that does less, tells you less and stays out of '
              'the way. Examples worth copying, and what to avoid.',
          notebook: 'Inbox',
          time: 'Oct 12',
          tags: <MockTag>[tagReading, tagIdea],
        ),
        MockNote(
          title: 'Packing list',
          excerpt:
              'Charger, notebook, the small camera, a light jacket. '
              'Leave the laptop at home.',
          notebook: 'Travel',
          time: 'Oct 9',
          status: NoteStatus.completed,
        ),
      ],
    ),
  ];

  static const String openNoteTitle = 'Sync design notes';
  static const String openNoteNotebook = 'Projects / Kanso';
  static const String openNoteCreated = 'Oct 12, 2026';
  static const String openNoteUpdated = 'Today, 09:41';
  static const List<MockTag> openNoteTags = <MockTag>[tagIdea, tagDraft];

  static const String lockedNoteTitle = 'Private journal';
  static const String lockedNoteNotebook = 'Journal';
  static const String lockedNoteCreated = 'Jan 3, 2026';
  static const String lockedNoteUpdated = 'Wed, 14:20';

  /// The open note, rendered.
  static const List<MockBlock> noteBlocks = <MockBlock>[
    MockBlock.heading(level: 1, text: 'Goals'),
    MockBlock.paragraph(
      'Every device keeps a full copy. A change is **encrypted on the '
      'device**, sent once and *merged* on the others. The first sketch is '
      'in `sync.rs`; the reasoning is in the '
      '[design write-up](https://kanso.app/design).',
    ),
    MockBlock.heading(level: 2, text: 'Open questions'),
    MockBlock.bullets(<String>[
      'How long a device may stay offline',
      'What a conflict looks like to a person',
    ]),
    MockBlock.tasks(<MockTaskItem>[
      MockTaskItem(text: 'Write down the wire format', done: true),
      MockTaskItem(text: 'Try a conflict on three devices'),
      MockTaskItem(text: 'Decide how deleted notes are kept'),
    ]),
    MockBlock.quote('Calm software says less and remembers more.'),
    MockBlock.code(
      language: 'rust',
      lines: <List<MockCodeToken>>[
        <MockCodeToken>[
          MockCodeToken('// last write wins, for now', MockTokenKind.comment),
        ],
        <MockCodeToken>[
          MockCodeToken('fn', MockTokenKind.keyword),
          MockCodeToken(' ', MockTokenKind.plain),
          MockCodeToken('merge', MockTokenKind.function),
          MockCodeToken('(', MockTokenKind.punctuation),
          MockCodeToken('a', MockTokenKind.variable),
          MockCodeToken(': ', MockTokenKind.punctuation),
          MockCodeToken('Note', MockTokenKind.typeName),
          MockCodeToken(', ', MockTokenKind.punctuation),
          MockCodeToken('b', MockTokenKind.variable),
          MockCodeToken(': ', MockTokenKind.punctuation),
          MockCodeToken('Note', MockTokenKind.typeName),
          MockCodeToken(') -> ', MockTokenKind.operator),
          MockCodeToken('Note', MockTokenKind.typeName),
          MockCodeToken(' {', MockTokenKind.punctuation),
        ],
        <MockCodeToken>[
          MockCodeToken('    ', MockTokenKind.plain),
          MockCodeToken('if', MockTokenKind.keyword),
          MockCodeToken(' ', MockTokenKind.plain),
          MockCodeToken('b', MockTokenKind.variable),
          MockCodeToken('.', MockTokenKind.punctuation),
          MockCodeToken('at', MockTokenKind.property),
          MockCodeToken(' > ', MockTokenKind.operator),
          MockCodeToken('a', MockTokenKind.variable),
          MockCodeToken('.', MockTokenKind.punctuation),
          MockCodeToken('at', MockTokenKind.property),
          MockCodeToken(' { ', MockTokenKind.punctuation),
          MockCodeToken('b', MockTokenKind.variable),
          MockCodeToken(' } ', MockTokenKind.punctuation),
          MockCodeToken('else', MockTokenKind.keyword),
          MockCodeToken(' { ', MockTokenKind.punctuation),
          MockCodeToken('a', MockTokenKind.variable),
          MockCodeToken(' }', MockTokenKind.punctuation),
        ],
        <MockCodeToken>[MockCodeToken('}', MockTokenKind.punctuation)],
      ],
    ),
    MockBlock.paragraph('Next: [[Weekly review]]'),
  ];

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

  static const String searchQuery = 'sync';

  static const List<MockSearchResult> searchResults = <MockSearchResult>[
    MockSearchResult(
      title: 'Sync design notes',
      notebook: 'Projects / Kanso',
      snippet:
          '...every device keeps a full copy. A change is encrypted before '
          'it is sent, and sync never reads a note.',
    ),
    MockSearchResult(
      title: 'Weekly review',
      notebook: 'Journal',
      snippet:
          '...finished the sync conflict screen before the Garden notes. '
          'Moved the sync call to Thursday.',
    ),
    MockSearchResult(
      title: 'Syncing across devices',
      notebook: 'Reading',
      snippet:
          '...how apps that sync in the background hide the work from the '
          'person using them.',
    ),
    MockSearchResult(
      title: 'Sync wishlist',
      notebook: 'Inbox',
      snippet:
          '...wake this device when notes change elsewhere, and sync again '
          'when the connection comes back.',
    ),
    MockSearchResult(
      title: 'Quarterly goals',
      notebook: 'Work',
      snippet: '...ship the mobile beta with sync, then write the manual.',
    ),
  ];

  static const String defaultNotebook = 'Inbox';

  /// A notebook with no notes, for the empty state.
  static const String emptyNotebookName = 'Someday';
  static const String signedInAs = 'ngoni';
  static const String version = '0.1.0 (1) \u00b7 patch 0';
}
