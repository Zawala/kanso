// Kanso — note_pdf_exporter_test.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:io';

import 'package:dartz/dartz.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/configs/kanso_palette.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/core/models/repos/abs/renderer_repository.dart';
import 'package:kanso/core/services/preview_theme_fitter.dart';
import 'package:kanso/note/state/export/note_pdf_exporter.dart';
import 'package:kanso_print/kanso_print.dart';
import 'package:mocktail/mocktail.dart';

import '../../../helpers/seeded_appearance_bloc.dart' show MockThemesRepository;
import '../../../helpers/theme_tokens.dart';

/// The themes chosen: a gallery theme for the syntax and the preview.
const PreviewThemes _picked = (
  ui: 'kanso-light',
  syntax: 'gallery-dracula',
  preview: 'gallery-paper',
  dark: false,
);

const _failure = ApplicationError.storage(message: 'Disk error.');

/// Answers the call an export makes to draw the page, and records what it
/// was asked.
class _FakeRenderer extends Fake implements RendererRepository {
  _FakeRenderer({this.page});

  final Either<ApplicationError, ExportedPage>? page;

  final List<({String id, PaletteDto palette, PreviewOptions options})> drawed =
      <({String id, PaletteDto palette, PreviewOptions options})>[];

  @override
  Future<Either<ApplicationError, ExportedPage>> pagePdf(
    String id,
    PaletteDto palette,
    PreviewOptions options,
  ) async {
    drawed.add((id: id, palette: palette, options: options));
    return page!;
  }
}

void main() {
  final tokens = tokensFor(fakePalette('gallery-paper', dark: true));
  final own = tokensFor(KansoPalette.kansoLight);
  const page = ExportedPage(html: '<p>Hello</p>', fileName: 'Hello.pdf');

  late MockThemesRepository themes;

  setUp(() {
    themes = MockThemesRepository();
    when(() => themes.fit(any(), any(), any())).thenAnswer(
      (_) async => Right<ApplicationError, ThemeTokens>(tokens),
    );
  });

  NotePdfExporter exporter(
    _FakeRenderer renderer, {
    PdfPrinter? print,
    String locale = 'en_GB',
  }) => NotePdfExporter(
    renderer,
    PreviewThemeFitter(themes),
    folder: () async => Directory('/tmp/kanso-export-test'),
    print: print ?? ({required html, required path, required paper}) async => 1,
    locale: locale,
  );

  test('a locale name gives its country, or none', () {
    expect(countryOf('en_GB'), 'GB');
    expect(countryOf('en-GB'), 'GB');
    expect(countryOf('pt_BR'), 'BR');
    expect(countryOf('en'), isNull);
    expect(countryOf('C'), isNull);
  });

  test('the paper follows the country, A4 elsewhere', () {
    expect(exporter(_FakeRenderer(), locale: 'en_US').paper, KansoPaper.letter);
    expect(exporter(_FakeRenderer()).paper, KansoPaper.a4);
    expect(exporter(_FakeRenderer(), locale: 'en').paper, KansoPaper.a4);
  });

  test('the note is written in its theme, with its page name', () async {
    final renderer = _FakeRenderer(
      page: const Right<ApplicationError, ExportedPage>(page),
    );
    final printed = <({String html, String path, KansoPaper paper})>[];

    final written = await exporter(
      renderer,
      print: ({required html, required path, required paper}) async {
        printed.add((html: html, path: path, paper: paper));
        return 2;
      },
      locale: 'en_US',
    ).export(noteId: 'note:1', themes: _picked);

    expect(
      written,
      const Right<ApplicationError, String>('/tmp/kanso-export-test/Hello.pdf'),
    );
    expect(printed.single.html, '<p>Hello</p>');
    expect(printed.single.paper, KansoPaper.letter);
    expect(renderer.drawed.single.id, 'note:1');
    expect(renderer.drawed.single.palette, tokens.palette);
    expect(renderer.drawed.single.options.code, tokens.syntax);
    expect(renderer.drawed.single.options.utcOffsetSecs, 0);
  });

  test('the interface, syntax and preview themes are fitted together, '
      'gallery themes included', () async {
    await exporter(
      _FakeRenderer(page: const Right<ApplicationError, ExportedPage>(page)),
    ).export(noteId: 'note:1', themes: _picked);

    verify(
      () => themes.fit('kanso-light', 'gallery-dracula', 'gallery-paper'),
    ).called(1);
  });

  test("themes that can't be read fall back to Kanso's own", () async {
    when(
      () => themes.fit('kanso-light', 'gallery-dracula', 'gallery-paper'),
    ).thenAnswer(
      (_) async => const Left<ApplicationError, ThemeTokens>(_failure),
    );
    when(
      () => themes.fit('kanso-light', 'kanso-light', 'kanso-light'),
    ).thenAnswer((_) async => Right<ApplicationError, ThemeTokens>(own));
    final renderer = _FakeRenderer(
      page: const Right<ApplicationError, ExportedPage>(page),
    );

    final written = await exporter(
      renderer,
    ).export(noteId: 'note:1', themes: _picked);

    expect(written.isRight(), isTrue);
    expect(renderer.drawed.single.palette, own.palette);
    expect(renderer.drawed.single.options.code, own.syntax);
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
      expect(renderer.drawed, isEmpty);
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

  test(
    'a writer that fails is a storage error, and nothing to share',
    () async {
      final written = await exporter(
        _FakeRenderer(page: const Right<ApplicationError, ExportedPage>(page)),
        print: ({required html, required path, required paper}) async =>
            throw const FileSystemException('no room'),
      ).export(noteId: 'note:1', themes: _picked);

      expect(written.isLeft(), isTrue);
      expect(
        written.fold((error) => error, (_) => fail('a path')),
        isA<StorageError>(),
      );
    },
  );
}
