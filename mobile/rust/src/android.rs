//! Android's side of the library: the native method the app calls once, as
//! the process starts.
//!
//! # The contract with Kotlin
//!
//! ```kotlin
//! package app.kanso.kanso
//!
//! object KansoNative {
//!     /** Hands the library the app's Context. Idempotent. */
//!     @JvmStatic external fun init(context: android.content.Context)
//! }
//! ```
//!
//! A static native method `init(Context)` of the class
//! `app.kanso.kanso.KansoNative`, found by the JVM under the exported name
//! `Java_app_kanso_kanso_KansoNative_init`. The app calls it from
//! `Application.onCreate`, after `System.loadLibrary("kanso_mobile_bridge")`,
//! with `applicationContext`. That is the only place it can be done in the
//! background too: WorkManager runs jobs in the app's process, after
//! `Application.onCreate`.
//!
//! It does two things, once:
//!
//! 1. Looks up the Kotlin class `app.kanso.kanso.KansoSecrets` (see
//!    `kanso_mobile::secrets::android` for its three static methods) on this
//!    thread, where the app's class loader finds it, and keeps a global
//!    reference for every other thread.
//! 2. Gives the system certificate verifier the `Context`
//!    (`kanso_tls::init_android`), without which no request over TLS is
//!    made.
//!
//! `JNI_OnLoad` is not used: Dart loads the library with `dlopen`, which
//! does not call it, and it has no `Context` in any case. Both Kotlin
//! classes must be kept by shrinking (`proguard-rules.pro`), along with the
//! verifier's own (`org.rustls.platformverifier.**`).
//!
//! A failure throws a `RuntimeException` into Kotlin, with the reason.

use std::fmt;
use std::sync::atomic::{AtomicBool, Ordering};

use jni::objects::{JByteArray, JClass, JObject};
use jni::sys::{jint, jlong};
use jni::{Env, EnvUnowned};
use zeroize::Zeroize;

/// Set once the verifier and the Keystore bridge are set up.
static DONE: AtomicBool = AtomicBool::new(false);

/// Why `init` failed, in words for the exception.
#[derive(Debug)]
struct InitError(String);

impl fmt::Display for InitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for InitError {}

impl From<jni::errors::Error> for InitError {
    fn from(err: jni::errors::Error) -> InitError {
        InitError(err.to_string())
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_app_kanso_kanso_KansoNative_init<'caller>(
    mut unowned: EnvUnowned<'caller>,
    _class: JClass<'caller>,
    context: JObject<'caller>,
) {
    unowned
        .with_env(|env| setup(env, context))
        .resolve::<jni::errors::ThrowRuntimeExAndDefault>()
}

fn setup(env: &mut Env, context: JObject) -> Result<(), InitError> {
    if DONE.load(Ordering::Acquire) {
        return Ok(());
    }
    kanso_mobile::secrets::android::install(env).map_err(|err| InitError(err.0))?;
    kanso_tls::init_android(env, context).map_err(|err| InitError(err.to_string()))?;
    DONE.store(true, Ordering::Release);
    Ok(())
}

/// `KansoNative.biometricAnswer(ticket: Long, status: Int, secret: ByteArray?): Int`:
/// the native lock plugin's answer to a biometric check, handed to
/// [`crate::biometric::kanso_biometric_answer`]'s logic. `secret` is the
/// platform's 32-byte secret when `status` is `0`, else `null`; the copy
/// made here is zeroed before returning. The caller zeroes its own.
#[unsafe(no_mangle)]
pub extern "system" fn Java_app_kanso_kanso_KansoNative_biometricAnswer<'caller>(
    mut unowned: EnvUnowned<'caller>,
    _class: JClass<'caller>,
    ticket: jlong,
    status: jint,
    secret: JByteArray<'caller>,
) -> jint {
    unowned
        .with_env(|env| -> Result<jint, jni::errors::Error> {
            let mut bytes = if secret.is_null() {
                None
            } else {
                Some(env.convert_byte_array(&secret)?)
            };
            let ticket = u64::from_ne_bytes(ticket.to_ne_bytes());
            let answer = crate::biometric::answer_with(ticket, status, bytes.as_deref());
            if let Some(bytes) = bytes.as_mut() {
                bytes.zeroize();
            }
            Ok(answer)
        })
        .resolve::<jni::errors::ThrowRuntimeExAndDefault>()
}
