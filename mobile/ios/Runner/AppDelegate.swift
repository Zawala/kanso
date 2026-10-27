import Flutter
import UIKit
import UserNotifications
import WidgetKit
import kanso_background

@main
@objc class AppDelegate: FlutterAppDelegate, FlutterImplicitEngineDelegate {
  override func application(
    _ application: UIApplication,
    didFinishLaunchingWithOptions launchOptions: [UIApplication.LaunchOptionsKey: Any]?
  ) -> Bool {
    // The background sync's handler must be registered before launch finishes.
    KansoBackground.register()
    CrashDiagnostics.shared.start()
    let launched = super.application(application, didFinishLaunchingWithOptions: launchOptions)
    UNUserNotificationCenter.current().delegate = self
    return launched
  }

  /// A reminder may be shown while Kanso is in front; other notifications
  /// keep Flutter's existing handling.
  override func userNotificationCenter(
    _ center: UNUserNotificationCenter,
    willPresent notification: UNNotification,
    withCompletionHandler completionHandler: @escaping (UNNotificationPresentationOptions) -> Void
  ) {
    if KansoReminders.shared.isReminder(notification) {
      completionHandler([.banner, .sound, .list])
    } else {
      super.userNotificationCenter(
        center,
        willPresent: notification,
        withCompletionHandler: completionHandler
      )
    }
  }

  /// A reminder tap or action is answered once by the reminder coordinator.
  override func userNotificationCenter(
    _ center: UNUserNotificationCenter,
    didReceive response: UNNotificationResponse,
    withCompletionHandler completionHandler: @escaping () -> Void
  ) {
    if KansoReminders.shared.isReminder(response.notification) {
      KansoReminders.shared.respond(to: response, done: completionHandler)
    } else {
      super.userNotificationCenter(
        center,
        didReceive: response,
        withCompletionHandler: completionHandler
      )
    }
  }

  func didInitializeImplicitFlutterEngine(_ engineBridge: FlutterImplicitEngineBridge) {
    GeneratedPluginRegistrant.register(with: engineBridge.pluginRegistry)
    if let registrar = engineBridge.pluginRegistry.registrar(forPlugin: "KansoCrashDiagnostics") {
      CrashDiagnostics.shared.register(with: registrar.messenger())
    }
    if let registrar = engineBridge.pluginRegistry.registrar(forPlugin: "KansoReminders") {
      KansoReminders.shared.register(with: registrar.messenger())
    }
    registerWidgetChannel(engineBridge.pluginRegistry)
    if let registrar = engineBridge.pluginRegistry.registrar(forPlugin: "KansoFiles") {
      KansoFiles.shared.register(with: registrar.messenger())
    }
  }

  /// `app.kanso.kanso/widgets`: the app asks for the home-screen widgets to
  /// be drawn again once it has written their colors (`widget.json`).
  private func registerWidgetChannel(_ registry: FlutterPluginRegistry) {
    guard let registrar = registry.registrar(forPlugin: "KansoWidgets") else { return }
    let channel = FlutterMethodChannel(
      name: "app.kanso.kanso/widgets", binaryMessenger: registrar.messenger())
    channel.setMethodCallHandler { call, result in
      guard call.method == "reload" else {
        result(FlutterMethodNotImplemented)
        return
      }
      WidgetCenter.shared.reloadAllTimelines()
      result(nil)
    }
  }
}
