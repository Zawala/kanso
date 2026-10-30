// Kanso — preview_list_marker.dart
// Created 2026-10-30.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/widgets.dart';

import 'preview_list.dart';
import 'preview_style.dart';

/// A list item's bullet or number, [text], as the page draws it: right
/// aligned in a column a marker wide beside the item's text, in the quiet
/// colour of a ticked task.
class PreviewListMarker extends StatelessWidget {
  const PreviewListMarker(this.text, {super.key});

  final String text;

  @override
  Widget build(BuildContext context) {
    final style = PreviewScope.of(context).style;
    return SizedBox(
      width: PreviewList.markerWidth(style),
      child: Padding(
        padding: EdgeInsets.only(right: 0.35 * style.size),
        child: Text(
          text,
          textAlign: TextAlign.right,
          style: style.body.copyWith(color: style.taskDone),
        ),
      ),
    );
  }
}
