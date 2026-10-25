//! The new device's side of "Unlocking a new device", and the approver's
//! checks (docs/ACCOUNTS.md › Unlocking a new device): where the request is,
//! what the waiting page says, and what the approver does with the request
//! at the front. The windows that show them are the apps'.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use kanso_sync::keychain;
use kanso_sync::{
    AccountError, Approval, ApprovalKeys, DeviceIdentity, LockedSession, OpenedRequest,
    PendingApproval, PinnedRequest, RequestState, Session, cancel_saved_approval,
    complete_saved_request, finish_unlock, name_device_resumable, new_approval_request_id,
    open_request, poll_saved_approval, request_approval_resumable, resume_approval,
    saved_locked_pending,
};

use super::words::{self, ErrorContext};
use super::{Failure, Flow, UnlockedCandidate};
use crate::Secrets;

/// How long the new device waits before it offers the recovery key.
pub const RECOVERY_AFTER_MS: i64 = 120_000;
/// How often the waiting device asks its request.
pub const POLL_EVERY: Duration = Duration::from_secs(2);

/// Where the new device's request is, for its waiting page.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WaitPhase {
    /// Asking the account's other devices.
    Requesting,
    /// The request is waiting for a device to answer.
    Waiting,
    /// A device answered: show the code and post the sealed name.
    Compare {
        code: String,
    },
    /// The data key is on its way.
    Unlocking,
    /// A device approved: the key arrived.
    Approved,
    /// No other device is signed in; the recovery key is the only way.
    NoDevices,
    Cancelled,
    Expired,
    Failed(String),
}

/// The waiting page's state. `started_ms` is when the request was made.
#[derive(Clone, Debug)]
pub struct WaitingState {
    pub phase: WaitPhase,
    pub code: Option<String>,
    pub started_ms: i64,
    pub other_devices: usize,
    /// False until the create answer says how many other devices could answer.
    pub count_known: bool,
    pub recovery_offered: bool,
    /// The server's request ID, once it exists.
    pub request_id: Option<String>,
    /// Set by the waiting page's Cancel; the polling thread asks the server
    /// to cancel and stops.
    pub cancel: Arc<AtomicBool>,
}

impl WaitingState {
    pub fn new(other_devices: usize, started_ms: i64) -> WaitingState {
        WaitingState {
            phase: WaitPhase::Requesting,
            code: None,
            started_ms,
            other_devices,
            count_known: true,
            recovery_offered: other_devices == 0,
            request_id: None,
            cancel: Arc::new(AtomicBool::new(false)),
        }
    }

    /// The state while the create request is in flight. The count is not
    /// known yet, so recovery is never offered early; `know_devices` settles
    /// it when the create answer arrives.
    pub fn requesting(started_ms: i64) -> WaitingState {
        let mut state = WaitingState::new(0, started_ms);
        state.count_known = false;
        state.recovery_offered = false;
        state
    }

    /// The create answer's count. With no other device the recovery key is
    /// offered at once; otherwise only after two minutes.
    pub fn know_devices(&mut self, other_devices: usize, now_ms: i64) {
        self.other_devices = other_devices;
        self.count_known = true;
        let elapsed = now_ms.saturating_sub(self.started_ms);
        self.recovery_offered = other_devices == 0 || elapsed >= RECOVERY_AFTER_MS;
    }

    /// The recovery key is offered at once with no other device, otherwise
    /// after two minutes. Before the count is known, nothing is offered.
    pub fn offer_recovery(&mut self, now_ms: i64) {
        if !self.count_known {
            return;
        }
        let elapsed = now_ms.saturating_sub(self.started_ms);
        self.recovery_offered |= self.other_devices == 0 || elapsed >= RECOVERY_AFTER_MS;
    }
}

/// What the waiting page says above the code.
pub fn waiting_line(state: &WaitingState) -> &'static str {
    match &state.phase {
        WaitPhase::Requesting => "Asking another device…",
        WaitPhase::Waiting => "Waiting for another device…",
        WaitPhase::Compare { .. } => "Waiting for another device…",
        WaitPhase::Unlocking => "Approved. Unlocking this Mac…",
        WaitPhase::Approved => "This Mac is unlocked.",
        WaitPhase::NoDevices => "No other device answered. Use your recovery key to open this Mac.",
        WaitPhase::Cancelled => "That request was cancelled.",
        WaitPhase::Expired => "That request expired.",
        WaitPhase::Failed(_) => words::CANT_REACH,
    }
}

/// How a request's name reads before the sealed name arrives.
pub fn request_title(_pending: &PendingApproval, opened_name: Option<&str>) -> String {
    match opened_name {
        Some(name) if !name.trim().is_empty() => name.to_owned(),
        _ => "A device".to_owned(),
    }
}

// ---------------------------------------------------------------------------
// The approver's checks
// ---------------------------------------------------------------------------

/// This device's ephemeral key for one request, with the request as it read
/// when the key was made. The pin is what the revealed key is checked
/// against later, never the row a later GET returns.
pub struct ApproverKey {
    pub keys: ApprovalKeys,
    pub pin: PinnedRequest,
}

impl ApproverKey {
    pub fn new(pending: &PendingApproval) -> ApproverKey {
        ApproverKey {
            keys: ApprovalKeys::generate(),
            pin: PinnedRequest::of(pending),
        }
    }
}

/// What the sheet does next with its front request.
#[derive(Debug)]
pub enum FrontStep {
    /// Post this device's key.
    Challenge,
    /// Nothing to show yet: the requester hasn't revealed its key and name.
    Wait,
    /// The request checks out: show its name and code.
    Open(OpenedRequest),
    /// Don't approve.
    Refuse(AccountError),
}

/// Decides the front request's next step from the latest row and this
/// device's pinned key. `challenged` is whether this device already posted
/// its key for the request.
pub fn front_step(
    session: &Session,
    pending: &PendingApproval,
    key: &ApproverKey,
    challenged: bool,
) -> FrontStep {
    if !key.pin.matches(pending) {
        return FrontStep::Refuse(AccountError::ApprovalChanged);
    }
    if pending.epk.is_none() || pending.sealed_name.is_none() {
        return if pending.approver_epk.is_none() && !challenged {
            FrontStep::Challenge
        } else {
            FrontStep::Wait
        };
    }
    if pending
        .approver_epk
        .is_some_and(|stored| stored != key.keys.epk())
    {
        return FrontStep::Refuse(AccountError::ApprovalChanged);
    }
    match open_request(session, pending, &key.pin, &key.keys) {
        Ok(Some(opened)) => FrontStep::Open(opened),
        Ok(None) => FrontStep::Wait,
        Err(err) => FrontStep::Refuse(err),
    }
}

impl WaitingState {
    /// Takes what the waiting worker reported.
    pub fn apply(&mut self, update: WaitUpdate, now_ms: i64) {
        match update {
            WaitUpdate::OtherDevices(count) => {
                self.know_devices(count, now_ms);
                self.phase = if count == 0 {
                    WaitPhase::NoDevices
                } else {
                    WaitPhase::Waiting
                };
            }
            WaitUpdate::Request(id) => self.request_id = Some(id),
            WaitUpdate::Code(code) => {
                self.code = Some(code.clone());
                self.phase = WaitPhase::Compare { code };
            }
            WaitUpdate::Tick => self.offer_recovery(now_ms),
        }
    }

    /// Takes how the wait ended without an unlock: the code goes when the
    /// request is over, and the recovery key is offered whatever the time.
    pub fn end(&mut self, end: &WaitEnd) {
        if matches!(
            end,
            WaitEnd::Cancelled | WaitEnd::Expired | WaitEnd::NoDevices
        ) {
            self.code = None;
        }
        self.phase = end.phase();
        self.recovery_offered = true;
    }
}

/// What the waiting worker reports while it polls.
#[derive(Debug)]
pub enum WaitUpdate {
    /// How many other devices could answer.
    OtherDevices(usize),
    /// The server's request ID.
    Request(String),
    /// A device answered: the code both screens compare.
    Code(String),
    /// One poll went by.
    Tick,
}

/// How a wait ended without an unlock.
#[derive(Debug)]
pub enum WaitEnd {
    Cancelled,
    Expired,
    NoDevices,
    Failed(Failure),
}

impl WaitEnd {
    pub fn phase(&self) -> WaitPhase {
        match self {
            WaitEnd::Cancelled => WaitPhase::Cancelled,
            WaitEnd::Expired => WaitPhase::Expired,
            WaitEnd::NoDevices => WaitPhase::NoDevices,
            WaitEnd::Failed(failure) => WaitPhase::Failed(failure.words.clone()),
        }
    }
}

/// The wait ended in an error. Some errors mean the locked credentials can't
/// be used again: `locked_cleanup` says to end the locked session.
pub fn approval_failure(error: AccountError) -> WaitEnd {
    let must_sign_out = matches!(
        &error,
        AccountError::Unauthorized { .. }
            | AccountError::KeyMismatch
            | AccountError::ApprovalChanged
            | AccountError::ApprovalRequestIdConflict
            | AccountError::Protocol(_)
    );
    let mut failure = Failure::account(error, ErrorContext::Account);
    failure.locked_cleanup = must_sign_out;
    WaitEnd::Failed(failure)
}

/// Forgets a locked session's saved approval request.
pub fn clear_approval_resume(
    secrets: &dyn Secrets,
    locked: &LockedSession,
) -> Result<(), AccountError> {
    let library_id = locked.library_id().ok_or_else(|| {
        AccountError::Protocol("the locked session has no library ID for Keychain storage".into())
    })?;
    keychain::clear_pending_approval(secrets, library_id).map_err(AccountError::Keychain)
}

// no-prop-test: the account tasks' workers, each a sequence of calls to
// kanso_sync and the secret store; tested end to end against MemoryServer and
// MemorySecrets, and the decisions between them by property.
impl Flow {
    /// Requests or resumes approval of a locked sign-in, and polls until a
    /// device answers. It only creates, polls, names and unlocks; binding
    /// decisions and promotion return to the caller's main thread.
    ///
    /// `update` hears each step as it happens; `pause` is how the worker
    /// waits between polls (`std::thread::sleep` in an app); setting
    /// `cancel` ends the request.
    pub fn wait_for_approval(
        &self,
        locked: &LockedSession,
        device: &DeviceIdentity,
        cancel: Option<&AtomicBool>,
        update: &dyn Fn(WaitUpdate),
        pause: &dyn Fn(Duration),
    ) -> Result<UnlockedCandidate, WaitEnd> {
        let backend = self.backend.as_ref();
        let secrets = self.secrets.as_ref();
        let request = if let Some(resume) = locked.approval_resume() {
            let request = match resume_approval(backend, secrets, locked) {
                Ok(request) => request,
                Err(
                    AccountError::ApprovalExpired
                    | AccountError::ApprovalUsed
                    | AccountError::NotFound,
                ) => {
                    clear_approval_resume(secrets, locked).map_err(approval_failure)?;
                    return Err(WaitEnd::Expired);
                }
                Err(error) => return Err(approval_failure(error)),
            };
            if request.id != resume.request_id() {
                return Err(approval_failure(AccountError::ApprovalChanged));
            }
            request
        } else {
            let keys = ApprovalKeys::generate();
            request_approval_resumable(backend, secrets, locked, &new_approval_request_id(), &keys)
                .map_err(approval_failure)?
        };
        if request.other_devices == 0 {
            cancel_saved_approval(backend, secrets, locked).map_err(approval_failure)?;
            return Err(WaitEnd::NoDevices);
        }
        update(WaitUpdate::OtherDevices(request.other_devices));
        update(WaitUpdate::Request(request.id.clone()));
        let mut approval: Option<Approval> = None;
        loop {
            if cancel.is_some_and(|flag| flag.load(Ordering::SeqCst)) {
                cancel_saved_approval(backend, secrets, locked).map_err(approval_failure)?;
                return Err(WaitEnd::Cancelled);
            }
            match poll_saved_approval(backend, secrets, locked) {
                Ok(RequestState::Waiting) => {}
                Ok(RequestState::Challenged { approver_epk, .. }) => {
                    // The challenge was just saved to the store, not to `locked`.
                    let saved = saved_locked_pending(secrets, locked).map_err(approval_failure)?;
                    let completed = complete_saved_request(&saved, &approver_epk)
                        .map_err(|e| WaitEnd::Failed(Failure::account(e, ErrorContext::Account)))?;
                    let code = completed.code();
                    if let Err(error) = name_device_resumable(backend, secrets, locked, device) {
                        match error {
                            AccountError::ApprovalExpired
                            | AccountError::ApprovalUsed
                            | AccountError::NotFound => {
                                clear_approval_resume(secrets, locked).map_err(approval_failure)?;
                                return Err(WaitEnd::Expired);
                            }
                            other => return Err(approval_failure(other)),
                        }
                    }
                    update(WaitUpdate::Code(code));
                    approval = Some(completed);
                }
                Ok(RequestState::Approved(approved)) => {
                    let approval = match approval.take() {
                        Some(approval) => approval,
                        None => {
                            // Resumed after the challenge: the store knows the
                            // approver's key, the caller's session may not.
                            let saved =
                                saved_locked_pending(secrets, locked).map_err(approval_failure)?;
                            let epk = saved
                                .approval_resume()
                                .and_then(|r| r.approver_epk())
                                .ok_or_else(|| approval_failure(AccountError::ApprovalChanged))?;
                            complete_saved_request(&saved, &epk).map_err(approval_failure)?
                        }
                    };
                    let unlocked = finish_unlock(backend, locked, &approval, &approved, device)
                        .map_err(approval_failure)?;
                    return Ok(UnlockedCandidate {
                        session: unlocked.session,
                        locked_session: locked.clone(),
                        evicted: unlocked.evicted,
                        library_id: device.library_id.clone(),
                        same_token_as_locked: true,
                    });
                }
                Ok(RequestState::Cancelled) => {
                    clear_approval_resume(secrets, locked).map_err(approval_failure)?;
                    return Err(WaitEnd::Cancelled);
                }
                Ok(RequestState::Expired) => {
                    clear_approval_resume(secrets, locked).map_err(approval_failure)?;
                    return Err(WaitEnd::Expired);
                }
                Err(
                    AccountError::ApprovalExpired
                    | AccountError::ApprovalUsed
                    | AccountError::NotFound,
                ) => {
                    clear_approval_resume(secrets, locked).map_err(approval_failure)?;
                    return Err(WaitEnd::Expired);
                }
                Err(
                    error @ (AccountError::Unauthorized { .. }
                    | AccountError::KeyMismatch
                    | AccountError::ApprovalChanged
                    | AccountError::ApprovalRequestIdConflict
                    | AccountError::Protocol(_)),
                ) => return Err(approval_failure(error)),
                Err(_) => {}
            }
            update(WaitUpdate::Tick);
            pause(POLL_EVERY);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::words;
    use std::cell::{Cell, RefCell};
    use std::sync::atomic::{AtomicBool, Ordering};

    use super::super::lab::paired;
    use super::{
        ApproverKey, FrontStep, POLL_EVERY, RECOVERY_AFTER_MS, WaitEnd, WaitPhase, WaitUpdate,
        WaitingState, front_step, waiting_line,
    };
    use kanso_sync::api::LoginDevice;
    use kanso_sync::keychain;
    use kanso_sync::{
        AccountError, ApprovalKeys, DeviceIdentity, LockedSession, MemorySecrets, MemoryServer,
        Password, RequestState, Session, SessionToken, challenge_request, complete_request,
        finish_approval, finish_unlock, name_device, new_approval_request_id, pending_approvals,
        poll_approval, poll_saved_approval, request_approval, request_approval_resumable, sign_in,
        sign_up,
    };
    use proptest::prelude::*;

    const SERVER: &str = "https://sync.kansoapp.app";

    fn device(name: &str, secret: u8) -> DeviceIdentity {
        DeviceIdentity::from_parts(&format!("l_{:032x}", u64::from(secret)), name, [secret; 32])
    }

    /// Alice's unlocked Mac and a second Mac signed in but locked, which
    /// asks her to approve it.
    fn two_devices() -> (MemoryServer, Session, DeviceIdentity, LockedSession) {
        let server = MemoryServer::new();
        let password = Password::new("correct horse battery staple", "alice").unwrap();
        let alice = sign_up(
            &server,
            &MemorySecrets::default(),
            SERVER,
            "alice",
            &password,
            &device("Alex's MacBook Air", 1),
            &|_| {},
        )
        .unwrap()
        .session;
        let new_device = device("Alex's Mac mini", 2);
        let signed_in = sign_in(&server, SERVER, "alice", &password, &new_device, &|_| {})
            .unwrap()
            .session;
        let token = SessionToken::new(
            signed_in
                .authorization()
                .trim_start_matches("Bearer ")
                .to_owned(),
        );
        let locked = LockedSession::new(
            signed_in.server.clone(),
            signed_in.origin.clone(),
            signed_in.account_id.clone(),
            signed_in.username.clone(),
            signed_in.device_id.clone(),
            token,
            signed_in.key_id(),
            "0000000000000000".to_owned(),
            None,
            None,
        )
        .with_login_device(LoginDevice::for_identity(
            &new_device,
            &signed_in.origin,
            &signed_in.account_id,
        ));
        (server, alice, new_device, locked)
    }

    #[test]
    fn the_sheet_opens_an_honest_request_and_its_approval_unlocks_the_new_device() {
        let (server, alice, new_device, locked) = two_devices();
        let keys_new = ApprovalKeys::generate();
        let request = request_approval(&server, &locked, &keys_new).unwrap();

        // The sheet sees the request, makes and pins its key, and challenges.
        let pending = pending_approvals(&server, &alice).unwrap();
        let key = ApproverKey::new(&pending[0]);
        assert!(matches!(
            front_step(&alice, &pending[0], &key, false),
            FrontStep::Challenge
        ));
        challenge_request(&server, &alice, &pending[0], &key.keys).unwrap();
        let pending = pending_approvals(&server, &alice).unwrap();
        assert!(matches!(
            front_step(&alice, &pending[0], &key, true),
            FrontStep::Wait
        ));

        // The new device reveals its key and names itself.
        let RequestState::Challenged { approver_epk, .. } =
            poll_approval(&server, &locked, &request).unwrap()
        else {
            panic!("challenged");
        };
        let approval_new = complete_request(&locked, &request, &keys_new, &approver_epk).unwrap();
        name_device(&server, &locked, &request, &approval_new, &new_device).unwrap();

        // The sheet opens it: same code, the device's name, and approving
        // unlocks the new device.
        let pending = pending_approvals(&server, &alice).unwrap();
        let FrontStep::Open(opened) = front_step(&alice, &pending[0], &key, true) else {
            panic!("an honest request opens");
        };
        assert_eq!(opened.name, "Alex's Mac mini");
        assert_eq!(opened.approval.code(), approval_new.code());
        finish_approval(&server, &alice, &pending[0], &opened.approval).unwrap();
        let RequestState::Approved(approved) = poll_approval(&server, &locked, &request).unwrap()
        else {
            panic!("approved");
        };
        let unlocked =
            finish_unlock(&server, &locked, &approval_new, &approved, &new_device).unwrap();
        assert_eq!(unlocked.session.key_id(), alice.key_id());
    }

    #[test]
    fn the_sheet_refuses_a_request_whose_commitment_changed_after_it_was_pinned() {
        let (server, alice, _new_device, locked) = two_devices();
        request_approval(&server, &locked, &ApprovalKeys::generate()).unwrap();
        let pending = pending_approvals(&server, &alice).unwrap();
        let key = ApproverKey::new(&pending[0]);
        let mut swapped = pending[0].clone();
        swapped.commit = ApprovalKeys::generate().commitment();
        assert!(matches!(
            front_step(&alice, &swapped, &key, false),
            FrontStep::Refuse(AccountError::ApprovalChanged)
        ));
    }

    #[test]
    fn recovery_is_offered_at_once_with_no_device_and_after_two_minutes() {
        let mut state = WaitingState::new(0, 1_000);
        assert!(state.recovery_offered);
        state.offer_recovery(1_000);
        assert!(state.recovery_offered);

        let mut state = WaitingState::new(1, 1_000);
        state.offer_recovery(1_000 + RECOVERY_AFTER_MS - 1);
        assert!(!state.recovery_offered);
        state.offer_recovery(1_000 + RECOVERY_AFTER_MS);
        assert!(state.recovery_offered);
    }

    #[test]
    fn recovery_waits_for_the_create_answer_before_it_can_be_offered() {
        let mut state = WaitingState::requesting(1_000);
        assert!(!state.count_known);
        assert!(!state.recovery_offered);
        state.offer_recovery(1_000 + RECOVERY_AFTER_MS);
        assert!(!state.recovery_offered);
        state.know_devices(1, 1_000);
        assert!(state.count_known);
        assert!(!state.recovery_offered);
        state.offer_recovery(1_000 + RECOVERY_AFTER_MS);
        assert!(state.recovery_offered);

        let mut state = WaitingState::requesting(1_000);
        state.know_devices(0, 1_000);
        assert!(state.recovery_offered);
    }

    #[test]
    fn the_waiting_page_says_what_is_happening() {
        let mut state = WaitingState::new(1, 0);
        assert_eq!(waiting_line(&state), "Asking another device…");
        state.phase = WaitPhase::Waiting;
        assert_eq!(waiting_line(&state), "Waiting for another device…");
        state.phase = WaitPhase::Compare {
            code: "123 456".into(),
        };
        assert_eq!(waiting_line(&state), "Waiting for another device…");
        state.phase = WaitPhase::NoDevices;
        assert!(waiting_line(&state).contains("recovery key"));
        state.phase = WaitPhase::Failed("x".into());
        assert_eq!(waiting_line(&state), words::CANT_REACH);
    }

    proptest::proptest! {
        #![proptest_config(proptest::prelude::ProptestConfig::with_cases(64))]

        /// The recovery fallback depends only on the count and elapsed time.
        #[test]
        fn prop_recovery_offer_follows_time_and_count(
            other in 0usize..=9,
            started in 0i64..2_000_000,
            elapsed in -10_000i64..400_000,
        ) {
            let mut state = WaitingState::new(other, started);
            state.recovery_offered = other == 0;
            state.offer_recovery(started + elapsed);
            let expected = other == 0 || elapsed >= RECOVERY_AFTER_MS;
            proptest::prop_assert_eq!(state.recovery_offered, expected);
        }
    }

    #[test]
    fn a_new_device_waits_for_approval_and_unlocks_with_the_approvers_answer() {
        let mut p = paired();
        let (backend, secrets) = (p.server.as_ref(), p.second.secrets.as_ref());
        // The request was made, and the first device answered it, before
        // this one came back to it (a relaunch while waiting): the locked
        // item holds the request's key and the approver's.
        request_approval_resumable(
            backend,
            secrets,
            &p.locked,
            &new_approval_request_id(),
            &ApprovalKeys::generate(),
        )
        .unwrap();
        let pending = pending_approvals(backend, &p.alice).unwrap();
        let key = ApproverKey::new(&pending[0]);
        challenge_request(backend, &p.alice, &pending[0], &key.keys).unwrap();
        let resumed = keychain::load_locked_pending(secrets, &p.device.library_id)
            .unwrap()
            .expect("the pending item");
        assert!(matches!(
            poll_saved_approval(backend, secrets, &resumed).unwrap(),
            RequestState::Challenged { .. }
        ));
        p.locked = keychain::load_locked_pending(secrets, &p.device.library_id)
            .unwrap()
            .expect("the pending item");
        assert!(p.locked.approval_resume().unwrap().approver_epk().is_some());

        let updates = RefCell::new(Vec::new());
        let pauses = Cell::new(0);
        let code = RefCell::new(None);
        let result = p.second.flow.wait_for_approval(
            &p.locked,
            &p.device,
            None,
            &|update| updates.borrow_mut().push(update),
            &|slept| {
                assert_eq!(slept, POLL_EVERY);
                pauses.set(pauses.get() + 1);
                assert_eq!(pauses.get(), 1, "approved after one pause");
                // The new device has revealed itself: the first device
                // opens the request and approves it.
                let pending = pending_approvals(backend, &p.alice).unwrap();
                let FrontStep::Open(opened) = front_step(&p.alice, &pending[0], &key, true) else {
                    panic!("an honest request opens");
                };
                *code.borrow_mut() = Some(opened.approval.code());
                finish_approval(backend, &p.alice, &pending[0], &opened.approval).unwrap();
            },
        );
        let candidate = result.expect("approved");
        assert_eq!(candidate.session.key_id(), p.alice.key_id());
        assert_eq!(candidate.library_id, p.device.library_id);
        assert!(candidate.same_token_as_locked);
        let code = code.into_inner().expect("the approver saw a code");
        let kinds: Vec<String> = updates
            .into_inner()
            .iter()
            .map(|update| match update {
                WaitUpdate::OtherDevices(n) => format!("devices {n}"),
                WaitUpdate::Request(_) => "request".to_owned(),
                WaitUpdate::Code(shown) => {
                    assert_eq!(*shown, code, "both screens show one code");
                    "code".to_owned()
                }
                WaitUpdate::Tick => "tick".to_owned(),
            })
            .collect();
        assert_eq!(kinds, ["devices 1", "request", "code", "tick"]);
    }

    /// Waits on `locked` as the new device, and has the first device answer
    /// its request through the whole exchange once `idle` polls have passed.
    /// With `challenged` the approver's key is already on the request.
    fn wait_approved(
        p: &super::super::lab::Paired,
        locked: &LockedSession,
        idle: usize,
        key: Option<ApproverKey>,
    ) -> Result<super::UnlockedCandidate, WaitEnd> {
        let backend = p.server.as_ref();
        let polls = Cell::new(0);
        let key = RefCell::new(key);
        p.second
            .flow
            .wait_for_approval(locked, &p.device, None, &|_| {}, &|_| {
                polls.set(polls.get() + 1);
                assert!(polls.get() < idle + 6, "the approval never arrived");
                if polls.get() <= idle {
                    return;
                }
                let pending = pending_approvals(backend, &p.alice).unwrap();
                if key.borrow().is_none() {
                    let made = ApproverKey::new(&pending[0]);
                    challenge_request(backend, &p.alice, &pending[0], &made.keys).unwrap();
                    *key.borrow_mut() = Some(made);
                } else if let FrontStep::Open(opened) =
                    front_step(&p.alice, &pending[0], key.borrow().as_ref().unwrap(), true)
                {
                    finish_approval(backend, &p.alice, &pending[0], &opened.approval).unwrap();
                }
            })
    }

    #[test]
    fn a_fresh_request_is_approved_by_another_device_without_a_relaunch() {
        let p = paired();
        assert!(p.locked.approval_resume().is_none(), "a fresh request");
        let candidate = wait_approved(&p, &p.locked, 0, None).expect("approved without a relaunch");
        assert_eq!(candidate.session.key_id(), p.alice.key_id());
        assert_eq!(candidate.library_id, p.device.library_id);
    }

    #[test]
    fn cancelling_a_locked_sign_in_cancels_a_fresh_request_the_caller_never_reloaded() {
        let p = paired();
        // The request is saved to the store; `p.locked` still has no resume
        // state, as the app's in-memory session does.
        let backend = p.server.as_ref();
        let secrets = p.second.secrets.as_ref();
        request_approval_resumable(
            backend,
            secrets,
            &p.locked,
            &new_approval_request_id(),
            &ApprovalKeys::generate(),
        )
        .unwrap();
        assert_eq!(pending_approvals(backend, &p.alice).unwrap().len(), 1);
        p.second.flow.cancel_locked(&p.locked).unwrap();
        assert!(
            pending_approvals(backend, &p.alice).unwrap().is_empty(),
            "the server's request is cancelled"
        );
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(12))]

        /// However the request began (fresh, resumed from the store before
        /// any answer, or resumed after the approver's key arrived) and
        /// however many polls pass before the first device answers, the
        /// answer to that request unlocks the new device.
        #[test]
        fn prop_any_request_is_approved_however_it_began(start in 0usize..3, idle in 0usize..3) {
            let p = paired();
            let backend = p.server.as_ref();
            let secrets = p.second.secrets.as_ref();
            let (locked, key) = if start == 0 {
                (p.locked.clone(), None)
            } else {
                request_approval_resumable(
                    backend,
                    secrets,
                    &p.locked,
                    &new_approval_request_id(),
                    &ApprovalKeys::generate(),
                )
                .unwrap();
                let mut key = None;
                if start == 2 {
                    let pending = pending_approvals(backend, &p.alice).unwrap();
                    let made = ApproverKey::new(&pending[0]);
                    challenge_request(backend, &p.alice, &pending[0], &made.keys).unwrap();
                    key = Some(made);
                    let saved = keychain::load_locked_pending(secrets, &p.device.library_id)
                        .unwrap()
                        .unwrap();
                    poll_saved_approval(backend, secrets, &saved).unwrap();
                }
                let reloaded = keychain::load_locked_pending(secrets, &p.device.library_id)
                    .unwrap()
                    .unwrap();
                (reloaded, key)
            };
            let candidate = wait_approved(&p, &locked, idle, key).expect("approved");
            prop_assert_eq!(candidate.session.key_id(), p.alice.key_id());
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(8))]

        /// Cancelling after `k` polls ends the wait as cancelled, at the next
        /// poll and not before: nobody approved it.
        #[test]
        fn prop_cancelling_the_wait_ends_it_at_the_next_poll(k in 0usize..4) {
            let p = paired();
            let cancel = AtomicBool::new(k == 0);
            let pauses = Cell::new(0);
            let result = p.second.flow.wait_for_approval(
                &p.locked,
                &p.device,
                Some(&cancel),
                &|_| {},
                &|_| {
                    pauses.set(pauses.get() + 1);
                    if pauses.get() == k {
                        cancel.store(true, Ordering::SeqCst);
                    }
                },
            );
            prop_assert!(matches!(result, Err(WaitEnd::Cancelled)));
            prop_assert_eq!(pauses.get(), k);
        }
    }

    #[test]
    fn a_wait_with_no_other_device_ends_at_once_offering_the_recovery_key() {
        let p = paired();
        // The first device goes: the new one is the account's only.
        let others: Vec<_> = kanso_sync::devices(p.server.as_ref(), &p.signed_in)
            .unwrap()
            .into_iter()
            .filter(|device| !device.current)
            .collect();
        assert_eq!(others.len(), 1);
        kanso_sync::revoke_device(p.server.as_ref(), &p.signed_in, &others[0].id).unwrap();
        let result = p
            .second
            .flow
            .wait_for_approval(&p.locked, &p.device, None, &|_| {}, &|_| {
                panic!("nothing to wait for")
            });
        let Err(end @ WaitEnd::NoDevices) = result else {
            panic!("no other device answers: {:?}", result.err());
        };
        let mut waiting = WaitingState::requesting(0);
        waiting.end(&end);
        assert_eq!(waiting.phase, WaitPhase::NoDevices);
        assert!(waiting.recovery_offered);
    }

    #[derive(Clone, Debug)]
    enum Report {
        Tick,
        Code,
        Request,
    }

    proptest! {
        /// Model: the recovery key is offered only once the count of other
        /// devices is known, and then exactly when there are none or two
        /// minutes have passed at a poll; once offered it stays offered, and
        /// the end of a wait offers it whatever the time.
        #[test]
        fn prop_waiting_state_offer_is_monotone(
            others in 0usize..4,
            before in prop::collection::vec(0i64..60_000, 0..3),
            reports in prop::collection::vec(
                (0i64..90_000, prop_oneof![Just(Report::Tick), Just(Report::Code), Just(Report::Request)]),
                0..12,
            ),
            end in prop::option::of(0usize..3),
        ) {
            let started = 5_000;
            let mut now = started;
            let mut state = WaitingState::requesting(started);
            // Ticks before the count is known offer nothing.
            for dt in before {
                now += dt;
                state.apply(WaitUpdate::Tick, now);
                prop_assert!(!state.recovery_offered);
            }
            state.apply(WaitUpdate::OtherDevices(others), now);
            let mut offered = others == 0 || now - started >= RECOVERY_AFTER_MS;
            prop_assert_eq!(state.recovery_offered, offered);
            prop_assert_eq!(
                &state.phase,
                if others == 0 { &WaitPhase::NoDevices } else { &WaitPhase::Waiting }
            );
            for (dt, report) in reports {
                now += dt;
                let was = state.recovery_offered;
                match report {
                    Report::Tick => {
                        state.apply(WaitUpdate::Tick, now);
                        offered |= others == 0 || now - started >= RECOVERY_AFTER_MS;
                    }
                    Report::Code => {
                        state.apply(WaitUpdate::Code("123 456".into()), now);
                        prop_assert_eq!(state.code.as_deref(), Some("123 456"));
                        prop_assert_eq!(&state.phase, &WaitPhase::Compare { code: "123 456".into() });
                    }
                    Report::Request => {
                        state.apply(WaitUpdate::Request("r1".into()), now);
                        prop_assert_eq!(state.request_id.as_deref(), Some("r1"));
                    }
                }
                prop_assert_eq!(state.recovery_offered, offered);
                prop_assert!(!was || state.recovery_offered, "offered stays offered");
            }
            if let Some(kind) = end {
                let end = [WaitEnd::Cancelled, WaitEnd::Expired, WaitEnd::NoDevices][kind].phase();
                let ended = match kind {
                    0 => WaitEnd::Cancelled,
                    1 => WaitEnd::Expired,
                    _ => WaitEnd::NoDevices,
                };
                state.end(&ended);
                prop_assert!(state.recovery_offered);
                prop_assert_eq!(state.phase, end);
                prop_assert_eq!(state.code, None);
            }
        }
    }
}
