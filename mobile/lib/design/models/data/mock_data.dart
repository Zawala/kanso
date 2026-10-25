// Kanso — mock_data.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:ui';

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
import 'theme_appearance.dart';
import 'theme_preview.dart';

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
  static const String editorFont = 'Instrument Sans';
  static const String currentThemeName = 'Kanso Dark';
  static const String iconSet = 'Lucide';
  static const String signedInAs = 'ngoni';
  static const String version = '0.1.0 (1) \u00b7 patch 0';

  /// The themes in the Installed row; Kanso Dark is the one in use.
  static const List<ThemePreview> installedThemes = <ThemePreview>[
    kansoLight,
    kansoDark,
    nord,
  ];

  /// The themes the gallery offers to install.
  static const List<ThemePreview> galleryThemes = <ThemePreview>[
    tokyoNight,
    kanagawaWave,
    rosePine,
    catppuccinLatte,
    gruvboxDark,
  ];

  static const ThemePreview kansoLight = ThemePreview(
    file: 'kanso-light',
    name: 'Kanso Light',
    author: 'Kanso',
    license: 'MIT',
    source: 'https://github.com/iamngoni/kanso',
    appearance: ThemeAppearance.light,
    editor: Color(0xFFFCFAF4),
    text: Color(0xFF26241F),
    accent: Color(0xFF5E7895),
    heading: Color(0xFF26241F),
    marker: Color(0xFF857C6E),
    quote: Color(0xFF857C6E),
    link: Color(0xFF4D6684),
    url: Color(0xFF857C6E),
    code: Color(0xFF7D5B32),
    strong: Color(0xFF26241F),
    emphasis: Color(0xFF26241F),
    builtIn: true,
  );

  static const ThemePreview kansoDark = ThemePreview(
    file: 'kanso-dark',
    name: 'Kanso Dark',
    author: 'Kanso',
    license: 'MIT',
    source: 'https://github.com/iamngoni/kanso',
    appearance: ThemeAppearance.dark,
    editor: Color(0xFF282824),
    text: Color(0xFFF1EFE7),
    accent: Color(0xFF7E96B3),
    heading: Color(0xFFF1EFE7),
    marker: Color(0xFF7E7A70),
    quote: Color(0xFF7E7A70),
    link: Color(0xFF869DB9),
    url: Color(0xFF7E7A70),
    code: Color(0xFFCDB892),
    strong: Color(0xFFF1EFE7),
    emphasis: Color(0xFFF1EFE7),
    builtIn: true,
  );

  static const ThemePreview nord = ThemePreview(
    file: 'nord',
    name: 'Nord',
    author: 'Arctic Ice Studio & Sven Greb',
    license: 'MIT',
    source: 'https://github.com/nordtheme/nord',
    appearance: ThemeAppearance.dark,
    editor: Color(0xFF2E3440),
    text: Color(0xFFD8DEE9),
    accent: Color(0xFF88C0D0),
    heading: Color(0xFF88C0D0),
    marker: Color(0xFF737F97),
    quote: Color(0xFF737F97),
    link: Color(0xFF88C0D0),
    url: Color(0xFF81A1C1),
    code: Color(0xFF8FBCBB),
    strong: Color(0xFFD8DEE9),
    emphasis: Color(0xFFD8DEE9),
  );

  static const ThemePreview tokyoNight = ThemePreview(
    file: 'tokyo-night',
    name: 'Tokyo Night',
    author: 'enkia',
    license: 'MIT',
    source: 'https://github.com/enkia/tokyo-night-vscode-theme',
    appearance: ThemeAppearance.dark,
    editor: Color(0xFF1A1B26),
    text: Color(0xFFA9B1D6),
    accent: Color(0xFF7AA2F7),
    heading: Color(0xFF89DDFF),
    marker: Color(0xFF5E6589),
    quote: Color(0xFF9AA5CE),
    link: Color(0xFF73DACA),
    url: Color(0xFF5C658E),
    code: Color(0xFFBB9AF7),
    strong: Color(0xFFC0CAF5),
    emphasis: Color(0xFFC0CAF5),
  );

  static const ThemePreview kanagawaWave = ThemePreview(
    file: 'kanagawa-wave',
    name: 'Kanagawa Wave',
    author: 'rebelot',
    license: 'MIT',
    source: 'https://github.com/rebelot/kanagawa.nvim',
    appearance: ThemeAppearance.dark,
    editor: Color(0xFF1F1F28),
    text: Color(0xFFDCD7BA),
    accent: Color(0xFF7E9CD8),
    heading: Color(0xFF7E9CD8),
    marker: Color(0xFF9CABCA),
    quote: Color(0xFFB8B4D0),
    link: Color(0xFF7FB4CA),
    url: Color(0xFF938AA9),
    code: Color(0xFF98BB6C),
    strong: Color(0xFFDCD7BA),
    emphasis: Color(0xFFDCD7BA),
  );

  static const ThemePreview rosePine = ThemePreview(
    file: 'rose-pine',
    name: 'Rosé Pine',
    author: 'Rosé Pine',
    license: 'MIT',
    source: 'https://github.com/rose-pine/rose-pine-theme',
    appearance: ThemeAppearance.dark,
    editor: Color(0xFF191724),
    text: Color(0xFFE0DEF4),
    accent: Color(0xFFEBBCBA),
    heading: Color(0xFFC4A7E7),
    marker: Color(0xFF908CAA),
    quote: Color(0xFF908CAA),
    link: Color(0xFF9CCFD8),
    url: Color(0xFFC4A7E7),
    code: Color(0xFFF6C177),
    strong: Color(0xFFE0DEF4),
    emphasis: Color(0xFFE0DEF4),
  );

  static const ThemePreview catppuccinLatte = ThemePreview(
    file: 'catppuccin-latte',
    name: 'Catppuccin Latte',
    author: 'Catppuccin',
    license: 'MIT',
    source: 'https://github.com/catppuccin/catppuccin',
    appearance: ThemeAppearance.light,
    editor: Color(0xFFEFF1F5),
    text: Color(0xFF4C4F69),
    accent: Color(0xFF8839EF),
    heading: Color(0xFF1E66F5),
    marker: Color(0xFF7C7F93),
    quote: Color(0xFFC26CB2),
    link: Color(0xFF6E81EE),
    url: Color(0xFF1E66F5),
    code: Color(0xFF409E2D),
    strong: Color(0xFF4C4F69),
    emphasis: Color(0xFF4C4F69),
  );

  static const ThemePreview gruvboxDark = ThemePreview(
    file: 'gruvbox-dark',
    name: 'Gruvbox Dark',
    author: 'Pavel Pertsev (morhetz)',
    license: 'MIT',
    source: 'https://github.com/morhetz/gruvbox',
    appearance: ThemeAppearance.dark,
    editor: Color(0xFF282828),
    text: Color(0xFFEBDBB2),
    accent: Color(0xFFFE8019),
    heading: Color(0xFFB8BB26),
    marker: Color(0xFF928374),
    quote: Color(0xFF928374),
    link: Color(0xFF83A598),
    url: Color(0xFFD3869B),
    code: Color(0xFF8EC07C),
    strong: Color(0xFFEBDBB2),
    emphasis: Color(0xFFBDAE93),
  );
}
