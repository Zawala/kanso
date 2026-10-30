// Kanso — kanso_text_size.dart
// Created 2026-10-29.
// Licensed under AGPL-3.0-or-later (see LICENSE).

/// The minimum text scale the note size setting can store.
const double minimumTextScale = 0.8;

/// The maximum text scale the note size setting can store.
const double maximumTextScale = 1.8;

/// The step between adjacent values offered by the note size setting.
const double textScaleStep = 0.05;

/// The minimum scale the interface text size setting can store.
const double minimumInterfaceTextScale = 0.9;

/// The maximum scale the interface text size setting can store.
const double maximumInterfaceTextScale = 1.3;

/// The step between adjacent values offered by the interface size setting.
const double interfaceTextScaleStep = 0.05;

/// Rounds [scale] to a supported step and keeps it inside the setting bounds.
/// Invalid values use the designed size, 1.0.
double normalizeTextScale(double scale) {
  if (!scale.isFinite) {
    return 1;
  }
  final bounded = scale.clamp(minimumTextScale, maximumTextScale);
  return ((bounded * 20).round() / 20).clamp(
    minimumTextScale,
    maximumTextScale,
  );
}

/// Rounds [scale] to the interface setting's supported step and keeps it
/// inside its narrower bounds. Invalid values use the designed size, 1.0.
double normalizeInterfaceTextScale(double scale) {
  if (!scale.isFinite) {
    return 1;
  }
  final bounded = scale.clamp(
    minimumInterfaceTextScale,
    maximumInterfaceTextScale,
  );
  final steps = ((bounded - minimumInterfaceTextScale) / interfaceTextScaleStep)
      .round();
  final normalized = minimumInterfaceTextScale + steps * interfaceTextScaleStep;
  return double.parse(
    normalized
        .clamp(minimumInterfaceTextScale, maximumInterfaceTextScale)
        .toStringAsFixed(2),
  );
}

/// Reads current numeric scales and the names written by earlier versions.
double textScaleFromJson(Object? value) {
  if (value is num) {
    return normalizeTextScale(value.toDouble());
  }
  return switch (value) {
    'small' => 0.9,
    'standard' => 1,
    'large' => 1.15,
    'larger' => 1.3,
    _ => 1,
  };
}

/// Writes scales as normalized numeric JSON values.
double textScaleToJson(double scale) => normalizeTextScale(scale);

/// Reads an interface scale from JSON, including a value written by a newer
/// version outside this build's range.
double interfaceTextScaleFromJson(Object? value) {
  if (value is num) {
    return normalizeInterfaceTextScale(value.toDouble());
  }
  return 1;
}

/// Writes interface scales as normalized numeric JSON values.
double interfaceTextScaleToJson(double scale) =>
    normalizeInterfaceTextScale(scale);
