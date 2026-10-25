// Kanso — error_mock_page.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/configs/kanso_spacing.dart';
import '../../../core/views/widgets/kanso_scrim.dart';
import '../../models/data/design_layout.dart';
import '../widgets/note_list_pane.dart';
import '../widgets/signed_out_card.dart';

/// The signed-out error card on a phone: the note list dimmed behind a
/// small centred card with one clear action.
class ErrorMockPage extends StatelessWidget {
  const ErrorMockPage({super.key});

  @override
  Widget build(BuildContext context) {
    return const Scaffold(
      resizeToAvoidBottomInset: false,
      body: Stack(
        children: <Widget>[
          Positioned.fill(child: NoteListPane(layout: DesignLayout.phone)),
          Positioned.fill(child: KansoScrim()),
          Center(
            child: Padding(
              padding: EdgeInsets.all(KansoSpacing.lg),
              child: SignedOutCard(),
            ),
          ),
        ],
      ),
    );
  }
}
