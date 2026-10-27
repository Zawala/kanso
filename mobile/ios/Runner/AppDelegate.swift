import Flutter
import UIKit
import UserNotifications

@main
@objc class AppDelegate: FlutterAppDelegate, FlutterImplicitEngineDelegate {
  override func application(
    _ application: UIApplication,
    didFinishLaunchingWithOptions launchOptions: [UIApplication.LaunchOptionsKey: Any]?
  ) -> Bool {
    CrashDiagnostics.shared.start()
    let launched = super.application(application, didFinishLaunchingWithOptions: launchOptions)
    // Flutter's delegate already took the notification centre; this keeps it
    // ours, which answers for Kanso's reminders and hands the rest to it.
    UNUserNotificationCenter.current().delegate = self
    return launched
  }

  /// A reminder shows while the app is in front; anything else is Flutter's.
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

  /// A tap on a reminder or its buttons is Kanso's; anything else is Flutter's.
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
  }
}
