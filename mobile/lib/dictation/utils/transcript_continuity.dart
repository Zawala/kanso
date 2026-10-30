// Kanso — transcript_continuity.dart
// Created 2026-10-29.
// Licensed under AGPL-3.0-or-later (see LICENSE).

/// Tells a recognizer's next words for the phrase being spoken from words it
/// started over with.
///
/// A recognizer reports the phrase being spoken again and again, each report
/// replacing the last: usually the same words with more on the end, or with
/// the last few heard better. Some recognizers (on-device recognition on
/// recent iOS, an Android recognizer listening again) instead start over
/// after a pause without saying the phrase before was finished. Taken as a
/// replacement, that would drop everything said before the pause. When
/// [restarts] says the next report started over, the words before it are a
/// finished phrase and must be kept.
class TranscriptContinuity {
  TranscriptContinuity({DateTime Function()? clock})
    : clock = clock ?? DateTime.now;

  /// The time now; tests set their own.
  DateTime Function() clock;

  /// When the last report came, or null before the first of a phrase.
  DateTime? last;

  /// How long a recognizer has heard nothing new before it starts over.
  static const Duration pause = Duration(milliseconds: 800);

  /// Whether [next] starts over rather than carrying on from [previous],
  /// the report before it. Remembers when it was asked, so that the next
  /// question knows how long the voice paused.
  bool restarts(String previous, String next) {
    final now = clock();
    final gap = last == null ? Duration.zero : now.difference(last!);
    last = now;
    return startsOver(words(previous), words(next), gap: gap);
  }

  /// The phrase was finished: the next report begins a new one.
  void reset() => last = null;

  /// Whether the words [next] start over rather than carry on from
  /// [previous], [gap] after them.
  static bool startsOver(
    List<String> previous,
    List<String> next, {
    Duration gap = Duration.zero,
  }) {
    if (previous.isEmpty || next.isEmpty) {
      return false;
    }
    final shared = commonPrefix(previous, next);
    if (shared == previous.length) {
      // The same words, or more of them.
      return false;
    }
    final paused = gap >= pause;
    if (paused && shared == 0) {
      return true;
    }
    if (paused && next.length < previous.length && shared < next.length) {
      return true;
    }
    // Far shorter and nothing alike: no correction drops that much at once.
    return previous.length >= 4 &&
        next.length * 3 <= previous.length &&
        shared == 0;
  }

  /// How many words [a] and [b] begin with alike.
  static int commonPrefix(List<String> a, List<String> b) {
    final most = a.length < b.length ? a.length : b.length;
    var i = 0;
    while (i < most && a[i] == b[i]) {
      i++;
    }
    return i;
  }

  /// The words of [text] as compared: lower case, without punctuation, so
  /// that "hello" and "Hello," are the same word.
  static List<String> words(String text) => <String>[
    for (final word in text.toLowerCase().split(_space))
      if (word.replaceAll(_notWord, '') case final bare when bare.isNotEmpty)
        bare,
  ];

  static final RegExp _space = RegExp(r'\s+');
  static final RegExp _notWord = RegExp(r'[^\p{L}\p{N}]', unicode: true);
}
