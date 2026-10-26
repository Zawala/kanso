//! The lock session: the lock password's keys, held in memory only.
//!
//! Typing the lock password once opens every note locked with it until the
//! notes are locked again, or the auto-lock time passes
//! ([`kanso_engine::lock::AUTO_LOCK`]) since a key was added or a locked note
//! saved. The keys never leave this crate: the shell hears only that the
//! notes are open or closed ([`LockSession::lock_events`]).

use std::sync::Arc;
use std::time::Instant;

use kanso_core::{DocType, Note, now_ms};
use kanso_engine::links::tags::apply_inline_tags;
use kanso_engine::lock::{self, LockView, Target};
use kanso_store::{Origin, Put};
use kanso_sync::lock::{Envelope, LockError, LockKey, LockPassword};

use crate::boundary::{guard, held};
use crate::errors::KansoError;
use crate::library::{Inner, Library};
use crate::model::{LockEvent, LockReason, LockStatus, Saved};

/// The lock session of one library.
#[derive(Clone)]
pub struct LockSession {
    pub(crate) inner: Arc<Inner>,
}

impl LockSession {
    /// Opens the locked notes with `password`. `Ok(false)` when it isn't the
    /// lock password; an error when it can't be one (too short) or nothing
    /// is locked yet. Deriving the key takes a fraction of a second.
    pub fn unlock(&self, password: String) -> Result<bool, KansoError> {
        guard(|| {
            let password = lock::password(&password).map_err(KansoError::invalid)?;
            let target = lock::any_lock(&self.inner.store)
                .ok_or_else(|| KansoError::invalid("Nothing is locked yet."))?;
            self.try_password(&target, &password)
        })
    }

    /// Opens note `id` with `password`, tried on that note's own lock (or on
    /// the locked notebook it is in, when it isn't sealed yet), and keeps
    /// the key for the session: every note locked with it opens. `Ok(false)`
    /// when it isn't the password.
    pub fn unlock_note(&self, id: &str, password: String) -> Result<bool, KansoError> {
        guard(|| {
            let password = lock::password(&password).map_err(KansoError::invalid)?;
            let note = self.library().stored(id)?;
            let target = Target::of_note(&note, &self.inner.store)
                .ok_or_else(|| KansoError::invalid("That note isn't locked."))?;
            self.try_password(&target, &password)
        })
    }

    /// Opens locked notebook `book_id` (or the locked notebook it is in)
    /// with `password`, as [`LockSession::unlock_note`] does a note.
    pub fn unlock_notebook(&self, book_id: &str, password: String) -> Result<bool, KansoError> {
        guard(|| {
            let password = lock::password(&password).map_err(KansoError::invalid)?;
            self.require_book(book_id)?;
            let target = Target::of_book(book_id, &self.inner.store)
                .ok_or_else(|| KansoError::invalid("That notebook isn't locked."))?;
            self.try_password(&target, &password)
        })
    }

    /// Tries `password` on `target`; the key it gives is kept when it opens.
    fn try_password(&self, target: &Target, password: &LockPassword) -> Result<bool, KansoError> {
        let envelope = target
            .envelope
            .as_ref()
            .map_err(|err| KansoError::invalid(lock::sentence(err)))?;
        let key = LockKey::for_envelope(password, envelope)
            .map_err(|err| KansoError::invalid(lock::sentence(&err)))?;
        match key.open(&target.id, envelope) {
            Ok(_) => {}
            Err(LockError::WrongPassword) => return Ok(false),
            Err(err) => return Err(KansoError::invalid(lock::sentence(&err))),
        }
        self.add_key(key);
        Ok(true)
    }

    /// Where the lock stands: whether a lock password exists, whether the
    /// notes are open, and whether biometrics can open them.
    pub fn lock_status(&self) -> Result<LockStatus, KansoError> {
        guard(|| {
            Ok(LockStatus {
                has_lock_password: lock::any_lock(&self.inner.store).is_some(),
                unlocked: held(&self.inner.keys).is_unlocked(),
                biometrics_on: self.biometrics_on()?,
            })
        })
    }

    /// Closes every locked note. Their keys are forgotten.
    pub fn lock(&self) -> Result<(), KansoError> {
        guard(|| {
            self.close(LockReason::Manual);
            Ok(())
        })
    }

    /// Closes every locked note because the app is leaving the foreground.
    /// The keys are forgotten, as by [`LockSession::lock`].
    pub fn lock_for_background(&self) -> Result<(), KansoError> {
        guard(|| {
            self.close(LockReason::Background);
            Ok(())
        })
    }

    /// Whether the lock password has opened the locked notes.
    pub fn is_unlocked(&self) -> Result<bool, KansoError> {
        guard(|| Ok(held(&self.inner.keys).is_unlocked()))
    }

    /// Whether note `id` can be read now: it isn't locked, or a key this
    /// session holds opens it.
    pub fn is_note_unlocked(&self, id: &str) -> Result<bool, KansoError> {
        guard(|| {
            let library = Library {
                inner: self.inner.clone(),
            };
            let mut note = library.stored(id)?;
            Ok(!library.reveal(&mut note).is_closed())
        })
    }

    /// Locks note `id` with the lock password: its text is sealed, its earlier
    /// versions are deleted, and it stays open for another auto-lock time.
    ///
    /// The first locked note sets the lock password; later ones need the
    /// same one ([`KansoError::Auth`] when it isn't).
    pub fn lock_note(&self, id: &str, password: String) -> Result<Saved, KansoError> {
        guard(|| {
            let password = lock::password(&password).map_err(KansoError::invalid)?;
            let note = self.lockable(id)?;
            let key = self.key_from_password(&password)?;
            self.add_key(key);
            self.seal_note(note)
        })
    }

    /// Locks note `id` with the key this session holds, as the desktop does
    /// when the notes are open: no password is typed.
    /// [`KansoError::Locked`] when no key is held.
    pub fn lock_note_with_session(&self, id: &str) -> Result<Saved, KansoError> {
        guard(|| {
            let note = self.lockable(id)?;
            if held(&self.inner.keys).current().is_none() {
                return Err(KansoError::Locked);
            }
            self.seal_note(note)
        })
    }

    /// Takes note `id`'s lock off: its text is saved as plain text. The note
    /// must be open ([`KansoError::Locked`] otherwise), and not in a locked
    /// notebook, where it stays locked.
    pub fn remove_note_lock(&self, id: &str) -> Result<Saved, KansoError> {
        guard(|| {
            let library = self.library();
            let stored = library.stored(id)?;
            // Reading a note in a locked notebook seals it, which moves its
            // revision.
            let mut note = stored.clone();
            let view = library.reveal(&mut note);
            match view {
                LockView::Plain => return Err(KansoError::invalid("That note isn't locked.")),
                LockView::Closed(_) => return Err(KansoError::Locked),
                LockView::Open => {}
            }
            if self.inner.store.note_in_locked_book(id)? {
                return Err(KansoError::invalid(
                    "The note is in a locked notebook, so it stays locked. Move it out first.",
                ));
            }
            // The text as it is stays, now plain; its `#tags` count again.
            Envelope::store(None, &mut note.extra);
            note.updated_at = now_ms();
            apply_inline_tags(&self.inner.store, &mut note, None);
            let value =
                serde_json::to_value(&note).map_err(|err| KansoError::storage(err.to_string()))?;
            match self
                .inner
                .store
                .put(DocType::Note, value, Put::Upsert, Origin::Ui)
            {
                Ok(written) => Ok(Saved {
                    id: written.id,
                    rev: written.rev,
                }),
                Err(kanso_store::StoreError::Conflict) => Err(KansoError::Conflict {
                    current_rev: library
                        .stored(id)
                        .ok()
                        .and_then(|n| n.rev)
                        .unwrap_or_default(),
                }),
                Err(err) => Err(err.into()),
            }
        })
    }

    /// Locks notebook `book_id`: every note in it and in the notebooks inside
    /// it is sealed, as are those put there later. Returns how many notes
    /// were sealed.
    ///
    /// With the notes open, the key held seals them and `password` is not
    /// asked. Otherwise `password` is needed: the lock password when
    /// something is locked already ([`KansoError::Auth`] when it isn't),
    /// else the first one, which sets it.
    pub fn lock_notebook(
        &self,
        book_id: &str,
        password: Option<String>,
    ) -> Result<u32, KansoError> {
        guard(|| {
            self.require_book(book_id)?;
            if Target::of_book(book_id, &self.inner.store).is_some() {
                return Err(KansoError::invalid("That notebook is locked already."));
            }
            if held(&self.inner.keys).current().is_none() {
                let password = password
                    .ok_or(KansoError::Locked)
                    .and_then(|text| lock::password(&text).map_err(KansoError::invalid))?;
                let key = self.key_from_password(&password)?;
                self.add_key(key);
            }
            let key = held(&self.inner.keys)
                .current()
                .ok_or(KansoError::Locked)?;
            let sealed = lock::seal_notebook(&self.inner.store, book_id, &key)
                .map_err(KansoError::storage)?;
            held(&self.inner.keys).touch(Instant::now());
            Ok(u32::try_from(sealed).unwrap_or(u32::MAX))
        })
    }

    /// Takes notebook `book_id`'s lock off: notes put in it from now on
    /// aren't locked, and the notes in it keep their own locks. The notebook
    /// must be open ([`KansoError::Locked`] otherwise).
    pub fn remove_notebook_lock(&self, book_id: &str) -> Result<(), KansoError> {
        guard(|| {
            self.require_book(book_id)?;
            let target = Target::of_book(book_id, &self.inner.store)
                .filter(|target| target.id == book_id)
                .ok_or_else(|| KansoError::invalid("That notebook isn't locked."))?;
            if !target.is_open(&held(&self.inner.keys)) {
                return Err(KansoError::Locked);
            }
            lock::take_lock_off(&self.inner.store, book_id).map_err(KansoError::storage)
        })
    }

    /// Whether notebook `book_id`'s notes can be read now: it (and the
    /// notebooks it is in) isn't locked, or a key this session holds opens
    /// it.
    pub fn notebook_is_open(&self, book_id: &str) -> Result<bool, KansoError> {
        guard(|| {
            self.require_book(book_id)?;
            Ok(Target::of_book(book_id, &self.inner.store)
                .is_none_or(|target| target.is_open(&held(&self.inner.keys))))
        })
    }

    /// Seals every locked note and notebook `old` opens again under a new
    /// password, and holds only the new key from now on. Returns how many
    /// `old` didn't open: they keep their own password. A wrong `old` is
    /// [`KansoError::Auth`]; biometrics, if on, keep the new key.
    pub fn change_password(&self, old: String, new: String) -> Result<u32, KansoError> {
        guard(|| {
            let wrong = || KansoError::auth(lock::sentence(&LockError::WrongPassword));
            // A shorter text can't be the password.
            let old = LockPassword::new(&old).map_err(|_| wrong())?;
            let new = lock::password(&new).map_err(KansoError::invalid)?;
            if lock::any_lock(&self.inner.store).is_none() {
                return Err(KansoError::invalid("Nothing is locked yet."));
            }
            let params = held(&self.inner.keys).params();
            let (key, skipped) = lock::rekey(&self.inner.store, &old, &new, params)
                .map_err(|words| {
                    if words == lock::sentence(&LockError::WrongPassword) {
                        wrong()
                    } else {
                        KansoError::invalid(words)
                    }
                })?;
            let was_open = {
                let mut keys = held(&self.inner.keys);
                let was_open = keys.is_unlocked();
                keys.replace(Some(key), Instant::now());
                was_open
            };
            crate::biometrics::keep(&self.inner);
            if !was_open {
                self.inner.lock_events.send(LockEvent::Unlocked);
            }
            Ok(u32::try_from(skipped).unwrap_or(u32::MAX))
        })
    }

    /// Calls `sink` each time the notes open or close, until it returns
    /// `false`.
    pub fn lock_events(&self, sink: impl FnMut(LockEvent) -> bool + Send + 'static) {
        self.inner.lock_events.add(sink);
    }

    /// Closes the notes if the auto-lock time has passed at `now`; whether it
    /// did. The library's timer calls it every few seconds; a phone that goes
    /// to the background may call it, or [`LockSession::lock`], at once.
    pub fn lock_if_idle(&self, now: Instant) -> bool {
        let idle = held(&self.inner.keys).idle(now);
        if idle {
            self.close(LockReason::Idle);
        }
        idle
    }

    /// Holds `key`, and tells listeners if the notes weren't open before.
    /// Biometrics, if on, keep the session's keys again.
    fn add_key(&self, key: LockKey) {
        self.add_keys(vec![key], true);
    }

    fn library(&self) -> Library {
        Library {
            inner: self.inner.clone(),
        }
    }

    /// Notebook `book_id` as a row of the library, or [`KansoError::NotFound`].
    fn require_book(&self, book_id: &str) -> Result<(), KansoError> {
        let books = self.inner.store.books()?;
        if books.iter().any(|book| book.id == book_id) {
            Ok(())
        } else {
            Err(KansoError::NotFound)
        }
    }

    /// Note `id`, which may be locked now: not sealed, and not in a locked
    /// notebook.
    fn lockable(&self, id: &str) -> Result<Note, KansoError> {
        let note = self.library().stored(id)?;
        if note.is_locked() || self.inner.store.note_in_locked_book(id)? {
            return Err(KansoError::invalid("That note is locked already."));
        }
        Ok(note)
    }

    /// The key `password` gives: the one that opens what is locked already
    /// ([`KansoError::Auth`] when it doesn't), or a new one when nothing is.
    fn key_from_password(&self, password: &LockPassword) -> Result<LockKey, KansoError> {
        match lock::any_lock(&self.inner.store) {
            Some(target) => lock::key_for(&target, password)
                .map_err(|_| KansoError::auth("That isn't the lock password.")),
            None => {
                let params = held(&self.inner.keys).params();
                LockKey::create(password, params)
                    .map_err(|err| KansoError::invalid(lock::sentence(&err)))
            }
        }
    }

    /// Seals `note` with the key this session holds and writes it.
    fn seal_note(&self, mut note: Note) -> Result<Saved, KansoError> {
        let key = held(&self.inner.keys)
            .current()
            .expect("a key was just held");
        let body = std::mem::take(&mut note.body);
        Envelope::store(Some(&key.seal(&note.id, &body)), &mut note.extra);
        let value =
            serde_json::to_value(&note).map_err(|err| KansoError::storage(err.to_string()))?;
        let written = self.inner.store.lock_note(value)?;
        held(&self.inner.keys).touch(Instant::now());
        Ok(Saved {
            id: written.id,
            rev: written.rev,
        })
    }

    /// Forgets the keys, and tells listeners if any were held.
    fn close(&self, reason: LockReason) {
        let was_open = {
            let mut keys = held(&self.inner.keys);
            let was_open = keys.is_unlocked();
            keys.replace(None, Instant::now());
            was_open
        };
        if was_open {
            self.inner.lock_events.send(LockEvent::Locked { reason });
        }
    }
}

