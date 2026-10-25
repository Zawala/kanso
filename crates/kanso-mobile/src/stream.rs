//! Streams without a bridge: a sink is a closure that takes a value and says
//! whether anyone is still listening.
//!
//! The facade hands streams out as sinks (`FnMut(T) -> bool + Send +
//! 'static`) so this crate needs no generator. The bridge wraps each in its
//! stream type; `false` means the Dart listener is gone, and the sink is
//! dropped.

use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::Mutex;

use crate::boundary::{held, panic_message};

/// A sink: takes a value, returns whether to keep sending.
pub type Sink<T> = Box<dyn FnMut(T) -> bool + Send + 'static>;

/// The sinks of one stream. [`Listeners::send`] reaches every live sink, in
/// the order they were added, and drops the ones that say they are gone or
/// that panic.
pub struct Listeners<T> {
    sinks: Mutex<Vec<Sink<T>>>,
    /// Held while sending, so two threads' values reach every sink in the
    /// same order.
    sending: Mutex<()>,
}

impl<T> Default for Listeners<T> {
    fn default() -> Self {
        Listeners {
            sinks: Mutex::new(Vec::new()),
            sending: Mutex::new(()),
        }
    }
}

impl<T: Clone> Listeners<T> {
    pub fn add(&self, sink: impl FnMut(T) -> bool + Send + 'static) {
        held(&self.sinks).push(Box::new(sink));
    }

    /// Sends `value` to every sink. The sinks run with no lock of the
    /// stream's held, so one may add another.
    pub fn send(&self, value: T) {
        let _one_at_a_time = held(&self.sending);
        let mut sinks = std::mem::take(&mut *held(&self.sinks));
        // A sink that panics is dropped like one that is done; the rest hear
        // the value.
        sinks.retain_mut(|sink| {
            catch_unwind(AssertUnwindSafe(|| sink(value.clone()))).unwrap_or_else(|payload| {
                eprintln!(
                    "kanso: a listener panicked and was dropped: {}",
                    panic_message(payload.as_ref())
                );
                false
            })
        });
        // Sinks added while these ran come after them.
        let mut kept = held(&self.sinks);
        sinks.append(&mut kept);
        *kept = sinks;
    }

    /// How many sinks are listening.
    pub fn len(&self) -> usize {
        held(&self.sinks).len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use proptest::prelude::*;

    use super::*;

    /// A sink that records what it was sent and listens for `limit` values.
    fn recorder(limit: usize) -> (Arc<Mutex<Vec<u8>>>, impl FnMut(u8) -> bool + Send + 'static) {
        let seen = Arc::new(Mutex::new(Vec::new()));
        let log = seen.clone();
        (seen, move |value| {
            let mut log = log.lock().unwrap();
            log.push(value);
            log.len() < limit
        })
    }

    proptest! {
        /// Every sink sees the values in order up to the one after which it
        /// said it was done, and is never sent another.
        #[test]
        fn prop_sinks_see_values_in_order_until_they_are_gone(
            values in proptest::collection::vec(any::<u8>(), 0..20),
            limits in proptest::collection::vec(1usize..8, 1..5),
        ) {
            let listeners = Listeners::default();
            let recorded: Vec<_> = limits
                .iter()
                .map(|limit| {
                    let (seen, sink) = recorder(*limit);
                    listeners.add(sink);
                    seen
                })
                .collect();
            for value in &values {
                listeners.send(*value);
            }
            for (seen, limit) in recorded.iter().zip(&limits) {
                let expected: Vec<u8> = values.iter().copied().take(*limit).collect();
                prop_assert_eq!(&*seen.lock().unwrap(), &expected);
            }
            let alive = limits.iter().filter(|limit| **limit > values.len()).count();
            prop_assert_eq!(listeners.len(), alive);
        }
    }

    // no-prop-test: one sink that panics among others.
    #[test]
    fn a_sink_that_panics_is_dropped_and_the_others_hear_on() {
        let listeners = Listeners::<u8>::default();
        let (before, first) = recorder(10);
        let (after, last) = recorder(10);
        listeners.add(first);
        listeners.add(|_| panic!("a bad listener"));
        listeners.add(last);
        listeners.send(1);
        listeners.send(2);
        assert_eq!(*before.lock().unwrap(), vec![1, 2]);
        assert_eq!(*after.lock().unwrap(), vec![1, 2]);
        assert_eq!(listeners.len(), 2);
    }

    // no-prop-test: one sink adding another from inside a send.
    #[test]
    fn a_sink_may_add_a_sink_while_it_runs() {
        let listeners = Arc::new(Listeners::<u8>::default());
        let (seen, later) = recorder(10);
        let later = Arc::new(Mutex::new(Some(later)));
        let inner = listeners.clone();
        listeners.add(move |_| {
            if let Some(sink) = later.lock().unwrap().take() {
                inner.add(sink);
            }
            true
        });
        listeners.send(1);
        listeners.send(2);
        assert_eq!(*seen.lock().unwrap(), vec![2]);
        assert_eq!(listeners.len(), 2);
    }
}
