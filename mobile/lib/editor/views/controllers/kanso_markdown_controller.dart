// Kanso — kanso_markdown_controller.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:dartz/dartz.dart';
import 'package:flutter/scheduler.dart';
import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';

import '../../../core/bridge/api/types.dart';
import '../../../core/models/data/application_error.dart';
import '../../models/repos/abs/editor_repository.dart';
import '../../utils/diff_values.dart';
import '../../utils/editor_spans.dart';
import '../../utils/shift_runs.dart';

/// The controller of the note's text field: the only thing that changes the
/// text.
///
/// The text is Rust's. Every change the field makes (a key, the input
/// method, a paste) comes through [value], is told to the open
/// [EditorDocument] as a [TextChange] and, when Rust accepts it, shown;
/// what Rust decides (a newline that continues a list, a format, an undo, a
/// completion chosen) comes back as a replacement that is applied in the
/// same turn, so no keystroke can land between the two.
///
/// It also keeps the colours of the text: the last runs the core sent,
/// moved along as the text changes ([shiftRuns]) until fresh ones arrive,
/// asked for once a frame at most, and drawn by [buildTextSpan].
///
/// A text that arrives from outside ([reload]) waits for a composition to
/// end; the text is never replaced under one.
class KansoMarkdownController extends TextEditingController {
  KansoMarkdownController({
    required this.document,
    required EditorSnapshot snapshot,
    required this.themeId,
    this.onContext,
    this.onEdited,
    this.onLocked,
    this.onFailure,
    this.misspelledStyle = const TextStyle(
      decoration: TextDecoration.underline,
      decorationStyle: TextDecorationStyle.wavy,
    ),
    this.composingStyle = const TextStyle(decoration: TextDecoration.underline),
  }) : _version = snapshot.version,
       _history = snapshot.history,
       super(text: snapshot.text) {
    // Rust starts with the caret at the start; tell it where the field's is.
    document.select(const Range16(start: 0, end: 0), composing: false);
    requestRuns();
  }

  /// The note open for editing.
  final EditorDocument document;

  /// The built-in theme the text is coloured in. Asks for fresh colours when
  /// it changes.
  String themeId;

  /// Told what is under the caret after every change to the text or the
  /// selection.
  final ValueChanged<EditorContext>? onContext;

  /// Told after each change to the text, whoever made it.
  final VoidCallback? onEdited;

  /// Told when Rust says the note's lock has closed.
  final VoidCallback? onLocked;

  /// Told of an error that is neither of the above.
  final ValueChanged<ApplicationError>? onFailure;

  /// How a misspelled word is drawn.
  final TextStyle misspelledStyle;

  /// How the text being composed is drawn.
  final TextStyle composingStyle;

  /// The completion being offered, and the item Return would take; set by
  /// the screen. A newline typed while there is one chooses it.
  Completion? completion;
  int completionIndex = 0;

  int _version;
  HistoryState _history;
  List<EditorRun> _runs = const <EditorRun>[];
  Range16? _lastSelection;
  bool _lastComposing = false;
  bool _syncing = false;
  bool _disposed = false;
  bool _runsDue = false;
  bool _runsScheduled = false;
  bool _runsInFlight = false;
  EditorSnapshot? _waiting;

  // The last spans built, to build them again only for new input.
  String? _spanText;
  List<EditorRun>? _spanRuns;
  TextRange? _spanComposing;
  List<TextRange>? _spanMisspellings;
  TextStyle? _spanBase;
  TextSpan? _span;

  /// The version of the text Rust has, as of the last thing it said.
  int get version => _version;

  /// Whether there is anything to undo and redo, as of the last thing Rust
  /// said.
  HistoryState get history => _history;

  /// The colours the text is drawn in now.
  List<EditorRun> get runs => _runs;

  /// Whether an input method is composing text now.
  bool get isComposing => _composingIn(value);

  /// Whether the text at [offset] is code, an address or other text that is
  /// not spell checked.
  bool isQuietAt(int offset) {
    var low = 0;
    var high = _runs.length - 1;
    while (low <= high) {
      final middle = (low + high) >> 1;
      final run = _runs[middle];
      if (offset < run.range.start) {
        high = middle - 1;
      } else if (offset >= run.range.end) {
        low = middle + 1;
      } else {
        return run.quiet;
      }
    }
    return false;
  }

  // MARK: Changes the field makes

  @override
  set value(TextEditingValue newValue) {
    if (_syncing || _disposed) {
      super.value = newValue;
      return;
    }
    final old = super.value;
    if (old.text == newValue.text) {
      super.value = newValue;
      _moved(newValue);
      return;
    }
    _typed(old, newValue);
  }

  void _typed(TextEditingValue old, TextEditingValue now) {
    final diff = diffValues(old, now);
    final composingBefore = _composingIn(old);
    final composingAfter = _composingIn(now);
    if (diff.isInsertion &&
        diff.text == '\n' &&
        !composingBefore &&
        !composingAfter) {
      // Return: a list continues, a table gets its next row; or the
      // completion is chosen.
      final offered = completion;
      if (offered != null &&
          completionIndex >= 0 &&
          completionIndex < offered.items.length) {
        _tell(old);
        insert(offered.items[completionIndex]);
        return;
      }
      _tell(old);
      if (_command(
        EditCommand.enter(plain: HardwareKeyboard.instance.isShiftPressed),
      )) {
        return;
      }
    }
    final length = now.text.length;
    final change = TextChange(
      baseVersion: _version,
      start: diff.start,
      end: diff.end,
      text: diff.text,
      selection: _rangeOf(now.selection, length),
      composing: composingAfter
          ? Range16(
              start: now.composing.start.clamp(0, length),
              end: now.composing.end.clamp(0, length),
            )
          : null,
    );
    document
        .apply(change)
        .fold(
          (error) {
            // The field has the person's text and Rust refused it. Put the
            // field back as it was; it is told to the input method too.
            notifyListeners();
            _fail(error);
          },
          (outcome) {
            _version = outcome.version;
            _history = outcome.history;
            _runs = shiftRuns(
              _runs,
              start: diff.start,
              end: diff.end,
              inserted: diff.text.length,
            );
            super.value = now;
            final ended = _lastComposing && !composingAfter;
            _tell(now);
            requestRuns();
            onEdited?.call();
            if (ended) {
              _showWaiting();
            }
          },
        );
  }

  /// A selection or composition change with the text as it was.
  void _moved(TextEditingValue now) {
    final selection = _rangeOf(now.selection, now.text.length);
    final composing = _composingIn(now);
    if (selection == _lastSelection && composing == _lastComposing) {
      return;
    }
    final ended = _lastComposing && !composing;
    _tell(now);
    if (ended) {
      _showWaiting();
    }
  }

  /// Shows the text that waited for a composition to end.
  void _showWaiting() {
    final waiting = _waiting;
    if (waiting != null) {
      _waiting = null;
      _show(waiting);
    }
  }

  /// Tells Rust where the selection is, and the screen what is under it.
  void _tell(TextEditingValue now) {
    final selection = _rangeOf(now.selection, now.text.length);
    final composing = _composingIn(now);
    _lastSelection = selection;
    _lastComposing = composing;
    document.select(selection, composing: composing).fold(_fail, (context) {
      _history = context.history;
      onContext?.call(context);
    });
  }

  // MARK: Changes Rust decides

  /// Runs [command] and shows what it did. False when there was nothing for
  /// it to do.
  bool run(EditCommand command) => _command(command);

  bool _command(EditCommand command) {
    return document.command(command).fold((error) {
      _fail(error);
      return false;
    }, _adopt);
  }

  /// Undoes the last step; false when there is none.
  bool undo() {
    return document.undo().fold((error) {
      _fail(error);
      return false;
    }, _adopt);
  }

  /// Redoes the last undone step; false when there is none.
  bool redo() {
    return document.redo().fold((error) {
      _fail(error);
      return false;
    }, _adopt);
  }

  /// Puts the completion [item] in the place the completion names, as one
  /// step.
  bool insert(CompletionItem item) {
    final offered = completion;
    if (offered == null) {
      return false;
    }
    completion = null;
    return _command(
      EditCommand.insert(range: offered.replace, text: item.insert),
    );
  }

  /// Shows what Rust did: the replacement, then the selection. False for no
  /// outcome.
  bool _adopt(EditOutcome? outcome) {
    if (outcome == null) {
      return false;
    }
    final replace = outcome.replace;
    var next = super.value.text;
    if (replace != null) {
      next = next.replaceRange(replace.start, replace.end, replace.text);
      _runs = shiftRuns(
        _runs,
        start: replace.start,
        end: replace.end,
        inserted: replace.text.length,
      );
    }
    _version = outcome.version;
    _history = outcome.history;
    final selection = TextSelection(
      baseOffset: outcome.selection.start.clamp(0, next.length),
      extentOffset: outcome.selection.end.clamp(0, next.length),
    );
    final shown = TextEditingValue(text: next, selection: selection);
    _syncing = true;
    try {
      super.value = shown;
    } finally {
      _syncing = false;
    }
    _tell(shown);
    requestRuns();
    onEdited?.call();
    return true;
  }

  // MARK: Text from outside

  /// Shows [snapshot]'s text in place of the field's (the note was changed
  /// elsewhere, or the person took the other version), after a composition
  /// ends if one is open. The caret stays where it was, as far as the text
  /// reaches.
  void reload(EditorSnapshot snapshot) {
    if (isComposing) {
      _waiting = snapshot;
      return;
    }
    _show(snapshot);
  }

  void _show(EditorSnapshot snapshot) {
    final old = super.value;
    final length = snapshot.text.length;
    final shown = TextEditingValue(
      text: snapshot.text,
      selection: old.selection.isValid
          ? TextSelection(
              baseOffset: old.selection.baseOffset.clamp(0, length),
              extentOffset: old.selection.extentOffset.clamp(0, length),
            )
          : const TextSelection.collapsed(offset: 0),
    );
    _version = snapshot.version;
    _history = snapshot.history;
    _runs = const <EditorRun>[];
    _syncing = true;
    try {
      super.value = shown;
    } finally {
      _syncing = false;
    }
    _tell(shown);
    requestRuns();
  }

  /// Takes the text Rust has: after it refused a change made against an
  /// older text.
  Future<void> resync() async {
    final result = await document.snapshot();
    if (_disposed) {
      return;
    }
    result.fold(_fail, reload);
  }

  void _fail(ApplicationError error) {
    switch (error) {
      case LockedError():
        onLocked?.call();
      case InvalidError():
        unawaited(resync());
      default:
        onFailure?.call(error);
    }
  }

  // MARK: Colours

  /// Asks for fresh colours: once a frame at most, one request at a time,
  /// and never one for a version of the text that has been left behind.
  void requestRuns() {
    _runsDue = true;
    if (_runsScheduled || _runsInFlight || _disposed) {
      return;
    }
    _runsScheduled = true;
    SchedulerBinding.instance
      ..addPostFrameCallback((_) {
        _runsScheduled = false;
        unawaited(_fetchRuns());
      })
      ..ensureVisualUpdate();
  }

  Future<void> _fetchRuns() async {
    if (_disposed || !_runsDue) {
      return;
    }
    _runsDue = false;
    _runsInFlight = true;
    final Either<ApplicationError, RunBatch> result = await document.styledRuns(
      start: 0,
      end: super.value.text.length,
      themeId: themeId,
    );
    _runsInFlight = false;
    if (_disposed) {
      return;
    }
    result.fold(
      (error) {
        if (error is LockedError) {
          onLocked?.call();
        }
        // Any other error leaves the text as it is coloured: plain is fine.
      },
      (batch) {
        if (batch.version == _version) {
          _runs = batch.runs;
          notifyListeners();
        } else {
          // Left behind by a change made while it was on the way.
          _runsDue = true;
        }
      },
    );
    if (_runsDue) {
      requestRuns();
    }
  }

  // MARK: Drawing

  @override
  TextSpan buildTextSpan({
    required BuildContext context,
    TextStyle? style,
    // ignore: always_put_required_named_parameters_first
    required bool withComposing,
  }) {
    return spanFor(
      style: style ?? const TextStyle(),
      withComposing: withComposing,
    );
  }

  /// The text as spans in [style], with the misspellings in [misspellings]
  /// marked (outside code and addresses).
  TextSpan spanFor({
    required TextStyle style,
    required bool withComposing,
    List<TextRange> misspellings = const <TextRange>[],
  }) {
    final text = super.value.text;
    final composing = withComposing ? super.value.composing : TextRange.empty;
    final cached = _span;
    if (cached != null &&
        identical(text, _spanText) &&
        identical(_runs, _spanRuns) &&
        composing == _spanComposing &&
        identical(misspellings, _spanMisspellings) &&
        style == _spanBase) {
      return cached;
    }
    final span = editorSpans(
      text: text,
      runs: _runs,
      base: style,
      composing: composing,
      misspellings: misspellings,
      composingStyle: composingStyle,
      misspelledStyle: misspelledStyle,
    );
    _spanText = text;
    _spanRuns = _runs;
    _spanComposing = composing;
    _spanMisspellings = misspellings;
    _spanBase = style;
    _span = span;
    return span;
  }

  @override
  void dispose() {
    _disposed = true;
    super.dispose();
  }

  // MARK: Units

  static bool _composingIn(TextEditingValue value) =>
      value.composing.isValid && !value.composing.isCollapsed;

  /// [selection] as units of a text [length] long; the end of the text for
  /// a selection the field has not set yet.
  static Range16 _rangeOf(TextSelection selection, int length) {
    if (!selection.isValid) {
      return Range16(start: length, end: length);
    }
    return Range16(
      start: selection.start.clamp(0, length),
      end: selection.end.clamp(0, length),
    );
  }
}
