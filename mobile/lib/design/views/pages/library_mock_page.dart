// Kanso — library_mock_page.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/configs/kanso_effects.dart';
import '../../../core/configs/kanso_sizes.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_divider.dart';
import '../../../core/views/widgets/kanso_scrim.dart';
import '../../models/data/design_layout.dart';
import '../widgets/library_pane.dart';
import '../widgets/note_list_pane.dart';

/// The library on a phone: a drawer 85% of the width over the dimmed note
/// list. With [scrolled] the drawer is shown at its foot.
class LibraryMockPage extends StatelessWidget {
  const LibraryMockPage({this.scrolled = false, super.key});

  final bool scrolled;

  @override
  Widget build(BuildContext context) {
    final width = MediaQuery.sizeOf(context).width * KansoSizes.drawerFraction;
    return Scaffold(
      resizeToAvoidBottomInset: false,
      body: Stack(
        children: <Widget>[
          const Positioned.fill(
            child: NoteListPane(layout: DesignLayout.phone),
          ),
          const Positioned.fill(child: KansoScrim()),
          Align(
            alignment: AlignmentDirectional.centerStart,
            child: Container(
              width: width,
              decoration: BoxDecoration(
                boxShadow: <BoxShadow>[
                  BoxShadow(
                    color: Colors.black.withValues(alpha: KansoEffects.shadow),
                    blurRadius: 32,
                  ),
                ],
              ),
              child: Row(
                children: <Widget>[
                  Expanded(child: LibraryPane(startAtEnd: scrolled)),
                  const KansoDivider(vertical: true),
                ],
              ),
            ),
          ),
        ],
      ),
      backgroundColor: context.colors.sidebar,
    );
  }
}
