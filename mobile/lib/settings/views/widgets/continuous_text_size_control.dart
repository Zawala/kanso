// Kanso — continuous_text_size_control.dart
// Created 2026-10-29.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';

import '../../../core/models/data/kanso_text_size.dart';
import '../../../core/utils/context_extensions.dart';

/// An owned, keyboard accessible control for adjusting a text scale.
class ContinuousTextSizeControl extends StatelessWidget {
  const ContinuousTextSizeControl({
    required this.value,
    required this.onChanged,
    this.minimum = minimumTextScale,
    this.maximum = maximumTextScale,
    this.step = textScaleStep,
    this.semanticLabel,
    this.decreaseLabel,
    this.increaseLabel,
    super.key,
  });

  final double value;
  final ValueChanged<double> onChanged;
  final double minimum;
  final double maximum;
  final double step;
  final String? semanticLabel;
  final String? decreaseLabel;
  final String? increaseLabel;

  double get _scale => _normalize(value);

  double _normalize(double scale) {
    if (!scale.isFinite) {
      return 1;
    }
    final bounded = scale.clamp(minimum, maximum);
    final steps = ((bounded - minimum) / step).round();
    return double.parse(
      (minimum + steps * step).clamp(minimum, maximum).toStringAsFixed(2),
    );
  }

  void _adjust(double amount) => onChanged(_normalize(_scale + amount));

  void _setFromOffset(double dx, double width) {
    if (width <= 0) {
      return;
    }
    final fraction = (dx / width).clamp(0.0, 1.0);
    onChanged(_normalize(minimum + fraction * (maximum - minimum)));
  }

  int get _percent => (_scale * 100).round();

  int get _nextPercent =>
      ((_scale + step).clamp(minimum, maximum) * 100).round();

  int get _previousPercent =>
      ((_scale - step).clamp(minimum, maximum) * 100).round();

  KeyEventResult _onKey(FocusNode node, KeyEvent event) {
    if (event is! KeyDownEvent) {
      return KeyEventResult.ignored;
    }
    switch (event.logicalKey) {
      case LogicalKeyboardKey.arrowLeft:
      case LogicalKeyboardKey.arrowDown:
      case LogicalKeyboardKey.minus:
        _adjust(-step);
        return KeyEventResult.handled;
      case LogicalKeyboardKey.arrowRight:
      case LogicalKeyboardKey.arrowUp:
      case LogicalKeyboardKey.equal:
        _adjust(step);
        return KeyEventResult.handled;
      case LogicalKeyboardKey.home:
        onChanged(minimum);
        return KeyEventResult.handled;
      case LogicalKeyboardKey.end:
        onChanged(maximum);
        return KeyEventResult.handled;
      default:
        return KeyEventResult.ignored;
    }
  }

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final colors = context.colors;
    final percent = _percent;
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: <Widget>[
        Row(
          children: <Widget>[
            _ScaleButton(
              label: decreaseLabel ?? l10n.textSizeDecrease,
              symbol: '\u2212',
              enabled: _scale > minimum,
              onTap: () => _adjust(-step),
            ),
            const SizedBox(width: 12),
            Expanded(
              child: Focus(
                onKeyEvent: _onKey,
                child: Semantics(
                  label: semanticLabel ?? l10n.textSizeControlLabel,
                  value: l10n.textSizePercent(percent),
                  increasedValue: l10n.textSizePercent(_nextPercent),
                  decreasedValue: l10n.textSizePercent(_previousPercent),
                  slider: true,
                  focusable: true,
                  onIncrease: _scale < maximum ? () => _adjust(step) : null,
                  onDecrease: _scale > minimum ? () => _adjust(-step) : null,
                  child: LayoutBuilder(
                    builder: (context, constraints) {
                      final width = constraints.maxWidth;
                      final fraction = (_scale - minimum) / (maximum - minimum);
                      return GestureDetector(
                        behavior: HitTestBehavior.opaque,
                        onTapDown: (details) => _setFromOffset(
                          details.localPosition.dx,
                          width,
                        ),
                        onHorizontalDragUpdate: (details) => _setFromOffset(
                          details.localPosition.dx,
                          width,
                        ),
                        child: SizedBox(
                          height: 48,
                          child: CustomPaint(
                            painter: _TextScaleTrackPainter(
                              fraction: fraction,
                              active: colors.button,
                              inactive: colors.divider,
                              thumb: colors.button,
                            ),
                          ),
                        ),
                      );
                    },
                  ),
                ),
              ),
            ),
            const SizedBox(width: 12),
            _ScaleButton(
              label: increaseLabel ?? l10n.textSizeIncrease,
              symbol: '+',
              enabled: _scale < maximum,
              onTap: () => _adjust(step),
            ),
          ],
        ),
        Align(
          child: Semantics(
            liveRegion: true,
            label: l10n.textSizePercent(percent),
            child: Text(
              l10n.textSizePercent(percent),
              style: context.typography.label,
            ),
          ),
        ),
      ],
    );
  }
}

class _ScaleButton extends StatelessWidget {
  const _ScaleButton({
    required this.label,
    required this.symbol,
    required this.enabled,
    required this.onTap,
  });

  final String label;
  final String symbol;
  final bool enabled;
  final VoidCallback onTap;

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
    return Semantics(
      button: true,
      enabled: enabled,
      label: label,
      child: GestureDetector(
        onTap: enabled ? onTap : null,
        child: ExcludeSemantics(
          child: DecoratedBox(
            decoration: BoxDecoration(
              color: colors.fill,
              borderRadius: BorderRadius.circular(10),
            ),
            child: SizedBox(
              width: 44,
              height: 44,
              child: Center(
                child: Text(
                  symbol,
                  style: context.typography.heading2.copyWith(
                    color: enabled ? colors.text : colors.textMuted,
                  ),
                ),
              ),
            ),
          ),
        ),
      ),
    );
  }
}

class _TextScaleTrackPainter extends CustomPainter {
  const _TextScaleTrackPainter({
    required this.fraction,
    required this.active,
    required this.inactive,
    required this.thumb,
  });

  final double fraction;
  final Color active;
  final Color inactive;
  final Color thumb;

  @override
  void paint(Canvas canvas, Size size) {
    final centerY = size.height / 2;
    const left = 2.0;
    final right = size.width - 2;
    final thumbX = left + (right - left) * fraction;
    final inactivePaint = Paint()
      ..color = inactive
      ..strokeWidth = 3
      ..strokeCap = StrokeCap.round;
    final activePaint = Paint()
      ..color = active
      ..strokeWidth = 3
      ..strokeCap = StrokeCap.round;
    canvas
      ..drawLine(Offset(left, centerY), Offset(right, centerY), inactivePaint)
      ..drawLine(Offset(left, centerY), Offset(thumbX, centerY), activePaint)
      ..drawCircle(Offset(thumbX, centerY), 9, Paint()..color = thumb)
      ..drawCircle(Offset(thumbX, centerY), 4, Paint()..color = Colors.white);
  }

  @override
  bool shouldRepaint(_TextScaleTrackPainter oldDelegate) =>
      oldDelegate.fraction != fraction ||
      oldDelegate.active != active ||
      oldDelegate.inactive != inactive ||
      oldDelegate.thumb != thumb;
}
