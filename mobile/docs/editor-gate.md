# The editor's performance gate

The Markdown editor of the phone app is a Flutter `EditableText`. This is the
gate it has to pass before it ships, and what has been measured so far.

## What is measured

`integration_test/helpers/typing_script.dart` types into a note of N words
(`long_note.dart`: headings, lists, tasks, tables, fenced code, links,
wiki-links, a CJK paragraph, emoji, and one broken table three quarters down,
which sends the highlighter down its slow path). The script:

- 300 characters at the end of the note and 300 in the middle, at 10 per second
  and in bursts of 30 per second;
- Return at the end of a list, 20 times;
- one paste of 2 KB;
- 20 undos;
- two minutes of steady typing, for the memory measure.

Input goes through `EditableTextState.updateEditingValue`, the method the
platform channel calls. `lib/core/utils/typing_report.dart` turns the frames
and the timed spans into a `TypingReport`.

| Measure | Limit, in profile mode on a device |
| --- | --- |
| Build and raster time | p99 under one refresh interval, at most 1% of frames over it |
| Keystroke to frame | p95 at most 2 intervals, p99 at most 4 |
| Fresh highlight (a change to its new colours) | p95 at most 100 ms |
| `kanso.apply` (the core's call for a change) | p99 at most 1 ms |
| Memory growth over the two minutes | at most 50 MB |

## How it runs

```sh
# With a cable (or a phone that keeps its Dart VM service reachable):
flutter drive --profile \
  --driver=test_driver/typing_driver.dart \
  --target=integration_test/editor_typing_performance_test.dart \
  -d <device> [--dart-define=KANSO_TYPING_WORDS=20000] \
  [--dart-define=KANSO_WINDOW_UNITS=100000000]   # the whole note in the field

# Without the VM service (a wireless iPhone): the same script as an app.
flutter build ios --profile -t integration_test/editor_bench_app.dart \
  --dart-define=KANSO_TYPING_WORDS=50000
xcrun devicectl device install app --device <iPhone> build/ios/iphoneos/Runner.app
xcrun devicectl device process launch --device <iPhone> app.kanso.Kanso
xcrun devicectl device copy from --device <iPhone> \
  --domain-type appDataContainer --domain-identifier app.kanso.Kanso \
  --source Documents/editor_typing.json --destination editor_typing.json
```

The phone must stay unlocked and awake for the whole run (a few minutes):
when the screen locks, iOS suspends the app and the run stalls or its tail
percentiles are nonsense. In debug mode, or on a simulator or emulator, the
test runs a short script on a small note and only checks that the report was
made (CI does this).

`--dart-define=KANSO_WINDOW_UNITS=<n>` sets the size of the window (below);
without it the window is 8,192 UTF-16 units.

## Window mode

Flutter lays out and paints an `EditableText`'s whole text as one paragraph on
every change, and the text input channel sends the whole text with each update.
So a note longer than the window (8,192 UTF-16 units by default, about 1,400
words) is edited a window of whole lines at a time around the selection; the
text before and after is drawn as read-only slivers in the same scroll view.
The window moves when the selection comes within 1,024 units of an edge and no
composition is open. Select All makes the window the whole note (one slow
layout, for something deliberate). A note shorter than the window has exactly
one window, so ordinary notes behave as before. The Rust core never knows
about windows. Spell check is only given the window, so there is no longer a
character limit for it (the old limit of 200,000 characters is gone).

Costs, accepted by the owner: VoiceOver and TalkBack announce the field as the
window and read the rest as static text; a selection is limited to about two
windows except with Select All.

## Results

Fixture: `longNote(words)`; 5,000 words = 24,667 units, hash
`d72fb80f4399584d`. Flutter 3.44.8 (framework `058e0af2c2`, engine
`0cd610717b`), Dart 3.12.2.

### iPhone 17 Pro (the proxy for an iPhone 12-class phone), iOS 27.2, profile, 120 Hz, wireless

The budget is 8.33 ms. The run was wireless: `flutter drive` reached the app
only now and then (some attempts lost the VM service, one installed in ten
minutes), and the full script could not be completed under supervision. Two
runs gave numbers; neither is a clean gate run.

| Run | Mode | Frames | Build p50 / p99 (ms) | Raster p99 | Frames over | Keystroke to frame p50 / p95 / p99 (ms) | Fresh highlight p50 / p95 (ms) | apply p99 | Memory |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 5k words, quick script (167 keystrokes), `flutter drive` | whole note in the field | 542 | 0.29 / 39.4 | 2.5 | 22.1% | 34 / 114 / 177 | 338 / 1,657 | 0.42 ms | 215 to 219 MB |
| 5k words, full script (2,462 keystrokes), bench app | whole note in the field | 919 | 15.1 / 22.1 | 2.1 | 94.1% | 42 / 389 / (tail unusable) | 1,288 / (tail unusable) | 0.05 ms | 193 to 209 MB (+16.9) |

The second run was suspended by the screen locking for most of its length, so
its p99 keystroke latency and p95 fresh highlight (506 s, 538 s) are the
suspension, not the editor. Its frame build times are real: a median frame
that builds in 15 ms against a budget of 8.33 ms, with a note of only 5,000
words. Raster is not the problem (p99 about 2 ms); the time is in the build,
which is the paragraph being laid out again. `kanso.apply` is far inside its
limit in both runs.

The 20k and 50k whole-note runs, and every window-mode run on the iPhone,
were **not measured**: the phone's screen locked during each attempt.

### Android emulator (Pixel-class AVD, Android 16, 60 Hz, profile): functional numbers only

An emulator's timings come from the host's software GPU and say nothing about
a phone; there are no thresholds here. What they show is that the script runs
and that size matters.

| Words | Mode | Script | Frames | Build p50 (ms) | Keystroke to frame p50 (ms) | Note |
| --- | --- | --- | --- | --- | --- | --- |
| 5,000 | window 8,192 | quick | 136 | 7.4 | 1,141 | runs |
| 50,000 | window 8,192 | quick | 44 | 49.1 | 1,724 | runs; opening the note took five minutes |
| 50,000 | whole note | full | | | | stalled: frames of 9 s and 2,300 skipped frames; the driver lost the app |

### Mid-range Android

**Not measured: no device.** The gate stays open (owner decision: ship with
the Android gate recorded as open).

## Memory and Flutter's undo history

`EditableText` keeps a copy of the whole text in an unbounded undo stack every
500 ms (Kanso's own history does the undo; Flutter's is never read). For a
note held whole this is about 600 KB every 500 ms at 50,000 words, about
70 MB over two minutes of typing. With window mode the field holds at most
about 8,000 units, so the same two minutes cost about 2 MB; that is the
mitigation. Swapping the controller at each autosave is therefore not built.
The iPhone's 5k-word run grew by 16.9 MB over the two minutes (a 25 KB note;
the suspension makes the figure unreliable), so the 50 MB limit is **not
confirmed** for 50k words in either mode.

## Findings that need a clean run

1. **Whole-note editing misses the budget at 5,000 words** on the proxy phone
   (median frame build 15 ms against 8.33 ms). At 50,000 words it cannot be
   better, and the emulator's whole-note run stalled. Window mode is built for
   this reason.
2. **Fresh highlight is slow at 5,000 words** (median 338 ms in the quick run
   on the phone, against a limit of 100 ms at p95). That is Kanso's code, or the
   core's, not Flutter's layout: the time between a change and its new colours
   includes the coalescing to one request per frame and the call into Rust.
   The escape-hatch table says to fix this within step 6; it needs a profile
   (the timeline of a clean run) first.

## Decision

Provisional. Window mode stays in step 6 (it was built, has unit, widget and
golden tests, and works on the emulator). **The gate is not measured on the
iPhone in window mode, so it is not passed.** Step 6b (a native text view) is
not triggered by these numbers: the only time seen on a phone is in the build
of one whole paragraph, which is what window mode removes, and the highlight
delay is in Kanso's own code. A clean run on the iPhone at 5k, 20k and 50k
words (window mode, then whole note for the record) decides it. What is needed
is a cable (`flutter drive` over USB), or the phone unlocked with Auto-Lock
set to Never for ten minutes per run.

## Summary for #48

Step 6, part 3: long notes and the gate.

- The editor types into a window of whole lines (8,192 units) around the
  caret for long notes, with the text before and after drawn read-only in the
  same scroll view; a note shorter than the window is one window. Spell check
  needs no size limit any more.
- The gate (`integration_test/editor_typing_performance_test.dart`, and the
  self-contained `editor_bench_app.dart` for a phone `flutter drive` cannot
  reach) is built and documented in `mobile/docs/editor-gate.md`.
- On the iPhone 17 Pro (the proxy, 120 Hz), whole-note editing already misses
  the build budget at 5,000 words (median frame 15 ms against 8.33 ms), and the
  fresh-highlight delay is too long (median 338 ms). The wireless phone's
  screen locked during every other attempt, so no clean full-script run exists
  yet, and window mode is **not measured on the phone**. The gate is therefore
  not passed. Needed: a USB cable, or an unlocked phone with Auto-Lock off.
- The Android emulator runs the script (functional only). The mid-range
  Android gate stays open (no device).
- Composition (Japanese, Chinese, Gboard-style), autocorrect, dictation, spell
  check and handwriting placeholders pass on an iPhone simulator and an Android
  emulator. VoiceOver on the iPhone needs the owner; Scribble was not run.
