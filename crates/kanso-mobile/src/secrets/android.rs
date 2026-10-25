//! The Android Keystore as a [`Secrets`](crate::Secrets) store, through a
//! Kotlin class the app ships.
//!
//! # The contract with Kotlin
//!
//! The class is `app.kanso.kanso.KansoSecrets`, with three **static**
//! methods (a Kotlin `object` with `@JvmStatic` members, or `companion
//! object` members with `@JvmStatic`). Item names are the secret store's:
//! short ASCII like `sync-account:l_0123...`. Values are the item's UTF-8
//! bytes.
//!
//! | Method | JNI signature | Meaning |
//! |---|---|---|
//! | `byte[] get(String item)` | `(Ljava/lang/String;)[B` | The item's bytes, or `null` when there is no such item. |
//! | `void set(String item, byte[] value)` | `(Ljava/lang/String;[B)V` | Creates or replaces the item. |
//! | `void delete(String item)` | `(Ljava/lang/String;)V` | Removes the item; does nothing when there is none. |
//!
//! Any exception a method throws is a failure of the store (the Rust side
//! clears it and returns a [`SecretError`]); a method that cannot reach the
//! Keystore must throw, never return `null` from `get`. The methods may be
//! called from any thread, concurrently, including from the WorkManager
//! process's threads, so they must not touch the UI or an `Activity`. They
//! must keep the value's bytes out of logs, and zero the arrays they hold
//! when they can.
//!
//! The class must be kept by shrinking (`proguard-rules.pro`), as must
//! `app.kanso.kanso.KansoNative`, which calls [`install`] through the glue
//! crate's `Java_app_kanso_kanso_KansoNative_init`.
//!
//! # Threads and class loading
//!
//! The JVM is found through [`jni::JavaVM::singleton`], which the
//! `init` call (a native method, so it has a JNI environment) sets up. A
//! native thread can't find an application class by name (`FindClass` there
//! uses the system loader), so [`install`] looks the class up once, on the
//! app's own thread, and keeps a global reference to it.

use std::sync::OnceLock;

use jni::errors::Error as JniError;
use jni::objects::{JByteArray, JClass, JString, JValue};
use jni::{Env, JavaVM, jni_sig, jni_str};
use kanso_sync::{SecretError, SecretStore};
use zeroize::Zeroizing;

/// The Kotlin class, in JNI's slash form.
pub const SECRETS_CLASS: &str = "app/kanso/kanso/KansoSecrets";

/// The class, found on the app's thread and kept for every other.
static CLASS: OnceLock<jni::objects::Global<JClass<'static>>> = OnceLock::new();

/// Finds `KansoSecrets` and keeps it for [`KeystoreSecrets`]. Call it from a
/// native method of the app (`KansoNative.init`), once; later calls do
/// nothing.
// no-prop-test: needs a JVM; the Rust side of the contract compiles for
// Android and the emulator test reaches it.
pub fn install(env: &mut Env) -> Result<(), SecretError> {
    if CLASS.get().is_some() {
        return Ok(());
    }
    let class = env
        .find_class(jni_str!("app/kanso/kanso/KansoSecrets"))
        .map_err(|err| SecretError(format!("can't find KansoSecrets: {err}")))?;
    let global = env
        .new_global_ref(&class)
        .map_err(|err| SecretError(format!("can't keep KansoSecrets: {err}")))?;
    // A second thread installing at once keeps the first one's class.
    let _ = CLASS.set(global);
    Ok(())
}

/// Whether [`install`] has run.
pub fn installed() -> bool {
    CLASS.get().is_some()
}

/// The Keystore-backed store.
#[derive(Clone, Copy, Debug)]
pub struct KeystoreSecrets;

/// Runs `call` on a thread attached to the JVM, with the Kotlin class.
fn with_class<T>(
    call: impl FnOnce(&mut Env, &jni::objects::Global<JClass<'static>>) -> Result<T, JniError>,
) -> Result<T, SecretError> {
    let class = CLASS
        .get()
        .ok_or_else(|| SecretError("the Keystore isn't set up yet (KansoNative.init)".into()))?;
    let vm = JavaVM::singleton().map_err(|err| SecretError(format!("no JVM: {err}")))?;
    vm.attach_current_thread(|env| call(env, class))
        .map_err(|err| SecretError(format!("the Keystore refused: {err}")))
}

impl SecretStore for KeystoreSecrets {
    fn get(&self, item: &str) -> Result<Option<Zeroizing<String>>, SecretError> {
        let bytes = with_class(|env, class| {
            let name = JString::from_str(env, item)?;
            let result = env.call_static_method(
                class,
                jni_str!("get"),
                jni_sig!("(Ljava/lang/String;)[B"),
                &[JValue::Object(&name)],
            )?;
            let object = result.l()?;
            if object.is_null() {
                return Ok(None);
            }
            let array = env.cast_local::<JByteArray>(object)?;
            Ok(Some(Zeroizing::new(env.convert_byte_array(&array)?)))
        })?;
        bytes
            .map(|bytes| {
                String::from_utf8(bytes.to_vec())
                    .map(Zeroizing::new)
                    .map_err(|_| SecretError("a Keystore item isn't text".into()))
            })
            .transpose()
    }

    fn set(&self, item: &str, value: &str) -> Result<(), SecretError> {
        with_class(|env, class| {
            let name = JString::from_str(env, item)?;
            let array = env.byte_array_from_slice(value.as_bytes())?;
            env.call_static_method(
                class,
                jni_str!("set"),
                jni_sig!("(Ljava/lang/String;[B)V"),
                &[JValue::Object(&name), JValue::Object(&array)],
            )?
            .v()
        })
    }

    fn delete(&self, item: &str) -> Result<(), SecretError> {
        with_class(|env, class| {
            let name = JString::from_str(env, item)?;
            env.call_static_method(
                class,
                jni_str!("delete"),
                jni_sig!("(Ljava/lang/String;)V"),
                &[JValue::Object(&name)],
            )?
            .v()
        })
    }
}
