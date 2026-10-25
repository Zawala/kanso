//! Signing back in at launch, and unlocking a provider sign-in from a
//! session this device already holds (docs/ACCOUNTS.md › Services, ›
//! Library binding).

use kanso_sync::keychain::{self, PendingRecoveryKey};
use kanso_sync::{LockedSession, SecretError, Session, meta_keys, unlock_with_local_key};

use super::{Flow, UnlockedCandidate};

/// What the launch check found in the secret store.
pub struct Restored {
    /// The library's ID, which names its secret-store items.
    pub library_id: String,
    /// The account's session, when the config's account, the library's
    /// binding and the secret store's item all agree.
    pub loaded: Result<Option<Session>, SecretError>,
    /// An unsaved recovery key, whatever the account.
    pub kept: Option<PendingRecoveryKey>,
    /// A provider sign-in waiting to be unlocked, with its methods
    /// refreshed.
    pub pending: Result<Option<LockedSession>, SecretError>,
    /// The saved full session of the pending one's account, which may hold
    /// the key to unlock it.
    pub local: Result<Option<Session>, SecretError>,
}

/// The candidate a saved full session unlocks `locked` into, if it does.
/// Its origin, account and key ID are checked by `unlock_with_local_key`
/// before any consent.
pub fn candidate_from_local(
    locked: &LockedSession,
    library_id: &str,
    local: Result<Option<Session>, SecretError>,
) -> Option<UnlockedCandidate> {
    match local {
        Ok(Some(local)) => match unlock_with_local_key(locked, &local) {
            Ok(unlocked) => Some(UnlockedCandidate {
                session: unlocked.session,
                locked_session: locked.clone(),
                evicted: unlocked.evicted,
                library_id: library_id.to_owned(),
                same_token_as_locked: true,
            }),
            Err(error) => {
                eprintln!("kanso: local key did not unlock provider session: {error}");
                None
            }
        },
        Ok(None) => None,
        Err(error) => {
            eprintln!("kanso: reading the saved local session: {error}");
            None
        }
    }
}

// no-prop-test: the account tasks' workers, each a sequence of calls to
// kanso_sync and the secret store; tested end to end against MemoryServer and
// MemorySecrets, and the decisions between them by property.
impl Flow {
    /// Reads what a launch needs from the secret store and the library:
    /// `account` is the config's account, `(origin, id)`. `None` when this
    /// device's identity can't be read. Call [`Flow::flush_logouts`] once
    /// the answer is in.
    pub fn restore(&self, account: Option<(&str, &str)>) -> Option<Restored> {
        let device = match self.device() {
            Ok(device) => device,
            Err(failure) => {
                eprintln!("kanso: this device's identity: {}", failure.words);
                return None;
            }
        };
        let secrets = self.secrets.as_ref();
        // An unsaved recovery key shows whatever this Mac is signed in
        // to: a cancelled recovery's, or one whose answer was lost.
        let kept =
            keychain::load_any_recovery_key(secrets, &device.library_id).unwrap_or_else(|err| {
                eprintln!("kanso: reading the unsaved recovery key: {err}");
                None
            });
        let loaded = match account {
            Some((origin, id)) => {
                let bound = |key| self.store.meta(key).ok().flatten();
                let agree = bound(meta_keys::ORIGIN).as_deref() == Some(origin)
                    && bound(meta_keys::ACCOUNT_ID).as_deref() == Some(id);
                if agree {
                    keychain::load_session(secrets, &device.library_id, origin, id)
                } else {
                    Ok(None)
                }
            }
            None => Ok(None),
        };
        // The locked provider flow is keyed only by this library ID; it
        // deliberately does not depend on config's account or the current
        // library binding.
        let mut pending = keychain::load_locked_pending(secrets, &device.library_id);
        let local = match &mut pending {
            Ok(Some(locked)) => {
                let previous = locked.clone();
                if kanso_sync::refresh_locked_methods(self.backend.as_ref(), locked).is_ok()
                    && let Err(error) =
                        keychain::update_locked_pending(secrets, &device.library_id, locked)
                {
                    eprintln!("kanso: saving refreshed locked sign-in methods: {error}");
                    *locked = previous;
                }
                keychain::load_session(
                    secrets,
                    &device.library_id,
                    &locked.origin,
                    &locked.account_id,
                )
            }
            _ => Ok(None),
        };
        Some(Restored {
            library_id: device.library_id,
            loaded,
            kept,
            pending,
            local,
        })
    }

    /// Tries to retry the logouts the server hasn't heard: the session is
    /// in already, so old tokens can take their time.
    pub fn flush_logouts(&self, library_id: &str) {
        kanso_sync::flush_pending_logouts(self.backend.as_ref(), self.secrets.as_ref(), library_id);
    }

    /// Unlocks `locked` from a saved full session of the same account, if
    /// this library holds one: an offline key source, with no server in
    /// between. The candidate waits for consent like any other.
    pub fn local_unlock(
        &self,
        locked: &LockedSession,
        library_id: &str,
    ) -> Option<UnlockedCandidate> {
        let local = keychain::load_session(
            self.secrets.as_ref(),
            library_id,
            &locked.origin,
            &locked.account_id,
        );
        candidate_from_local(locked, library_id, local)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use kanso_sync::{Choice, DeviceIdentity, MemorySecrets, MemoryServer, RecoveryKey};
    use proptest::prelude::*;

    use super::super::lab::{Lab, SERVER, paired, password};
    use super::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(64))]

        /// Refinement: a launch signs in only when the config's account, the
        /// library's binding and the secret store's item all agree; an
        /// unsaved recovery key shows whatever the account.
        #[test]
        fn prop_restore_signs_in_only_when_the_config_the_binding_and_the_item_agree(
            config: bool,
            bound: bool,
            item: bool,
            kept: bool,
        ) {
            let server = Arc::new(MemoryServer::new());
            let lab = Lab::new(&server, "Studio", 1);
            let device = lab.flow.device().unwrap();
            let session = kanso_sync::sign_up(
                server.as_ref(),
                &MemorySecrets::new(),
                SERVER,
                "alice",
                &password("alice"),
                &device,
                &|_| {},
            )
            .unwrap()
            .session;
            if bound {
                kanso_sync::bind(&lab.store, &session, Choice::Add, &lab.flow.backups, 0).unwrap();
            }
            if item {
                keychain::save_session(lab.secrets.as_ref(), &device.library_id, &session).unwrap();
            }
            if kept {
                let key = PendingRecoveryKey {
                    origin: session.origin.clone(),
                    account_id: session.account_id.clone(),
                    username: session.username.clone(),
                    server: session.server.clone(),
                    recovery_key: RecoveryKey::from_bytes([3; 16]),
                    unconfirmed: false,
                };
                keychain::save_recovery_key(lab.secrets.as_ref(), &device.library_id, &key)
                    .unwrap();
            }
            let account = config.then(|| (session.origin.clone(), session.account_id.clone()));
            let restored = lab
                .flow
                .restore(account.as_ref().map(|(o, i)| (o.as_str(), i.as_str())))
                .expect("this device is known");
            prop_assert_eq!(&restored.library_id, &device.library_id);
            prop_assert_eq!(matches!(restored.loaded, Ok(Some(_))), config && bound && item);
            prop_assert!(restored.loaded.is_ok());
            prop_assert_eq!(restored.kept.is_some(), kept);
            prop_assert!(matches!(restored.pending, Ok(None)));
            prop_assert!(matches!(restored.local, Ok(None)));
        }
    }

    #[test]
    fn a_secret_store_that_will_not_answer_leaves_the_account_for_the_next_launch() {
        let server = Arc::new(MemoryServer::new());
        let lab = Lab::new(&server, "Studio", 0);
        lab.secrets.fail_reads(true);
        let restored = lab
            .flow
            .restore(Some(("https://x.example", "acct")))
            .unwrap();
        // The binding doesn't agree, so nothing is asked of the store for the
        // account; the pending sign-in can't be read.
        assert!(matches!(restored.loaded, Ok(None)));
        assert!(restored.pending.is_err());
        assert!(restored.kept.is_none());
    }

    #[test]
    fn a_pending_provider_sign_in_unlocks_from_a_saved_session_of_its_account() {
        let p = paired();
        let library_id = &p.device.library_id;
        // Nothing saved: nothing to unlock with.
        assert!(p.second.flow.local_unlock(&p.locked, library_id).is_none());
        // The account's full session is on this device, as after a sign-in.
        keychain::save_session(p.second.secrets.as_ref(), library_id, &p.signed_in).unwrap();
        let candidate = p
            .second
            .flow
            .local_unlock(&p.locked, library_id)
            .expect("the saved session unlocks it");
        assert_eq!(candidate.session.key_id(), p.signed_in.key_id());
        assert!(candidate.same_token_as_locked);
        assert_eq!(&candidate.library_id, library_id);
        // The same decision through a launch's pending item.
        keychain::save_locked_pending(p.second.secrets.as_ref(), library_id, &p.locked).unwrap();
        let restored = p.second.flow.restore(None).unwrap();
        let locked = restored.pending.unwrap().expect("the pending sign-in");
        let candidate = candidate_from_local(&locked, library_id, restored.local);
        assert!(candidate.is_some());
    }

    #[test]
    fn a_saved_session_of_another_account_does_not_unlock_a_pending_sign_in() {
        let p = paired();
        let library_id = &p.device.library_id;
        let elsewhere = DeviceIdentity::from_parts(&format!("l_{:032x}", 9), "Elsewhere", [9; 32]);
        let bob = kanso_sync::sign_up(
            p.server.as_ref(),
            &MemorySecrets::new(),
            SERVER,
            "bob",
            &password("bob"),
            &elsewhere,
            &|_| {},
        )
        .unwrap()
        .session;
        keychain::save_session(p.second.secrets.as_ref(), library_id, &bob).unwrap();
        assert!(p.second.flow.local_unlock(&p.locked, library_id).is_none());
    }
}
