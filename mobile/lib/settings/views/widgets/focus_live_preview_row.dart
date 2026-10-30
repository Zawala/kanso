// Kanso — focus_live_preview_row.dart
// Created 2026-10-30.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../core/utils/context_extensions.dart';
import '../../../shell/state/focus_mode/focus_mode_bloc.dart';
import 'discovery_setting_switch_row.dart';

/// Settings › Writing: whether focus mode shows the open note in Live, as
/// the `FocusModeBloc` above it has it.
class FocusLivePreviewRow extends StatelessWidget {
  const FocusLivePreviewRow({super.key});

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    return BlocBuilder<FocusModeBloc, FocusModeState>(
      buildWhen: (before, after) => before.livePreview != after.livePreview,
      builder: (context, focus) => DiscoverySettingSwitchRow(
        title: l10n.editorFocusLivePreview,
        detail: l10n.editorFocusLivePreviewDetail,
        value: focus.livePreview,
        onChanged: (enabled) => context.read<FocusModeBloc>().add(
          FocusModeEvent.livePreviewChanged(enabled: enabled),
        ),
      ),
    );
  }
}
