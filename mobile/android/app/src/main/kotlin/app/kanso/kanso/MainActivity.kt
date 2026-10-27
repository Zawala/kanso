package app.kanso.kanso

import io.flutter.embedding.android.FlutterFragmentActivity
import io.flutter.embedding.engine.FlutterEngine

/** A `FragmentActivity`, which `BiometricPrompt` needs. */
class MainActivity : FlutterFragmentActivity() {
    override fun configureFlutterEngine(flutterEngine: FlutterEngine) {
        super.configureFlutterEngine(flutterEngine)
        KansoCrashes.register(flutterEngine)
    }
}
