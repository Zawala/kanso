import AVFoundation
import Speech

/// One run of the recognizer: the microphone into an on-device request,
/// until the person stops it, the system interrupts it or an unrecoverable
/// error ends it. It reports through `emit`, and the last thing it reports,
/// once, is `ended`.
///
/// The recognizer closes a task after a phrase. Dictation is continuous, so
/// when a phrase finishes and the person has not stopped it, the task is
/// replaced while the audio engine and its microphone tap keep running. The
/// tap appends to whichever request is current, through
/// [RecognitionRequestSink], so restarting a phrase never closes the
/// microphone or loses the next words.
@MainActor
final class DictationSession {
  private let recognizer: SFSpeechRecognizer
  private let emit: (DictationEvent) -> Void
  private let engine = AVAudioEngine()
  private let sink = RecognitionRequestSink()

  private var request: SFSpeechAudioBufferRecognitionRequest?
  private var task: SFSpeechRecognitionTask?
  private var interruption: NSObjectProtocol?
  private var mediaReset: NSObjectProtocol?
  private var finishTimer: Task<Void, Never>?
  private var restart: Task<Void, Never>?
  private var tapInstalled = false
  private var run = 0
  /// The run that already delivered its final result, so a trailing error
  /// from the same closed task cannot end a continuous session.
  private var finishedRun = 0
  private var stopping = false
  private(set) var finished = false

  /// How long the recognizer gets to deliver its final result after the
  /// microphone closed.
  private static let finishTimeout: Duration = .seconds(3)

  /// A recognizer that ends on silence gets another task shortly after.
  private static let restartDelay: Duration = .milliseconds(250)

  init(recognizer: SFSpeechRecognizer, emit: @escaping (DictationEvent) -> Void) {
    self.recognizer = recognizer
    self.emit = emit
  }

  /// Opens the microphone and starts recognizing. Throws when the audio
  /// session or the engine cannot start; nothing is left running then.
  func begin() throws {
    let audio = AVAudioSession.sharedInstance()
    do {
      try audio.setCategory(.record, mode: .measurement, options: [.duckOthers])
      try audio.setActive(true, options: .notifyOthersOnDeactivation)

      let input = engine.inputNode
      input.installTap(
        onBus: 0,
        bufferSize: 1024,
        format: input.outputFormat(forBus: 0),
        block: DictationSession.tap(sink: sink, emit: emit)
      )
      tapInstalled = true
      engine.prepare()
      try engine.start()
    } catch {
      teardown()
      throw error
    }

    observeSystem(audio)
    beginTask()
    emit(DictationEvent(kind: .listening))
  }

  /// Closes the microphone and lets the recognizer finish what it heard.
  func stop() {
    guard !finished, !stopping else { return }
    restart?.cancel()
    restart = nil

    // The phrase's final result is already in the note. If no audio has
    // arrived since, there is nothing left for the recognizer to finish and
    // waiting three seconds would only delay the terminal event.
    if finishedRun == run {
      let buffered = sink.hasBufferedAudio
      closeMicrophone()
      if !buffered {
        end()
        return
      }
      stopping = true
      // Flush the short post-final gap into a fresh request before ending it.
      beginTask(allowWhileStopping: true)
      request?.endAudio()
      startFinishTimer()
      return
    }

    stopping = true
    closeMicrophone()
    request?.endAudio()
    startFinishTimer()
  }

  /// Stops at once, dropping what the recognizer had not finished.
  func cancel() {
    guard !finished else { return }
    stopping = true
    restart?.cancel()
    restart = nil
    finishTimer?.cancel()
    task?.cancel()
    end()
  }

  /// Starts a fresh request and recognition task. The audio tap is already
  /// alive and will append the next buffer to this request.
  private func beginTask(allowWhileStopping: Bool = false) {
    guard !finished,
          allowWhileStopping || (!stopping && engine.isRunning)
    else { return }
    let request = makeRequest()
    self.request = request
    sink.use(request)
    run += 1
    let run = self.run
    task = recognizer.recognitionTask(with: request) { [weak self] result, error in
      let text = result?.bestTranscription.formattedString
      let isFinal = result?.isFinal ?? false
      let failure = error as NSError?
      Task { @MainActor in
        self?.handle(run: run, text: text, isFinal: isFinal, failure: failure)
      }
    }
  }

  private func makeRequest() -> SFSpeechAudioBufferRecognitionRequest {
    let request = SFSpeechAudioBufferRecognitionRequest()
    request.shouldReportPartialResults = true
    // Never a server: the caller checked supportsOnDeviceRecognition.
    request.requiresOnDeviceRecognition = true
    request.addsPunctuation = true
    return request
  }

  private func handle(
    run: Int,
    text: String?,
    isFinal: Bool,
    failure: NSError?
  ) {
    // A callback from a task that has already been replaced or stopped is
    // stale; only the current run may change the session. Once stopping, a
    // phrase's final result still closes the session, while partials no
    // longer need to reach the note.
    guard !finished, run == self.run else { return }
    if failure != nil, finishedRun == run { return }
    if stopping, failure == nil, !isFinal { return }

    if let text, !text.isEmpty {
      emit(DictationEvent(kind: isFinal ? .finalResult : .partial, text: text))
    }

    if isFinal {
      finishedRun = run
      // Do not let the microphone keep appending to a request whose task has
      // already finished. Buffers arriving during the short restart gap are
      // replayed into the next request by the sink.
      sink.use(nil)
      if stopping {
        end()
      } else {
        scheduleRestart()
      }
      return
    }

    if let failure {
      let code = DictationSession.code(of: failure)
      if stopping {
        end()
      } else if code == .noSpeech {
        scheduleRestart()
      } else {
        emit(
          DictationEvent(
            kind: .error,
            error: code,
            message: "\(failure.domain) \(failure.code)"
          )
        )
        end()
      }
    }
  }

  /// Gives the recognizer a bounded window to deliver its final result.
  private func startFinishTimer() {
    finishTimer?.cancel()
    finishTimer = Task { @MainActor [weak self] in
      try? await Task.sleep(for: DictationSession.finishTimeout)
      guard !Task.isCancelled else { return }
      self?.end()
    }
  }

  /// The recognizer finished a phrase, or heard silence. Keep the microphone
  /// open and begin the next task after the current callback has returned.
  private func scheduleRestart() {
    guard !finished, !stopping else { return }
    restart?.cancel()
    restart = Task { @MainActor [weak self] in
      try? await Task.sleep(for: DictationSession.restartDelay)
      guard !Task.isCancelled, let self, !self.finished, !self.stopping else {
        return
      }
      self.task?.cancel()
      self.task = nil
      self.request = nil
      self.beginTask()
    }
  }

  /// The reason for a failure of the recognition task.
  static func code(of failure: NSError) -> DictationErrorCode {
    // kAFAssistantErrorDomain 1110: no speech was detected.
    if failure.domain == "kAFAssistantErrorDomain", failure.code == 1110 {
      return .noSpeech
    }
    return .unknown
  }

  /// The tap on the microphone, made away from the session so that it holds
  /// nothing of it: it runs on the audio thread. It appends to the request
  /// current at that moment, which changes when a phrase ends.
  nonisolated private static func tap(
    sink: RecognitionRequestSink,
    emit: @escaping (DictationEvent) -> Void
  ) -> AVAudioNodeTapBlock {
    let throttle = LevelThrottle()
    return { buffer, _ in
      sink.append(buffer)
      if let level = throttle.next(DictationLevel.of(buffer)) {
        DispatchQueue.main.async { emit(DictationEvent(kind: .level, level: level)) }
      }
    }
  }

  private func observeSystem(_ audio: AVAudioSession) {
    let center = NotificationCenter.default
    interruption = center.addObserver(
      forName: AVAudioSession.interruptionNotification, object: audio, queue: .main
    ) { [weak self] note in
      let raw = note.userInfo?[AVAudioSessionInterruptionTypeKey] as? UInt
      guard raw == AVAudioSession.InterruptionType.began.rawValue else { return }
      Task { @MainActor in
        guard let self, !self.finished else { return }
        self.emit(DictationEvent(kind: .error, error: .interrupted))
        self.stop()
      }
    }
    mediaReset = center.addObserver(
      forName: AVAudioSession.mediaServicesWereResetNotification, object: audio, queue: .main
    ) { [weak self] _ in
      Task { @MainActor in
        guard let self, !self.finished else { return }
        self.emit(DictationEvent(kind: .error, error: .audio))
        self.cancel()
      }
    }
  }

  private func closeMicrophone() {
    if engine.isRunning {
      engine.stop()
    }
    if tapInstalled {
      engine.inputNode.removeTap(onBus: 0)
      tapInstalled = false
    }
    sink.use(nil)
  }

  private func teardown() {
    restart?.cancel()
    restart = nil
    finishTimer?.cancel()
    finishTimer = nil
    closeMicrophone()
    task?.cancel()
    task = nil
    request = nil
    for token in [interruption, mediaReset] {
      if let token { NotificationCenter.default.removeObserver(token) }
    }
    interruption = nil
    mediaReset = nil
    try? AVAudioSession.sharedInstance().setActive(false, options: .notifyOthersOnDeactivation)
  }

  private func end() {
    guard !finished else { return }
    finished = true
    stopping = true
    request?.endAudio()
    task?.cancel()
    teardown()
    emit(DictationEvent(kind: .ended))
  }
}

protocol AudioBufferAppending: AnyObject {
  func append(_ buffer: AVAudioPCMBuffer)
}

extension SFSpeechAudioBufferRecognitionRequest: AudioBufferAppending {}

/// Hands the audio tap's buffers to whichever recognition request is current.
/// Appending and switching happen on different threads: the phone's audio
/// tap runs on the audio thread, while a finished phrase replaces the request
/// on the main actor. Audio that arrives between a final phrase and the next
/// request is retained and replayed in order when that request is attached.
final class RecognitionRequestSink: @unchecked Sendable {
  private static let maxBufferedBuffers = 128
  private let lock = NSLock()
  private var request: AudioBufferAppending?
  private var buffered: [AVAudioPCMBuffer] = []

  var hasBufferedAudio: Bool {
    lock.lock()
    defer { lock.unlock() }
    return !buffered.isEmpty
  }

  var pendingBufferCount: Int {
    lock.lock()
    defer { lock.unlock() }
    return buffered.count
  }

  func use(_ request: AudioBufferAppending?) {
    lock.lock()
    defer { lock.unlock() }
    self.request = request
    guard let request, !buffered.isEmpty else { return }
    // Keep the lock while flushing so the audio thread cannot append a newer
    // buffer to the request before the older buffered ones are replayed.
    for buffer in buffered {
      request.append(buffer)
    }
    buffered.removeAll(keepingCapacity: true)
  }

  func append(_ buffer: AVAudioPCMBuffer) {
    lock.lock()
    if let request {
      // Keep the lock while appending for the same ordering reason as use().
      request.append(buffer)
      lock.unlock()
      return
    }
    guard let copy = buffer.copy() as? AVAudioPCMBuffer else {
      lock.unlock()
      return
    }
    buffered.append(copy)
    if buffered.count > Self.maxBufferedBuffers {
      buffered.removeFirst(buffered.count - Self.maxBufferedBuffers)
    }
    lock.unlock()
  }
}

/// Lets about fifteen levels a second through.
final class LevelThrottle: @unchecked Sendable {
  private var last: CFAbsoluteTime = 0

  func next(_ level: Double) -> Double? {
    let now = CFAbsoluteTimeGetCurrent()
    guard now - last >= 1.0 / 15.0 else { return nil }
    last = now
    return level
  }
}
