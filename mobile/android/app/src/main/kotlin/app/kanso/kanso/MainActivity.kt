package app.kanso.kanso

import android.content.Intent
import android.os.Bundle
import io.flutter.embedding.android.FlutterActivity
import io.flutter.embedding.engine.FlutterEngine

class MainActivity : FlutterActivity() {
    override fun configureFlutterEngine(flutterEngine: FlutterEngine) {
        super.configureFlutterEngine(flutterEngine)
        KansoCrashes.register(flutterEngine)
        KansoReminders.register(this, flutterEngine)
    }

    override fun cleanUpFlutterEngine(flutterEngine: FlutterEngine) {
        KansoReminders.unregister()
        super.cleanUpFlutterEngine(flutterEngine)
    }

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        // A recreated activity has already answered the tap that started it.
        if (savedInstanceState == null) {
            KansoReminders.handleOpen(this, intent)
        }
    }

    override fun onNewIntent(intent: Intent) {
        super.onNewIntent(intent)
        KansoReminders.handleOpen(this, intent)
    }

    override fun onResume() {
        super.onResume()
        KansoReminders.inFront = true
    }

    override fun onPause() {
        KansoReminders.inFront = false
        super.onPause()
    }

    override fun onRequestPermissionsResult(
        requestCode: Int,
        permissions: Array<out String>,
        grantResults: IntArray,
    ) {
        super.onRequestPermissionsResult(requestCode, permissions, grantResults)
        KansoReminders.onPermissionResult(requestCode, grantResults)
    }
}
