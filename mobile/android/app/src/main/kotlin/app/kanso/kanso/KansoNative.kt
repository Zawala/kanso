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
     * The sync that runs with no app, for the WorkManager job: syncs the
     * library file at [path] within [budgetMs] and returns the code of
     * `mobile/rust/src/background.rs` (0 or 1 done, negative a reason).
     * Never throws.
     */
    @JvmStatic
    external fun backgroundSync(path: String, budgetMs: Int): Int

    /**
     * A passkey sheet's answer for the ceremony waiting on [ticket]
     * (`mobile/rust/src/passkey.rs`): 1 taken, 0 nobody waits, -5 a fault.
     * Never throws; the answer is never logged.
     */
    @JvmStatic
    external fun passkeyAnswer(ticket: Long, json: String): Int
}
