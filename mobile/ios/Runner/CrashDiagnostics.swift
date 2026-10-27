import Flutter
import MetricKit
import UIKit

/// Crash reports on iOS: MetricKit hands the app the crash diagnostics of an
/// earlier run at a later launch, and this writes each payload that has one
/// to `Application Support/crash-reports/raw/<uuid>.metrickit.json`. The Dart
/// side (`mobile/lib/crash/`) turns it into a scrubbed report, after the
/// whitelist in docs/CRASH-REPORTS.md, and deletes the file. Nothing is sent
/// from here, and nothing is read from the payload.
///
/// It also answers the `app.kanso/crash` channel with the device model and
/// the OS version for the report.
final class CrashDiagnostics: NSObject, MXMetricManagerSubscriber {
  static let shared = CrashDiagnostics()

  private static let channelName = "app.kanso/crash"
  private static let folderName = "crash-reports"

  /// Starts listening; call it as the app finishes launching.
  func start() {
    MXMetricManager.shared.add(self)
  }

  /// Answers the `app.kanso/crash` channel.
  func register(with messenger: FlutterBinaryMessenger) {
    let channel = FlutterMethodChannel(
      name: CrashDiagnostics.channelName,
      binaryMessenger: messenger
    )
    channel.setMethodCallHandler { call, result in
      switch call.method {
      case "deviceModel":
        result(CrashDiagnostics.deviceModel())
      case "osVersion":
        result(UIDevice.current.systemVersion)
      default:
        result(FlutterMethodNotImplemented)
      }
    }
  }

  func didReceive(_ payloads: [MXDiagnosticPayload]) {
    for payload in payloads {
      guard let crashes = payload.crashDiagnostics, !crashes.isEmpty else {
        continue
      }
      write(payload.jsonRepresentation())
    }
  }

  private func write(_ json: Data) {
    do {
      let support = try FileManager.default.url(
        for: .applicationSupportDirectory,
        in: .userDomainMask,
        appropriateFor: nil,
        create: true
      )
      var folder = support.appendingPathComponent(
        CrashDiagnostics.folderName,
        isDirectory: true
      )
      let raw = folder.appendingPathComponent("raw", isDirectory: true)
      try FileManager.default.createDirectory(
        at: raw,
        withIntermediateDirectories: true
      )
      // Crash reports stay on the phone, like the notes.
      var excluded = URLResourceValues()
      excluded.isExcludedFromBackup = true
      try folder.setResourceValues(excluded)
      var rawFolder = raw
      try rawFolder.setResourceValues(excluded)
      let file = raw.appendingPathComponent(
        "\(UUID().uuidString).metrickit.json"
      )
      try json.write(to: file, options: .atomic)
    } catch {
      // A report that can't be kept is lost; the app is not to fail for it.
    }
  }

  /// The machine identifier, `iPhone17,1`; a simulator says which phone it is.
  private static func deviceModel() -> String {
    if let simulated = ProcessInfo.processInfo
      .environment["SIMULATOR_MODEL_IDENTIFIER"]
    {
      return simulated
    }
    var system = utsname()
    uname(&system)
    return withUnsafePointer(to: &system.machine) { pointer in
      pointer.withMemoryRebound(
        to: CChar.self,
        capacity: Int(_SYS_NAMELEN)
      ) { String(cString: $0) }
    }
  }
}
