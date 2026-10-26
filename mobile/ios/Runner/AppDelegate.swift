import Flutter
import UIKit
import WidgetKit

@main
@objc class AppDelegate: FlutterAppDelegate, FlutterImplicitEngineDelegate {
  override func application(
    _ application: UIApplication,
    didFinishLaunchingWithOptions launchOptions: [UIApplication.LaunchOptionsKey: Any]?
  ) -> Bool {
    return super.application(application, didFinishLaunchingWithOptions: launchOptions)
  }

  func didInitializeImplicitFlutterEngine(_ engineBridge: FlutterImplicitEngineBridge) {
    GeneratedPluginRegistrant.register(with: engineBridge.pluginRegistry)
    registerWidgetChannel(engineBridge.pluginRegistry)
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
