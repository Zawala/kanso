// Kanso — fixture_renderer_repository.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';
import 'package:flutter_test/flutter_test.dart' show Fake;
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/core/models/repos/abs/renderer_repository.dart';

/// The colours of the runs [FixtureRendererRepository] makes.
abstract final class FixtureRunColors {
  static const int heading = 0x8A4B14;
  static const int code = 0x2F7469;
  static const int quote = 0x6B6258;
  static const int marker = 0x9A8F80;
  static const int link = 0x2F5F9E;
}

/// A renderer that styles Markdown the plain way, for tests that must not
/// depend on the Rust core: headings, fenced code, quotes, list markers,
/// bold, inline code and note links. Every other call is an error.
class FixtureRendererRepository extends Fake implements RendererRepository {
  /// The themes it was asked for, in order.
  final List<String> themes = <String>[];

  /// What it answers instead of runs, to test a text that cannot be styled.
  ApplicationError? failure;

  @override
  Future<Either<ApplicationError, List<StyledRun>>> styledRuns(
    String text, {
    required int start,
    required int end,
    required String themeId,
  }) async {
    themes.add(themeId);
    final error = failure;
    if (error != null) {
      return Left<ApplicationError, List<StyledRun>>(error);
    }
    return Right<ApplicationError, List<StyledRun>>(
      fixtureRuns(text.substring(start, end), offset: start),
    );
  }
}

/// The runs a plain pass over [text] makes, starting at [offset].
List<StyledRun> fixtureRuns(String text, {int offset = 0}) {
  final runs = <StyledRun>[];
  var at = 0;
  var fenced = false;
  for (final line in text.split('\n')) {
    final start = offset + at;
    final end = start + line.length;
    if (line.startsWith('```')) {
      fenced = !fenced;
      runs.add(_run(start, end, FixtureRunColors.marker));
    } else if (fenced) {
      runs.add(_run(start, end, FixtureRunColors.code));
    } else if (line.startsWith('#')) {
      runs.add(_run(start, end, FixtureRunColors.heading, weight: 700));
    } else if (line.startsWith('>')) {
      runs.add(_run(start, end, FixtureRunColors.quote, italic: true));
    } else {
      for (final match in RegExp(r'\*\*[^*]+\*\*').allMatches(line)) {
        runs.add(
          _run(start + match.start, start + match.end, null, weight: 700),
        );
      }
      for (final match in RegExp('`[^`]+`').allMatches(line)) {
        runs.add(
          _run(start + match.start, start + match.end, FixtureRunColors.code),
        );
      }
      for (final match in RegExp(r'\[\[[^\]]+\]\]').allMatches(line)) {
        runs.add(
          _run(start + match.start, start + match.end, FixtureRunColors.link),
        );
      }
    }
    at += line.length + 1;
  }
  return runs;
}

StyledRun _run(int start, int end, int? color, {bool? italic, int? weight}) {
  return StyledRun(
    range: Range16(start: start, end: end),
    color: color,
    italic: italic,
    weight: weight,
  );
}
