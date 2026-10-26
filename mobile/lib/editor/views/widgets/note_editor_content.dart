// Kanso — note_editor_content.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../core/bridge/api/types.dart' hide EditorEvent;
import '../../../core/configs/kanso_spacing.dart';
import '../../../note/state/note/note_bloc.dart';
import '../../models/data/flush_reason.dart';
import '../../models/repos/abs/editor_repository.dart';
import '../../state/editor/editor_bloc.dart';
import '../controllers/kanso_markdown_controller.dart';
import 'editor_conflict_strip.dart';
import 'editor_failure_strip.dart';
import 'editor_title_field.dart';
import 'markdown_editor.dart';
import 'properties_row.dart';

/// The open note's editor: the strips for a change made elsewhere or a
/// failed save, the title, the properties, and the text, in one page that
/// scrolls.
///
/// It owns the field's controller and focus, the title's, and the page's
/// scroll position; it hands them what the `EditorBloc` says, and the
/// `EditorBloc` what they do. When it goes it leaves the caret and scroll
/// position behind for the next time, and the `EditorBloc` above it saves
/// what was typed.
class NoteEditorContent extends StatefulWidget {
  const NoteEditorContent({
    required this.document,
    required this.snapshot,
    required this.gutter,
    this.caret,
    this.caretInBody = false,
    this.scroll = 0,
    this.focusTitle = false,
    super.key,
  });

  /// The note open for editing.
  final EditorDocument document;

  /// The note as the editor opens it.
  final EditorSnapshot snapshot;

  /// The space either side of the page.
  final double gutter;

  /// Where the caret starts, in units of the text; the start when null.
  final int? caret;

  /// Whether [caret] counts the front matter that the text does not have.
  final bool caretInBody;

  /// How far the page was scrolled when the editor was left before.
  final double scroll;

  /// Whether the title, rather than the text, gets the focus (a new note).
  final bool focusTitle;

  /// The longest text the platform's spell checker is given: beyond it spell
  /// check is off, because it checks the whole text at every change.
  static const int spellCheckLimit = 200000;

  @override
  State<NoteEditorContent> createState() => _NoteEditorContentState();
}

class _NoteEditorContentState extends State<NoteEditorContent> {
  late final EditorBloc editor;
  late final NoteBloc note;
  late final KansoMarkdownController controller;
  late final TextEditingController title;
  late final FocusNode textFocus;
  late final FocusNode titleFocus;
  late final ScrollController scroll;
  late final AppLifecycleListener lifecycle;
  EditorContext? lastContext;

  @override
  void initState() {
    super.initState();
    editor = context.read<EditorBloc>();
    note = context.read<NoteBloc>();
    controller = KansoMarkdownController(
      document: widget.document,
      snapshot: widget.snapshot,
      themeId: note.state.themeId,
      onContext: told,
      onEdited: () => editor.add(const EditorEvent.edited()),
      onLocked: () => editor.add(
        // Rust says the lock is closed: the same as being told.
        const EditorEvent.signalReceived(EditorEvent_Closed()),
      ),
    );
    title = TextEditingController(text: widget.snapshot.title);
    textFocus = FocusNode(debugLabel: 'Note text');
    titleFocus = FocusNode(debugLabel: 'Note title');
    scroll = ScrollController(initialScrollOffset: widget.scroll);
    lifecycle = AppLifecycleListener(
      onStateChange: (state) => editor.add(EditorEvent.lifecycleChanged(state)),
    );
    final caret = widget.caret;
    if (caret != null) {
      // The text is the end of the body: what is before it is the front
      // matter.
      final body = note.state.note?.body ?? '';
      final hidden = widget.caretInBody && body.endsWith(controller.text)
          ? body.length - controller.text.length
          : 0;
      final at = (caret - hidden).clamp(0, controller.text.length);
      controller.selection = TextSelection.collapsed(offset: at);
    }
    WidgetsBinding.instance.addPostFrameCallback((_) {
      if (mounted) {
        (widget.focusTitle ? titleFocus : textFocus).requestFocus();
      }
    });
  }

  /// What is under the caret: told on, unless it is what was told last and
  /// nothing completes there.
  void told(EditorContext context) {
    if (context == lastContext && !context.completionHere) {
      return;
    }
    lastContext = context;
    editor.add(EditorEvent.caretMoved(context));
  }

  @override
  void dispose() {
    lifecycle.dispose();
    if (!note.isClosed) {
      final selection = controller.selection;
      note.add(
        NoteEvent.placeKept(
          selection.isValid ? selection.baseOffset : 0,
          scroll.hasClients ? scroll.offset : widget.scroll,
        ),
      );
    }
    scroll.dispose();
    titleFocus.dispose();
    textFocus.dispose();
    title.dispose();
    controller.dispose();
    super.dispose();
  }

  void onTitleChanged(String text) {
    widget.document.setTitle(text);
    editor.add(const EditorEvent.edited());
  }

  void focusText() {
    textFocus.requestFocus();
  }

  /// The blank space below the text puts the caret at its end.
  void focusEnd() {
    controller.selection = TextSelection.collapsed(
      offset: controller.text.length,
    );
    textFocus.requestFocus();
  }

  @override
  Widget build(BuildContext context) {
    final gutter = widget.gutter;
    final long =
        widget.snapshot.text.length > NoteEditorContent.spellCheckLimit;
    return MultiBlocListener(
      listeners: <BlocListener<dynamic, dynamic>>[
        BlocListener<EditorBloc, EditorState>(
          listenWhen: (before, now) => before.reloadSeq != now.reloadSeq,
          listener: (context, state) {
            final snapshot = state.snapshot;
            if (snapshot != null) {
              controller.reload(snapshot);
              if (title.text != snapshot.title) {
                title.text = snapshot.title;
              }
            }
          },
        ),
        BlocListener<EditorBloc, EditorState>(
          listenWhen: (before, now) => before.completion != now.completion,
          listener: (context, state) {
            controller
              ..completion = state.completion
              ..completionIndex = 0;
          },
        ),
        BlocListener<NoteBloc, NoteState>(
          listenWhen: (before, now) => before.themeId != now.themeId,
          listener: (context, state) {
            controller
              ..themeId = state.themeId
              ..requestRuns();
          },
        ),
      ],
      child: Padding(
        padding: EdgeInsets.only(
          bottom: MediaQuery.viewInsetsOf(context).bottom,
        ),
        child: Column(
          children: <Widget>[
            BlocBuilder<EditorBloc, EditorState>(
              buildWhen: (before, now) => before.save != now.save,
              builder: (context, state) {
                return switch (state.save) {
                  SaveState_Conflict(:final deleted) => EditorConflictStrip(
                    deleted: deleted,
                    gutter: gutter,
                    onKeepMine: () =>
                        editor.add(const EditorEvent.keepMineRequested()),
                    onUseTheirs: () =>
                        editor.add(const EditorEvent.useTheirsRequested()),
                  ),
                  SaveState_Failed() => EditorFailureStrip(
                    gutter: gutter,
                    onRetry: () => editor.add(
                      const EditorEvent.flushRequested(FlushReason.retry),
                    ),
                  ),
                  _ => const SizedBox.shrink(),
                };
              },
            ),
            Expanded(
              child: CustomScrollView(
                controller: scroll,
                slivers: <Widget>[
                  SliverPadding(
                    padding: EdgeInsets.fromLTRB(
                      gutter,
                      KansoSpacing.xxs,
                      gutter,
                      0,
                    ),
                    sliver: SliverToBoxAdapter(
                      child: Column(
                        crossAxisAlignment: CrossAxisAlignment.stretch,
                        children: <Widget>[
                          EditorTitleField(
                            controller: title,
                            focusNode: titleFocus,
                            onChanged: onTitleChanged,
                            onSubmitted: focusText,
                          ),
                          BlocBuilder<EditorBloc, EditorState>(
                            buildWhen: (before, now) =>
                                before.properties.length !=
                                now.properties.length,
                            builder: (context, state) => Align(
                              alignment: AlignmentDirectional.centerStart,
                              child: PropertiesRow(
                                count: state.properties.length,
                              ),
                            ),
                          ),
                        ],
                      ),
                    ),
                  ),
                  SliverPadding(
                    padding: EdgeInsets.symmetric(horizontal: gutter),
                    sliver: SliverToBoxAdapter(
                      child: MarkdownEditor(
                        controller: controller,
                        focusNode: textFocus,
                        spellCheckService: long
                            ? null
                            : DefaultSpellCheckService(),
                      ),
                    ),
                  ),
                  SliverFillRemaining(
                    hasScrollBody: false,
                    child: GestureDetector(
                      behavior: HitTestBehavior.opaque,
                      onTap: focusEnd,
                      child: SizedBox(
                        height:
                            MediaQuery.paddingOf(context).bottom +
                            KansoSpacing.xl,
                        width: double.infinity,
                      ),
                    ),
                  ),
                ],
              ),
            ),
          ],
        ),
      ),
    );
  }
}
