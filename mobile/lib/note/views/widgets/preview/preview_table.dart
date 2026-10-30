// Kanso — preview_table.dart
// Created 2026-10-30.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:math' as math;

import 'package:flutter/widgets.dart';

import '../../../../core/bridge/api/types.dart' hide Table, TableRow;
import '../../../../core/utils/context_extensions.dart';
import 'preview_style.dart';
import 'preview_text.dart';

/// A table: ruled cells, the header row on its own background, scrolling
/// sideways when it is wider than the note.
class PreviewTable extends StatelessWidget {
  const PreviewTable({
    required this.aligns,
    required this.head,
    required this.rows,
    super.key,
  });

  final List<PreviewAlign> aligns;
  final List<PreviewCell> head;
  final List<PreviewRow> rows;

  @override
  Widget build(BuildContext context) {
    final style = PreviewScope.of(context).style;
    final text = style.sized(0.9333);
    final columns = rows.fold(
      math.max(head.length, aligns.length),
      (most, row) => math.max(most, row.cells.length),
    );
    TextAlign alignOf(int column) =>
        switch (column < aligns.length ? aligns[column] : PreviewAlign.none) {
          PreviewAlign.center => TextAlign.center,
          PreviewAlign.right => TextAlign.right,
          PreviewAlign.left || PreviewAlign.none => TextAlign.left,
        };
    TableRow row(List<PreviewCell> cells, {required bool header}) => TableRow(
      decoration: header ? BoxDecoration(color: style.tableHead) : null,
      children: <Widget>[
        for (var column = 0; column < columns; column++)
          Padding(
            padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 7),
            child: PreviewText(
              column < cells.length
                  ? cells[column].inlines
                  : const <PreviewInline>[],
              style: header ? text.copyWith(fontWeight: FontWeight.w600) : text,
              textAlign: alignOf(column),
            ),
          ),
      ],
    );
    return SingleChildScrollView(
      scrollDirection: Axis.horizontal,
      child: Table(
        defaultColumnWidth: const IntrinsicColumnWidth(),
        border: TableBorder.all(color: style.border),
        children: <TableRow>[
          if (head.isNotEmpty) row(head, header: true),
          for (final each in rows) row(each.cells, header: false),
        ],
      ),
    );
  }
}

/// A query view's table: each matching note's title, a link to it, and its
/// columns, set small and muted; "No notes match." when none does.
class PreviewQuery extends StatelessWidget {
  const PreviewQuery({required this.columns, required this.rows, super.key});

  final List<String> columns;
  final List<PreviewQueryRow> rows;

  @override
  Widget build(BuildContext context) {
    final scope = PreviewScope.of(context);
    final style = scope.style;
    final text = style.sized(0.8).copyWith(color: style.muted);
    final count = rows.fold(
      columns.length,
      (most, row) => math.max(most, row.cells.length + 1),
    );
    Widget cell(String value, TextStyle look) => Padding(
      padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 5),
      child: Text(
        value,
        style: look,
        maxLines: 1,
        softWrap: false,
        overflow: TextOverflow.ellipsis,
      ),
    );
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      mainAxisSize: MainAxisSize.min,
      children: <Widget>[
        if (count > 0)
          Table(
            columnWidths: const <int, TableColumnWidth>{
              0: FractionColumnWidth(0.34),
            },
            children: <TableRow>[
              TableRow(
                decoration: BoxDecoration(
                  border: Border(bottom: BorderSide(color: style.border)),
                ),
                children: <Widget>[
                  for (var at = 0; at < count; at++)
                    cell(
                      at < columns.length ? columns[at] : '',
                      text.copyWith(fontSize: (text.fontSize ?? 14) * 0.92),
                    ),
                ],
              ),
              for (final row in rows)
                TableRow(
                  children: <Widget>[
                    GestureDetector(
                      behavior: HitTestBehavior.opaque,
                      onTap: () => scope.onLink(row.url),
                      child: Semantics(
                        link: true,
                        child: cell(
                          row.title,
                          text.copyWith(
                            color: style.accent,
                            fontSize: (text.fontSize ?? 14) * 1.04,
                          ),
                        ),
                      ),
                    ),
                    for (var at = 1; at < count; at++)
                      cell(
                        at - 1 < row.cells.length ? row.cells[at - 1] : '',
                        text,
                      ),
                  ],
                ),
            ],
          ),
        if (rows.isEmpty)
          Padding(
            padding: const EdgeInsets.all(8),
            child: Text(context.l10n.previewQueryEmpty, style: text),
          ),
      ],
    );
  }
}
