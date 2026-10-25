// Kanso — providers.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/widgets.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import 'core/state/appearance/appearance_bloc.dart';
import 'injection.dart';

/// The BLoCs the whole app shares, above the [child].
class AppProviders extends StatelessWidget {
  const AppProviders({required this.child, super.key});

  final Widget child;

  @override
  Widget build(BuildContext context) {
    return MultiBlocProvider(
      providers: <BlocProvider<dynamic>>[
        BlocProvider<AppearanceBloc>(
          create: (_) =>
              getIt<AppearanceBloc>()..add(const AppearanceEvent.started()),
        ),
      ],
      child: child,
    );
  }
}
