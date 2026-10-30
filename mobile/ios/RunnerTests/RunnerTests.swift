import AVFoundation
import Flutter
import UIKit
import UserNotifications
import XCTest

@testable import Runner
@testable import kanso_dictation

class RunnerTests: XCTestCase {

  func testReminderAnswerKeepsOnlyTheIds() {
    let info: [AnyHashable: Any] = ["key": "k", "noteId": "n", "line": 4, "extra": "x"]
    let answer = KansoReminders.answer(userInfo: info, action: "open")
    XCTAssertEqual(answer?["key"] as? String, "k")
    XCTAssertEqual(answer?["noteId"] as? String, "n")
    XCTAssertEqual(answer?["line"] as? Int, 4)
    XCTAssertEqual(answer?["action"] as? String, "open")
    XCTAssertEqual(answer?.count, 4)
    XCTAssertNil(KansoReminders.answer(userInfo: ["key": "k"], action: "open"))
  }

  func testReminderActions() {
    XCTAssertEqual(
      KansoReminders.action(for: UNNotificationDefaultActionIdentifier),
      "open"
    )
    XCTAssertEqual(KansoReminders.action(for: "mark-done"), "mark-done")
    XCTAssertEqual(KansoReminders.action(for: "dismiss"), "dismiss")
    XCTAssertNil(KansoReminders.action(for: UNNotificationDismissActionIdentifier))
  }

  func testReminderPermissionMapping() {
    XCTAssertEqual(KansoReminders.permission(.authorized), "granted")
    XCTAssertEqual(KansoReminders.permission(.provisional), "granted")
    XCTAssertEqual(KansoReminders.permission(.ephemeral), "granted")
    XCTAssertEqual(KansoReminders.permission(.denied), "denied")
    XCTAssertEqual(KansoReminders.permission(.notDetermined), "notDetermined")
  }

  func testReminderDeliveryIsLiveOrKept() {
    XCTAssertEqual(
      KansoReminders.delivery(action: "open", engineUp: false, appActive: false),
      .persist
    )
    XCTAssertEqual(
      KansoReminders.delivery(action: "open", engineUp: true, appActive: false),
      .live
    )
    XCTAssertEqual(
      KansoReminders.delivery(action: "mark-done", engineUp: true, appActive: false),
      .persist
    )
    XCTAssertEqual(
      KansoReminders.delivery(action: "dismiss", engineUp: true, appActive: true),
      .live
    )
  }

  func testReminderMapParses() {
    let map: [String: Any] = [
      "key": "k", "noteId": "n", "line": 2, "title": "T", "body": "B",
      "year": 2026, "month": 10, "day": 25, "hour": 9, "minute": 0,
    ]
    let reminder = Reminder(map)
    XCTAssertEqual(reminder?.dateComponents.hour, 9)
    XCTAssertEqual(reminder?.userInfo.count, 3)
    XCTAssertNil(Reminder(["key": "k"]))
  }
}

private final class RecordingAudioBufferRequest: AudioBufferAppending {
  var buffers: [AVAudioPCMBuffer] = []

  func append(_ buffer: AVAudioPCMBuffer) {
    buffers.append(buffer)
  }
}

final class DictationAudioBufferConverterTests: XCTestCase {
  func testMatchingFormatReturnsAnIndependentBufferCopy() throws {
    let format = try XCTUnwrap(
      AVAudioFormat(
        commonFormat: .pcmFormatFloat32,
        sampleRate: 48_000,
        channels: 1,
        interleaved: false
      )
    )
    let input = try XCTUnwrap(AVAudioPCMBuffer(pcmFormat: format, frameCapacity: 4))
    input.frameLength = 4
    let inputSamples = try XCTUnwrap(input.floatChannelData?.pointee)
    inputSamples[0] = 0.25

    let output = try DictationAudioBufferConverter().convert(input, to: format)

    XCTAssertFalse(output === input)
    XCTAssertEqual(output.floatChannelData?.pointee[0], 0.25)
    inputSamples[0] = 0.75
    XCTAssertEqual(output.floatChannelData?.pointee[0], 0.25)
  }

  func testBufferedAudioIsCopiedAndFlushedInOrder() throws {
    let format = try XCTUnwrap(
      AVAudioFormat(
        commonFormat: .pcmFormatFloat32,
        sampleRate: 48_000,
        channels: 1,
        interleaved: false
      )
    )
    let sink = RecognitionRequestSink()
    let first = try makeBuffer(format: format, sample: 0.25)
    let second = try makeBuffer(format: format, sample: 0.75)

    sink.append(first)
    sink.append(second)
    XCTAssertTrue(sink.hasBufferedAudio)
    XCTAssertEqual(sink.pendingBufferCount, 2)

    // The tap buffer may be reused by the audio engine after append returns;
    // the buffered copy must not change with it.
    first.floatChannelData?.pointee[0] = 0.0

    let request = RecordingAudioBufferRequest()
    sink.use(request)

    XCTAssertFalse(sink.hasBufferedAudio)
    XCTAssertEqual(sink.pendingBufferCount, 0)
    XCTAssertEqual(request.buffers.count, 2)
    XCTAssertEqual(request.buffers[0].floatChannelData?.pointee[0], 0.25)
    XCTAssertEqual(request.buffers[1].floatChannelData?.pointee[0], 0.75)
  }

  func testAudioAfterFinalWaitsForTheNextRequest() throws {
    let format = try XCTUnwrap(
      AVAudioFormat(
        commonFormat: .pcmFormatFloat32,
        sampleRate: 48_000,
        channels: 1,
        interleaved: false
      )
    )
    let sink = RecognitionRequestSink()
    let request = RecordingAudioBufferRequest()
    sink.use(request)
    sink.append(try makeBuffer(format: format, sample: 0.1))
    XCTAssertEqual(request.buffers.count, 1)

    // The final-result path detaches the request before the short restart.
    sink.use(nil)
    sink.append(try makeBuffer(format: format, sample: 0.2))
    XCTAssertEqual(request.buffers.count, 1)
    XCTAssertEqual(sink.pendingBufferCount, 1)

    sink.use(request)
    XCTAssertEqual(request.buffers.count, 2)
    XCTAssertEqual(request.buffers[1].floatChannelData?.pointee[0], 0.2)
  }

  private func makeBuffer(
    format: AVAudioFormat,
    sample: Float
  ) throws -> AVAudioPCMBuffer {
    let buffer = try XCTUnwrap(AVAudioPCMBuffer(pcmFormat: format, frameCapacity: 4))
    buffer.frameLength = 4
    let samples = try XCTUnwrap(buffer.floatChannelData?.pointee)
    samples[0] = sample
    return buffer
  }

  func testDifferentFormatIsConvertedToRequestedSampleRate() throws {
    let inputFormat = try XCTUnwrap(
      AVAudioFormat(
        commonFormat: .pcmFormatFloat32,
        sampleRate: 48_000,
        channels: 1,
        interleaved: false
      )
    )
    let outputFormat = try XCTUnwrap(
      AVAudioFormat(
        commonFormat: .pcmFormatFloat32,
        sampleRate: 16_000,
        channels: 1,
        interleaved: false
      )
    )
    let input = try XCTUnwrap(AVAudioPCMBuffer(pcmFormat: inputFormat, frameCapacity: 480))
    input.frameLength = 480
    let samples = try XCTUnwrap(input.floatChannelData?.pointee)
    for index in 0..<Int(input.frameLength) {
      samples[index] = index.isMultiple(of: 2) ? 0.25 : -0.25
    }

    let output = try DictationAudioBufferConverter().convert(input, to: outputFormat)

    XCTAssertEqual(output.format.sampleRate, 16_000)
    XCTAssertGreaterThan(output.frameLength, 0)
  }
}
