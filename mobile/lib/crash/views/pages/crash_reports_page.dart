// Kanso — crash_reports_page.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_page.dart';
import '../../../injection.dart';
import '../../state/crash_reports/crash_reports_bloc.dart';
import '../widgets/crash_reports_pane.dart';

/// Settings › Privacy › Crash reports.
class CrashReportsPage extends StatelessWidget {
  const CrashReportsPage({super.key});

  @override
  Widget build(BuildContext context) {
    return BlocProvider<CrashReportsBloc>(
      create: (context) =>
          getIt<CrashReportsBloc>()..add(const CrashReportsEvent.started()),
      child: KansoPage(
        resizeToAvoidBottomInset: false,
        backgroundColor: context.colors.app,
        body: const CrashReportsPane(),
      ),
    );
  }
}
