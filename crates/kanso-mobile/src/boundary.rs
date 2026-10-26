//! The edge of the library: what keeps a bug from taking the app down.
//!
//! Every facade function that can fail runs inside [`guard`]. A panic in it
//! (a bug, or an assertion in a dependency) is caught, logged and returned
//! as [`KansoError::Internal`], and the next call works as if nothing
//! happened. This needs `panic = "unwind"`, which the phones' build profile
//! sets (`[profile.mobile]`); the bridge's own panic handler is a second net
//! under it.
//!
//! The library's own locks never stay poisoned by a caught panic: [`held`]
//! takes the guard back from a poisoned mutex. The store's writer lock does
//! stay poisoned if a panic happens while a write holds it; no facade call
//! panics there, and the panic probe holds nothing.

use std::any::Any;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::{Mutex, MutexGuard, PoisonError};

use crate::errors::KansoError;

/// The longest panic message returned to the shell, in characters.
const MESSAGE_CHARS: usize = 240;

/// Runs `call`, turning a panic in it into [`KansoError::Internal`].
pub fn guard<T>(call: impl FnOnce() -> Result<T, KansoError>) -> Result<T, KansoError> {
    match catch_unwind(AssertUnwindSafe(call)) {
        Ok(result) => result,
        Err(payload) => {
            let message = panic_message(payload.as_ref());
            eprintln!("kanso: a call panicked and was caught: {message}");
            Err(KansoError::Internal { message })
        }
    }
}

/// The words of a panic's payload: the message of `panic!("...")`, cut to
/// [`MESSAGE_CHARS`].
pub fn panic_message(payload: &(dyn Any + Send)) -> String {
    let text = payload
        .downcast_ref::<&str>()
        .map(|text| (*text).to_owned())
        .or_else(|| payload.downcast_ref::<String>().cloned())
        .unwrap_or_else(|| "a panic with no message".to_owned());
    text.chars().take(MESSAGE_CHARS).collect()
}

/// Takes `mutex`, whether or not a caught panic poisoned it: the data is
/// whole, as every critical section here leaves it consistent before it
/// calls anything that can panic.
pub fn held<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(PoisonError::into_inner)
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::*;

    proptest! {
        /// Whatever a call panics with, the guard returns it as `Internal`
        /// (cut to the limit), and the next call through it is untouched.
        #[test]
        fn prop_a_panic_is_an_error_and_the_next_call_works(
            message in "\\PC{0,400}",
            value in any::<u32>(),
        ) {
            let failed: Result<(), _> = guard(|| panic!("{message}"));
            match failed {
                Err(KansoError::Internal { message: seen }) => {
                    prop_assert!(seen.chars().count() <= MESSAGE_CHARS);
                    prop_assert!(message.starts_with(&seen));
                }
                other => prop_assert!(false, "expected Internal, got {other:?}"),
            }
            prop_assert_eq!(guard(|| Ok(value)), Ok(value));
        }
    }

    // no-prop-test: a panic with a payload that isn't text.
    #[test]
    fn a_panic_without_a_message_is_still_caught() {
        let failed: Result<(), _> = guard(|| std::panic::panic_any(7_u8));
        assert_eq!(
            failed,
            Err(KansoError::Internal {
                message: "a panic with no message".into()
            })
        );
    }

    // no-prop-test: poisoning a mutex once and taking it again.
    #[test]
    fn a_mutex_poisoned_by_a_caught_panic_is_taken_back() {
        let mutex = Mutex::new(5);
        let _: Result<(), _> = guard(|| {
            let _inside = mutex.lock().unwrap();
            panic!("while holding");
        });
        assert!(mutex.is_poisoned());
        assert_eq!(*held(&mutex), 5);
    }

    // no-prop-test: errors pass through unchanged.
    #[test]
    fn errors_pass_through() {
        assert_eq!(
            guard::<()>(|| Err(KansoError::Locked)),
            Err(KansoError::Locked)
        );
    }
}
