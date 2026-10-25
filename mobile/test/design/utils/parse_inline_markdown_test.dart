// Kanso — parse_inline_markdown_test.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter_test/flutter_test.dart';
import 'package:glados/glados.dart' show Glados, any;
import 'package:kanso/design/utils/parse_inline_markdown.dart';

import '../../helpers/property.dart';

void main() {
  group('parseInlineMarkdown properties', () {
    Glados<String>(any.markdownish, kansoExploreConfig()).test(
      'the runs joined give back the source',
      (source) {
        expect(parseInlineMarkdown(source).map((r) => r.text).join(), source);
      },
    );

    Glados<String>(any.markdownish, kansoExploreConfig()).test(
      'no run is empty',
      (source) {
        expect(
          parseInlineMarkdown(source).every((run) => run.text.isNotEmpty),
          isTrue,
        );
      },
    );

    Glados<String>(any.markdownish, kansoExploreConfig()).test(
      'every styled run sits between marks',
      (source) {
        final runs = parseInlineMarkdown(source);
        const styled = <InlineKind>{
          InlineKind.bold,
          InlineKind.italic,
          InlineKind.code,
          InlineKind.wiki,
          InlineKind.link,
        };
        for (var i = 0; i < runs.length; i++) {
          if (styled.contains(runs[i].kind)) {
            expect(runs[i - 1].kind, InlineKind.marker);
            expect(runs[i + 1].kind, InlineKind.marker);
          }
        }
      },
    );

    Glados<String>(any.markdownish, kansoExploreConfig()).test(
      'a source with none of the marks is one plain run',
      (source) {
        final bare = source.replaceAll(RegExp(r'[*`\[\]()]'), '');
        final runs = parseInlineMarkdown(bare);
        expect(runs.length, bare.isEmpty ? 0 : 1);
        if (bare.isNotEmpty) {
          expect(runs.single.kind, InlineKind.plain);
        }
      },
    );
  });

  group('parseInlineMarkdown examples', () {
    List<(String, InlineKind)> kinds(String source) => <(String, InlineKind)>[
      for (final run in parseInlineMarkdown(source)) (run.text, run.kind),
    ];

    test('bold, italic and code', () {
      expect(kinds('a **b** *c* `d`'), <(String, InlineKind)>[
        ('a ', InlineKind.plain),
        ('**', InlineKind.marker),
        ('b', InlineKind.bold),
        ('**', InlineKind.marker),
        (' ', InlineKind.plain),
        ('*', InlineKind.marker),
        ('c', InlineKind.italic),
        ('*', InlineKind.marker),
        (' ', InlineKind.plain),
        ('`', InlineKind.marker),
        ('d', InlineKind.code),
        ('`', InlineKind.marker),
      ]);
    });

    test('a link and a wiki link', () {
      expect(kinds('[t](u) [[w]]'), <(String, InlineKind)>[
        ('[', InlineKind.marker),
        ('t', InlineKind.link),
        ('](', InlineKind.marker),
        ('u', InlineKind.url),
        (')', InlineKind.marker),
        (' ', InlineKind.plain),
        ('[[', InlineKind.marker),
        ('w', InlineKind.wiki),
        (']]', InlineKind.marker),
      ]);
    });

    test('a mark with no partner is plain text', () {
      expect(kinds('a * b'), <(String, InlineKind)>[
        ('a * b', InlineKind.plain),
      ]);
    });
  });
}
