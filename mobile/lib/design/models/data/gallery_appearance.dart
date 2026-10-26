// Kanso — gallery_appearance.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import '../../../core/state/appearance/appearance_bloc.dart';

/// The three ways the gallery's control sets the app's look: the system's
/// light or dark mode, or Kanso Light or Kanso Dark for good.
enum GalleryAppearance {
  system,
  light,
  dark;

  /// The way the app is set in [state].
  static GalleryAppearance of(AppearanceState state) {
    if (state.selection.followSystem) {
      return GalleryAppearance.system;
    }
    return state.light.dark ? GalleryAppearance.dark : GalleryAppearance.light;
  }

  /// The events that set the app this way.
  List<AppearanceEvent> get events => switch (this) {
    GalleryAppearance.system => <AppearanceEvent>[
      const AppearanceEvent.followSystemChanged(follow: true),
    ],
    GalleryAppearance.light => <AppearanceEvent>[
      const AppearanceEvent.followSystemChanged(follow: false),
      const AppearanceEvent.themePicked('kanso-light', dark: false),
    ],
    GalleryAppearance.dark => <AppearanceEvent>[
      const AppearanceEvent.followSystemChanged(follow: false),
      const AppearanceEvent.themePicked('kanso-dark', dark: true),
    ],
  };
}
