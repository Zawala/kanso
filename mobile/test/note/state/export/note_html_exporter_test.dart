// Kanso — note_html_exporter_test.dart
// Created 2026-10-28.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:io';

import 'package:dartz/dartz.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/configs/kanso_palette.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/core/models/repos/abs/renderer_repository.dart';
import 'package:kanso/core/services/preview_theme_fitter.dart';
import 'package:kanso/note/state/export/note_html_exporter.dart';
import 'package:kanso/note/state/preview/preview_bloc.dart';
import 'package:mocktail/mocktail.dart';

import '../../../helpers/seeded_appearance_bloc.dart' show MockThemesRepository;
import '../../../helpers/theme_tokens.dart';

/// The themes chosen: a gallery theme for the syntax and the preview.
const PreviewThemes _picked = (
  ui: 'nord',
  syntax: 'gallery-dracula',
  preview: 'gallery-paper',
  dark: true,
);

const _failure = ApplicationError.storage(message: 'Disk error.');

/// Answers the call an export makes to draw the page, and records what it
/// was asked.
class _FakeRenderer extends Fake implements RendererRepository {
  _FakeRenderer({this.page});

  final Either<ApplicationError, ExportedPage>? page;

  final List<({String id, PaletteDto palette, PreviewOptions options})> drawn =
      <({String id, PaletteDto palette, PreviewOptions options})>[];

  @override
  Future<Either<ApplicationError, ExportedPage>> pageHtml(
    String id,
    PaletteDto palette,
    PreviewOptions options,
  ) async {
    drawn.add((id: id, palette: palette, options: options));
    return page!;
  }
}

void main() {
  final tokens = tokensFor(fakePalette('nord'));
  final own = tokensFor(KansoPalette.kansoDark);
  const page = ExportedPage(html: '<p>Hello</p>', fileName: 'Hello.html');

  late Directory folder;
  late MockThemesRepository themes;

  setUp(() async {
    folder = await Directory.systemTemp.createTemp('kanso-html-export');
    themes = MockThemesRepository();
    when(() => themes.fit(any(), any(), any())).thenAnswer(
      (_) async => Right<ApplicationError, ThemeTokens>(tokens),
    );
  });

  tearDown(() async {
    if (folder.existsSync()) {
      await folder.delete(recursive: true);
    }
  });

  NoteHtmlExporter exporter(_FakeRenderer renderer, {Directory? into}) =>
      NoteHtmlExporter(
        renderer,
        PreviewThemeFitter(themes),
        folder: () async => into ?? folder,
      );

  test('the note is written in its theme, with its page name', () async {
    final renderer = _FakeRenderer(
      page: const Right<ApplicationError, ExportedPage>(page),
    );

    final written = await exporter(
      renderer,
    ).export(noteId: 'note:1', themes: _picked);

    final path = written.fold((_) => fail('a path'), (path) => path);
    expect(path, '${folder.path}/Hello.html');
    expect(File(path).readAsStringSync(), '<p>Hello</p>');
    expect(renderer.drawn.single.id, 'note:1');
    expect(renderer.drawn.single.palette, tokens.palette);
    expect(renderer.drawn.single.options.code, tokens.syntax);
    expect(renderer.drawn.single.options.size, previewTextSize);
    expect(renderer.drawn.single.options.utcOffsetSecs, 0);
  });

  test('the interface, syntax and preview themes are fitted together, '
      'gallery themes included', () async {
    await exporter(
      _FakeRenderer(page: const Right<ApplicationError, ExportedPage>(page)),
    ).export(noteId: 'note:1', themes: _picked);

    verify(
      () => themes.fit('nord', 'gallery-dracula', 'gallery-paper'),
    ).called(1);
  });

  test("themes that can't be read fall back to Kanso's own", () async {
    when(
      () => themes.fit('nord', 'gallery-dracula', 'gallery-paper'),
    ).thenAnswer(
      (_) async => const Left<ApplicationError, ThemeTokens>(_failure),
    );
    when(() => themes.fit('kanso-dark', 'kanso-dark', 'kanso-dark')).thenAnswer(
      (_) async => Right<ApplicationError, ThemeTokens>(own),
    );
    final renderer = _FakeRenderer(
      page: const Right<ApplicationError, ExportedPage>(page),
    );

    final written = await exporter(
      renderer,
    ).export(noteId: 'note:1', themes: _picked);

    expect(written.isRight(), isTrue);
    expect(renderer.drawn.single.palette, own.palette);
    expect(renderer.drawn.single.options.code, own.syntax);
  });

  test(
    'no theme that can be read is the error, and nothing is drawn',
    () async {
      when(() => themes.fit(any(), any(), any())).thenAnswer(
        (_) async => const Left<ApplicationError, ThemeTokens>(_failure),
      );
      final renderer = _FakeRenderer(
        page: const Right<ApplicationError, ExportedPage>(page),
      );

      expect(
        await exporter(renderer).export(noteId: 'note:1', themes: _picked),
        const Left<ApplicationError, String>(_failure),
      );
      expect(renderer.drawn, isEmpty);
    },
  );

  test('a locked note is the error it came as', () async {
    const error = ApplicationError.locked(message: 'sealed');

    expect(
      await exporter(
        _FakeRenderer(page: const Left<ApplicationError, ExportedPage>(error)),
      ).export(noteId: 'note:1', themes: _picked),
      const Left<ApplicationError, String>(error),
    );
  });

  test('a folder that cannot be written is a storage error', () async {
    final written = await exporter(
      _FakeRenderer(page: const Right<ApplicationError, ExportedPage>(page)),
      into: Directory('${folder.path}/gone'),
    ).export(noteId: 'note:1', themes: _picked);

    expect(written.isLeft(), isTrue);
    expect(
      written.fold((error) => error, (_) => fail('a path')),
      isA<StorageError>(),
    );
  });
}
