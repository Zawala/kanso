//! Generators for the account's and sync's states, shared by the tests of
//! the views that show them (the desktop app's, and the mobile app's). Only
//! built for tests and with the `test-support` feature.

use kanso_sync::{QuotaKind, SignOutReason, Step, SyncProblem};
use proptest::prelude::*;

use crate::sync::SyncState;

pub fn problem() -> impl Strategy<Value = SyncProblem> {
    prop_oneof![
        prop::sample::select(SignOutReason::ALL.to_vec()).prop_map(SyncProblem::SignedOut),
        Just(SyncProblem::AccountDisabled),
        (0i64..1_000_000).prop_map(|until| SyncProblem::Paused { until }),
        (0i64..1_000_000).prop_map(|until| SyncProblem::ServerBusy { until }),
        (0i64..1_000_000).prop_map(|until| SyncProblem::BackOff { until }),
        prop::sample::select(QuotaKind::ALL.to_vec()).prop_map(SyncProblem::QuotaExceeded),
        Just(SyncProblem::ServerFull),
        (1usize..100).prop_map(|count| SyncProblem::Skipped { count }),
        Just(SyncProblem::UpdateKanso),
        Just(SyncProblem::WrongKey),
        Just(SyncProblem::ServerOutdated),
        Just(SyncProblem::Offline),
        Just(SyncProblem::Other),
    ]
}

pub fn step() -> impl Strategy<Value = Step> {
    prop::sample::select(vec![
        Step::StoppingSync,
        Step::DerivingKeys,
        Step::Contacting,
        Step::Saving,
    ])
}

pub fn sync_state() -> impl Strategy<Value = SyncState> {
    prop_oneof![
        Just(SyncState::Off),
        Just(SyncState::Syncing),
        (0i64..1_000_000_000, 0usize..4, prop::option::of(problem())).prop_map(
            |(at_ms, conflicts, problem)| SyncState::Synced {
                at_ms,
                conflicts,
                conflicted: Vec::new(),
                problem,
            }
        ),
        problem().prop_map(SyncState::Failed),
    ]
}
