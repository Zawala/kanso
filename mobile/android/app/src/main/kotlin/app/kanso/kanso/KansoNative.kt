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
}
