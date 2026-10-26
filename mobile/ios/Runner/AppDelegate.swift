import Flutter
import UIKit

@main
@objc class AppDelegate: FlutterAppDelegate, FlutterImplicitEngineDelegate {
  override func application(
    _ application: UIApplication,
    didFinishLaunchingWithOptions launchOptions: [UIApplication.LaunchOptionsKey: Any]?
  ) -> Bool {
    return super.application(application, didFinishLaunchingWithOptions: launchOptions)
  }

  // A silent push calls `KansoPush.onWake` from the `kanso_push` plugin
  // (import kanso_push), which answers the OS within 25 seconds whatever
  // happens. Step 8's background sync is wired by setting it, before the app
  // finishes launching, to a closure that runs the sync and then calls `done`
  // with the result:
  //
  //   KansoPush.onWake = { done in /* run KansoBackgroundSync, then */ done(.newData) }
  //
  // Until it is set a push is answered with .noData and nothing syncs.

  func didInitializeImplicitFlutterEngine(_ engineBridge: FlutterImplicitEngineBridge) {
    GeneratedPluginRegistrant.register(with: engineBridge.pluginRegistry)
  }
}
