// Kanso — parse_inline_markdown.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

/// What a run of inline Markdown is.
enum InlineKind {
  /// Ordinary text.
  plain,

  /// Text between `**` marks.
  bold,

  /// Text between `*` marks.
  italic,

  /// Text between backticks.
  code,

  /// The text of a `[text](url)` link.
  link,

  /// The address of a link.
  url,

  /// The title inside `[[` and `]]`.
  wiki,

  /// A mark itself: `**`, `*`, a backtick, `[`, `](`, `)`, `[[`, `]]`.
  marker,
}

/// A run of text and what it is.
class InlineRun {
  const InlineRun(this.text, this.kind);

  final String text;
  final InlineKind kind;
}

/// [source] cut into runs of inline Markdown, marks included.
///
/// Understands `**bold**`, `*italic*`, `` `code` ``, `[text](url)` and
/// `[[wiki link]]`, one level deep. A mark with no partner is plain text.
/// The runs joined give back [source] and none is empty; a renderer that
/// hides the marks drops the [InlineKind.marker] and [InlineKind.url] runs.
List<InlineRun> parseInlineMarkdown(String source) {
  final runs = <InlineRun>[];
  final plain = StringBuffer();

  void flush() {
    if (plain.isNotEmpty) {
      runs.add(InlineRun(plain.toString(), InlineKind.plain));
      plain.clear();
    }
  }

  void add(String mark, String inner, InlineKind kind, String close) {
    flush();
    runs
      ..add(InlineRun(mark, InlineKind.marker))
      ..add(InlineRun(inner, kind))
      ..add(InlineRun(close, InlineKind.marker));
  }

  var i = 0;
  while (i < source.length) {
    if (source.startsWith('[[', i)) {
      final end = source.indexOf(']]', i + 2);
      if (end > i + 2) {
        add('[[', source.substring(i + 2, end), InlineKind.wiki, ']]');
        i = end + 2;
        continue;
      }
    } else if (source.startsWith('**', i)) {
      final end = source.indexOf('**', i + 2);
      if (end > i + 2) {
        add('**', source.substring(i + 2, end), InlineKind.bold, '**');
        i = end + 2;
        continue;
      }
    } else if (source.startsWith('`', i)) {
      final end = source.indexOf('`', i + 1);
      if (end > i + 1) {
        add('`', source.substring(i + 1, end), InlineKind.code, '`');
        i = end + 1;
        continue;
      }
    } else if (source.startsWith('*', i)) {
      final end = source.indexOf('*', i + 1);
      if (end > i + 1) {
        add('*', source.substring(i + 1, end), InlineKind.italic, '*');
        i = end + 1;
        continue;
      }
    } else if (source.startsWith('[', i)) {
      final middle = source.indexOf('](', i + 1);
      final end = middle < 0 ? -1 : source.indexOf(')', middle + 2);
      if (middle > i + 1 && end > middle + 2) {
        flush();
        runs
          ..add(const InlineRun('[', InlineKind.marker))
          ..add(InlineRun(source.substring(i + 1, middle), InlineKind.link))
          ..add(const InlineRun('](', InlineKind.marker))
          ..add(InlineRun(source.substring(middle + 2, end), InlineKind.url))
          ..add(const InlineRun(')', InlineKind.marker));
        i = end + 1;
        continue;
      }
    }
    plain.write(source[i]);
    i++;
  }
  flush();
  return runs;
}
