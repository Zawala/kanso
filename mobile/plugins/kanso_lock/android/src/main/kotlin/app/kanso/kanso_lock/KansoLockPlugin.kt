package app.kanso.kanso_lock

import android.app.Activity
import android.content.Context
import android.content.pm.PackageManager
import android.view.WindowManager
import androidx.biometric.BiometricManager
import androidx.biometric.BiometricPrompt
import androidx.core.content.ContextCompat
import androidx.fragment.app.FragmentActivity
import io.flutter.embedding.engine.plugins.FlutterPlugin
import io.flutter.embedding.engine.plugins.activity.ActivityAware
import io.flutter.embedding.engine.plugins.activity.ActivityPluginBinding
import java.security.SecureRandom
import javax.crypto.Cipher
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.suspendCancellableCoroutine
import kotlinx.coroutines.withContext
import kotlin.coroutines.resume

/**
 * Biometric unlock for locked notes (`BiometricPrompt` and the Keystore) and
 * `FLAG_SECURE`. The biometric secret goes to Rust ([RustAnswer]); Dart is
 * told only how the check ended.
 */
class KansoLockPlugin :
    FlutterPlugin,
    ActivityAware,
    BiometricsHostApi,
    PrivacyHostApi {
    private var context: Context? = null
    private var activity: Activity? = null

    @Volatile
    private var secure = false

    override fun onAttachedToEngine(binding: FlutterPlugin.FlutterPluginBinding) {
        context = binding.applicationContext
        BiometricsHostApi.setUp(binding.binaryMessenger, this)
        PrivacyHostApi.setUp(binding.binaryMessenger, this)
    }

    override fun onDetachedFromEngine(binding: FlutterPlugin.FlutterPluginBinding) {
        BiometricsHostApi.setUp(binding.binaryMessenger, null)
        PrivacyHostApi.setUp(binding.binaryMessenger, null)
        context = null
    }

    override fun onAttachedToActivity(binding: ActivityPluginBinding) {
        activity = binding.activity
        applySecure()
    }

    override fun onDetachedFromActivityForConfigChanges() = onDetachedFromActivity()

    override fun onReattachedToActivityForConfigChanges(binding: ActivityPluginBinding) = onAttachedToActivity(binding)

    override fun onDetachedFromActivity() {
        activity = null
    }

    // PrivacyHostApi

    override fun setSecure(on: Boolean) {
        secure = on
        applySecure()
    }

    /** Sets or clears `FLAG_SECURE` on the current activity's window. */
    private fun applySecure() {
        val host = activity ?: return
        val on = secure
        host.runOnUiThread {
            if (on) {
                host.window.addFlags(WindowManager.LayoutParams.FLAG_SECURE)
            } else {
                host.window.clearFlags(WindowManager.LayoutParams.FLAG_SECURE)
            }
        }
    }

    // BiometricsHostApi

    override fun availability(): BiometricKind {
        val host = context ?: return BiometricKind.NONE
        return when (BiometricManager.from(host).canAuthenticate(BiometricManager.Authenticators.BIOMETRIC_STRONG)) {
            BiometricManager.BIOMETRIC_SUCCESS -> {
                val features = host.packageManager
                BiometricStatus.kindOf(
                    features.hasSystemFeature(PackageManager.FEATURE_FINGERPRINT),
                    features.hasSystemFeature(PackageManager.FEATURE_FACE),
                    features.hasSystemFeature(PackageManager.FEATURE_IRIS),
                )
            }

            BiometricManager.BIOMETRIC_ERROR_NONE_ENROLLED -> BiometricKind.NOT_ENROLLED
            else -> BiometricKind.NONE
        }
    }

    override suspend fun enroll(
        ticket: Long,
        libraryId: String,
        reason: String,
    ): BiometricOutcome {
        val host = usableActivity() ?: return finish(ticket, BiometricStatus.UNAVAILABLE, libraryId)
        val keys = BiometricKeys(host)
        val cipher: Cipher
        try {
            cipher = keys.newEncryptCipher(libraryId)
        } catch (error: Exception) {
            keys.forget(libraryId)
            return finish(ticket, BiometricStatus.FAILED, libraryId)
        }
        val checked = prompt(host, cipher, reason, ticket)
        if (checked !is Checked.Passed) {
            keys.forget(libraryId)
            return finish(ticket, (checked as Checked.Failed).status, libraryId)
        }
        val secret = ByteArray(SECRET_LENGTH).also { SecureRandom().nextBytes(it) }
        try {
            keys.store(libraryId, checked.cipher, secret)
            return finish(ticket, BiometricStatus.OK, libraryId, secret)
        } catch (error: Exception) {
            keys.forget(libraryId)
            return finish(ticket, BiometricStatus.FAILED, libraryId)
        } finally {
            secret.fill(0)
        }
    }

    override suspend fun unlock(
        ticket: Long,
        libraryId: String,
        reason: String,
    ): BiometricOutcome {
        val host = usableActivity() ?: return finish(ticket, BiometricStatus.UNAVAILABLE, libraryId)
        val keys = BiometricKeys(host)
        val cipher: Cipher
        try {
            cipher = keys.decryptCipher(libraryId)
        } catch (error: BiometricKeys.Gone) {
            return finish(ticket, BiometricStatus.INVALIDATED, libraryId)
        } catch (error: Exception) {
            return finish(ticket, BiometricStatus.FAILED, libraryId)
        }
        val checked = prompt(host, cipher, reason, ticket)
        if (checked !is Checked.Passed) {
            return finish(ticket, (checked as Checked.Failed).status, libraryId)
        }
        var secret: ByteArray? = null
        try {
            val sealed = keys.sealed(libraryId) ?: throw BiometricKeys.Gone()
            secret = checked.cipher.doFinal(sealed)
            return finish(ticket, BiometricStatus.OK, libraryId, secret)
        } catch (error: Exception) {
            // A sealed secret that no longer opens (a bad tag, a lost key) is
            // as good as gone.
            val gone = error is BiometricKeys.Gone || error is javax.crypto.AEADBadTagException
            return finish(ticket, if (gone) BiometricStatus.INVALIDATED else BiometricStatus.FAILED, libraryId)
        } finally {
            secret?.fill(0)
        }
    }

    override fun forget(libraryId: String) {
        context?.let { BiometricKeys(it).forget(libraryId) }
    }

    // Helpers

    /** The activity when it can host a `BiometricPrompt` (a `FragmentActivity`). */
    private fun usableActivity(): FragmentActivity? = activity as? FragmentActivity

    /**
     * Gives Rust the answer, so it never waits, and returns what Dart is told.
     * An invalidated answer, or a secret Rust says does not open its keys,
     * also deletes the key and the stored secret.
     */
    private fun finish(
        ticket: Long,
        status: BiometricStatus,
        libraryId: String,
        secret: ByteArray? = null,
    ): BiometricOutcome {
        val outcome = BiometricStatus.outcomeAfter(status, RustAnswer.deliver(ticket, status, secret))
        if (outcome == BiometricOutcome.INVALIDATED || (status == BiometricStatus.OK && outcome != BiometricOutcome.DONE)) {
            forget(libraryId)
        }
        return outcome
    }

    /** How a prompt ended. */
    private sealed interface Checked {
        class Passed(
            val cipher: Cipher,
        ) : Checked

        class Failed(
            val status: BiometricStatus,
        ) : Checked
    }

    /** Shows the prompt for [cipher]; cancelling the coroutine dismisses it. */
    private suspend fun prompt(
        host: FragmentActivity,
        cipher: Cipher,
        reason: String,
        ticket: Long,
    ): Checked =
        try {
            withContext(Dispatchers.Main) {
                suspendCancellableCoroutine { continuation ->
                    val executor = ContextCompat.getMainExecutor(host)
                    val callback =
                        object : BiometricPrompt.AuthenticationCallback() {
                            override fun onAuthenticationSucceeded(result: BiometricPrompt.AuthenticationResult) {
                                val sealed = result.cryptoObject?.cipher
                                if (continuation.isActive) {
                                    continuation.resume(
                                        if (sealed != null) Checked.Passed(sealed) else Checked.Failed(BiometricStatus.FAILED),
                                    )
                                }
                            }

                            override fun onAuthenticationError(
                                errorCode: Int,
                                errString: CharSequence,
                            ) {
                                if (continuation.isActive) {
                                    continuation.resume(Checked.Failed(BiometricStatus.ofPromptError(errorCode)))
                                }
                            }
                        }
                    val prompt = BiometricPrompt(host, executor, callback)
                    val info =
                        BiometricPrompt.PromptInfo
                            .Builder()
                            .setTitle(reason)
                            .setNegativeButtonText(NEGATIVE_BUTTON)
                            .setAllowedAuthenticators(BiometricManager.Authenticators.BIOMETRIC_STRONG)
                            .build()
                    continuation.invokeOnCancellation { executor.execute { prompt.cancelAuthentication() } }
                    prompt.authenticate(info, BiometricPrompt.CryptoObject(cipher))
                }
            }
        } catch (error: CancellationException) {
            // The caller gave up: Rust must not wait on this ticket.
            RustAnswer.deliver(ticket, BiometricStatus.CANCELLED)
            throw error
        }

    private companion object {
        const val SECRET_LENGTH = 32
        const val NEGATIVE_BUTTON = "Use password"
    }
}
