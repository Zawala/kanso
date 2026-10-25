// Kanso — kanso_icon.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_svg/flutter_svg.dart';

import '../../configs/kanso_icons.dart';
import '../../utils/context_extensions.dart';

/// One of Kanso's [KansoIcons], drawn from its bundled SVG.
///
/// The colour defaults to the surrounding [IconTheme], then to the theme's
/// text colour. Pass [semanticLabel] when the icon carries meaning of its
/// own; otherwise it is hidden from screen readers.
class KansoIcon extends StatelessWidget {
  const KansoIcon(
    this.icon, {
    this.size = 20,
    this.color,
    this.semanticLabel,
    super.key,
  });

  final KansoIcons icon;
  final double size;
  final Color? color;
  final String? semanticLabel;

  @override
  Widget build(BuildContext context) {
    final resolved =
        color ?? IconTheme.of(context).color ?? context.colors.text;
    return SvgPicture.asset(
      icon.assetPath,
      width: size,
      height: size,
      colorFilter: ColorFilter.mode(resolved, BlendMode.srcIn),
      semanticsLabel: semanticLabel,
      excludeFromSemantics: semanticLabel == null,
    );
  }
}
