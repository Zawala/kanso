package app.kanso.kanso

import android.content.Context

/**
 * The library's native side (`mobile/rust/src/android.rs`).
 *
 * [init] hands the library the app's `Context`, once per process, before any
 * request is made: the system certificate verifier and the Keystore bridge
 * need it. `KansoApplication.onCreate` calls it, so it has run in the app's
 * process whether the app opened for a screen or for a background job.
 * Calling it again does nothing.
 *
 * Shrinking keeps this class and its method name (`proguard-rules.pro`): the
 * library finds it by name.
 */
object KansoNative {
    /** The name of the library, as `System.loadLibrary` takes it. */
    const val LIBRARY = "kanso_mobile_bridge"

    @JvmStatic
    external fun init(context: Context)

    /**
     * A biometric check's answer for the ticket waiting on [ticket]
     * (`mobile/rust/src/biometric.rs`). [status]: 0 ok ([secret] is the
     * 32-byte biometric secret), 1 cancelled, 2 invalidated, 3 unavailable,
     * 4 failed. Returns 1 taken, 0 nobody waits, -1 the secret did not open
     * the kept keys, -5 a fault. Never throws; the secret is never logged.
     */
    @JvmStatic
    external fun biometricAnswer(ticket: Long, status: Int, secret: ByteArray?): Int
}
