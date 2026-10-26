package app.kanso.kanso

import android.content.Context
import android.security.keystore.KeyGenParameterSpec
import android.security.keystore.KeyProperties
import android.security.keystore.StrongBoxUnavailableException
import java.io.File
import java.io.IOException
import java.security.GeneralSecurityException
import java.security.KeyStore
import java.security.MessageDigest
import javax.crypto.Cipher
import javax.crypto.KeyGenerator
import javax.crypto.SecretKey
import javax.crypto.spec.GCMParameterSpec

/**
 * The secrets of the signed-in account (the session token, the data key) in
 * the Android Keystore's keeping.
 *
 * Rust calls the three static methods below through JNI (the contract is
 * written in `crates/kanso-mobile/src/secrets/android.rs`): [get], [set] and
 * [delete]. Dart never sees a secret.
 *
 * How a value is kept: one AES-256 key lives in the AndroidKeyStore, in the
 * secure hardware (StrongBox) when the phone has it, and never leaves it.
 * Each value is sealed with AES-GCM under that key, its item name as the
 * associated data, and written to its own file in the app's `noBackupFilesDir`,
 * so a backup never carries sealed values to a phone whose key can't open
 * them. The file is `[format byte][12-byte IV][ciphertext and tag]`, named by
 * the SHA-256 of the item name.
 *
 * The key needs no screen unlock: a background sync runs while the phone is
 * locked, after the first unlock since boot has let the app start.
 *
 * A method that cannot do its job throws; [get] returns `null` only when
 * there is no such item. Values and item names are never logged. The methods
 * are safe to call from any thread.
 */
object KansoSecrets {
    private const val KEY_ALIAS = "app.kanso.kanso.secrets.v1"
    private const val FORMAT: Byte = 1
    private const val IV_BYTES = 12
    private const val TAG_BITS = 128

    private val lock = Any()

    @Volatile
    private var folder: File? = null

    /** Gives the object the app's storage. Call it before the first use. */
    @JvmStatic
    fun attach(context: Context) {
        synchronized(lock) {
            if (folder == null) {
                val directory = File(context.applicationContext.noBackupFilesDir, "secrets")
                if (!directory.isDirectory && !directory.mkdirs()) {
                    throw IOException("The secrets folder can't be made.")
                }
                folder = directory
            }
        }
    }

    /** The bytes of [item], or `null` when there is no such item. */
    @JvmStatic
    fun get(item: String): ByteArray? {
        val file = fileOf(item)
        synchronized(lock) {
            if (!file.exists()) {
                return null
            }
            val sealed = file.readBytes()
            if (sealed.size <= 1 + IV_BYTES || sealed[0] != FORMAT) {
                throw GeneralSecurityException("A sealed secret isn't in a format this app knows.")
            }
            val cipher = Cipher.getInstance(TRANSFORMATION)
            cipher.init(
                Cipher.DECRYPT_MODE,
                key(create = false),
                GCMParameterSpec(TAG_BITS, sealed, 1, IV_BYTES),
            )
            cipher.updateAAD(item.toByteArray(Charsets.UTF_8))
            return cipher.doFinal(sealed, 1 + IV_BYTES, sealed.size - 1 - IV_BYTES)
        }
    }

    /** Keeps [value] as [item], replacing what was there. */
    @JvmStatic
    fun set(item: String, value: ByteArray) {
        val file = fileOf(item)
        synchronized(lock) {
            val cipher = Cipher.getInstance(TRANSFORMATION)
            cipher.init(Cipher.ENCRYPT_MODE, key(create = true))
            cipher.updateAAD(item.toByteArray(Charsets.UTF_8))
            val iv = cipher.iv
            if (iv.size != IV_BYTES) {
                throw GeneralSecurityException("The Keystore chose an unexpected IV size.")
            }
            val sealed = cipher.doFinal(value)
            val out = ByteArray(1 + IV_BYTES + sealed.size)
            out[0] = FORMAT
            iv.copyInto(out, 1)
            sealed.copyInto(out, 1 + IV_BYTES)
            sealed.fill(0)
            // Written beside the file and moved over it: a crash leaves the
            // old value or the new, never half of one.
            val partial = File(file.parentFile, file.name + ".part")
            partial.writeBytes(out)
            out.fill(0)
            if (!partial.renameTo(file)) {
                partial.delete()
                throw IOException("A sealed secret can't be saved.")
            }
        }
    }

    /** Removes [item]; does nothing when there is none. */
    @JvmStatic
    fun delete(item: String) {
        val file = fileOf(item)
        synchronized(lock) {
            if (file.exists() && !file.delete()) {
                throw IOException("A sealed secret can't be removed.")
            }
        }
    }

    private const val TRANSFORMATION = "AES/GCM/NoPadding"

    private fun fileOf(item: String): File {
        val directory = folder
            ?: throw IllegalStateException("KansoSecrets.attach was not called.")
        val digest = MessageDigest.getInstance("SHA-256").digest(item.toByteArray(Charsets.UTF_8))
        val name = digest.joinToString("") { "%02x".format(it) }
        return File(directory, name)
    }

    /** The Keystore's key; made on first use when [create] is set. */
    private fun key(create: Boolean): SecretKey {
        val store = KeyStore.getInstance("AndroidKeyStore").apply { load(null) }
        (store.getKey(KEY_ALIAS, null) as? SecretKey)?.let { return it }
        if (!create) {
            throw GeneralSecurityException("The Keystore no longer holds this app's key.")
        }
        return try {
            generate(strongBox = true)
        } catch (_: StrongBoxUnavailableException) {
            generate(strongBox = false)
        }
    }

    private fun generate(strongBox: Boolean): SecretKey {
        val spec = KeyGenParameterSpec.Builder(
            KEY_ALIAS,
            KeyProperties.PURPOSE_ENCRYPT or KeyProperties.PURPOSE_DECRYPT,
        )
            .setBlockModes(KeyProperties.BLOCK_MODE_GCM)
            .setEncryptionPaddings(KeyProperties.ENCRYPTION_PADDING_NONE)
            .setKeySize(256)
            .setIsStrongBoxBacked(strongBox)
            .build()
        val generator = KeyGenerator.getInstance(KeyProperties.KEY_ALGORITHM_AES, "AndroidKeyStore")
        generator.init(spec)
        return generator.generateKey()
    }
}
