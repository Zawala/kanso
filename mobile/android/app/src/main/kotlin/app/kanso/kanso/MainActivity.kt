package app.kanso.kanso

import androidx.glance.appwidget.updateAll
import io.flutter.embedding.android.FlutterActivity
import io.flutter.embedding.engine.FlutterEngine
import io.flutter.plugin.common.MethodChannel
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.launch

class MainActivity : FlutterActivity() {
    private val scope = CoroutineScope(SupervisorJob() + Dispatchers.Main)

    override fun configureFlutterEngine(flutterEngine: FlutterEngine) {
        super.configureFlutterEngine(flutterEngine)
        KansoCrashes.register(flutterEngine)
        // `app.kanso.kanso/widgets`: the app asks for the home-screen widget
        // to be drawn again once it has written its colors (`widget.json`).
        MethodChannel(flutterEngine.dartExecutor.binaryMessenger, "app.kanso.kanso/widgets")
            .setMethodCallHandler { call, result ->
                if (call.method != "reload") {
                    result.notImplemented()
                    return@setMethodCallHandler
                }
                scope.launch {
                    try {
                        KansoWidget().updateAll(applicationContext)
                        result.success(null)
                    } catch (error: Exception) {
                        result.error("widgets", error.message, null)
                    }
                }
            }
    }
}
