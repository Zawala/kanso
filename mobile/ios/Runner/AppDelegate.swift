import Flutter
import UIKit
import kanso_push

@main
@objc class AppDelegate: FlutterAppDelegate, FlutterImplicitEngineDelegate {
  override func application(
    _ application: UIApplication,
    didFinishLaunchingWithOptions launchOptions: [UIApplication.LaunchOptionsKey: Any]?
  ) -> Bool {
    return super.application(application, didFinishLaunchingWithOptions: launchOptions)
  }

  // A silent push wakes the app to sync, launching it in the background if
  // it isn't running: `KansoPush.receive` (the `kanso_push` plugin) calls
  // `KansoPush.onWake` and answers the OS within 25 seconds whatever happens.
  // Until `onWake` is set (step 8's background sync), a push is answered
  // with .noData and nothing syncs.
  override func application(
    _ application: UIApplication,
    didReceiveRemoteNotification userInfo: [AnyHashable: Any],
    fetchCompletionHandler completionHandler: @escaping (UIBackgroundFetchResult) -> Void
  ) {
    KansoPush.receive(completionHandler)
  }

  func didInitializeImplicitFlutterEngine(_ engineBridge: FlutterImplicitEngineBridge) {
    GeneratedPluginRegistrant.register(with: engineBridge.pluginRegistry)
  }
}
