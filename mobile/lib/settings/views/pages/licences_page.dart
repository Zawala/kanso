// Kanso — licences_page.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../core/utils/context_extensions.dart';
import '../../../injection.dart';
import '../../state/licences/licences_bloc.dart';
import '../widgets/licences_pane.dart';

/// Settings › Licences: what the app is made of.
class LicencesPage extends StatelessWidget {
  const LicencesPage({super.key});

  @override
  Widget build(BuildContext context) {
    return BlocProvider<LicencesBloc>(
      create: (context) =>
          getIt<LicencesBloc>()..add(const LicencesEvent.started()),
      child: Scaffold(
        resizeToAvoidBottomInset: false,
        backgroundColor: context.colors.app,
        body: const LicencesPane(),
      ),
    );
  }
}
