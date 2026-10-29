import AVFoundation

/// Converts microphone buffers to SpeechAnalyzer's format and ensures every
/// returned buffer owns its storage. One instance is used on the audio tap
/// thread for the lifetime of a capture session.
final class DictationAudioBufferConverter: @unchecked Sendable {
  private var converter: AVAudioConverter?

  func convert(_ buffer: AVAudioPCMBuffer, to format: AVAudioFormat) throws -> AVAudioPCMBuffer {
    if buffer.format == format {
      guard let copy = buffer.copy() as? AVAudioPCMBuffer else {
        throw DictationAudioConversionFailure.cannotCopy
      }
      return copy
    }

    if converter?.inputFormat != buffer.format || converter?.outputFormat != format {
      converter = AVAudioConverter(from: buffer.format, to: format)
      converter?.primeMethod = .none
    }
    guard let converter else { throw DictationAudioConversionFailure.cannotConvert }
    let ratio = format.sampleRate / buffer.format.sampleRate
    let capacity = AVAudioFrameCount((Double(buffer.frameLength) * ratio).rounded(.up))
    guard let output = AVAudioPCMBuffer(pcmFormat: format, frameCapacity: capacity) else {
      throw DictationAudioConversionFailure.cannotAllocate
    }
    var conversionError: NSError?
    let input = DictationConverterInput(buffer)
    let status = converter.convert(to: output, error: &conversionError) { _, state in
      guard let next = input.take() else {
        state.pointee = .noDataNow
        return nil
      }
      state.pointee = .haveData
      return next
    }
    if status == .error {
      if let conversionError { throw conversionError }
      throw DictationAudioConversionFailure.cannotConvert
    }
    return output
  }
}

private final class DictationConverterInput: @unchecked Sendable {
  private let lock = NSLock()
  private var buffer: AVAudioPCMBuffer?

  init(_ buffer: AVAudioPCMBuffer) { self.buffer = buffer }

  func take() -> AVAudioPCMBuffer? {
    lock.lock()
    defer { lock.unlock() }
    let next = buffer
    buffer = nil
    return next
  }
}

private enum DictationAudioConversionFailure: Error {
  case cannotCopy
  case cannotConvert
  case cannotAllocate
}
