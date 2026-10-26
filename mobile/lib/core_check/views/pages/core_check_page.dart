// Kanso — core_check_page.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/widgets.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../injection.dart';
import '../../state/core_check/core_check_bloc.dart';
import '../widgets/core_check_view.dart';

/// The core check: real calls to the Rust library, shown on a page. This
/// is the composition boundary; [CoreCheckView] draws the state.
class CoreCheckPage extends StatelessWidget {
  const CoreCheckPage({super.key});

  @override
  Widget build(BuildContext context) {
    return BlocProvider<CoreCheckBloc>(
      create: (_) =>
          getIt<CoreCheckBloc>()..add(const CoreCheckEvent.started()),
      child: const CoreCheckView(),
    );
  }
}
