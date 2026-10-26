package app.kanso.kanso

import android.app.Application
import app.kanso.kanso_lock.RustAnswer as LockAnswer

/**
 * Sets the library up as the process starts: the Keystore's storage, the
 * native library, and the `Context` the certificate verifier needs.
 *
 * Android runs this in every process of the app, WorkManager's included, so a
 * background sync finds the library ready.
 */
class KansoApplication : Application() {
    override fun onCreate() {
        super.onCreate()
        KansoSecrets.attach(this)
        System.loadLibrary(KansoNative.LIBRARY)
        KansoNative.init(this)
        // The plugin is a library and cannot see KansoNative: hand it the call.
        LockAnswer.install { ticket, status, secret -> KansoNative.biometricAnswer(ticket, status, secret) }
    }
}
